use {
  async_trait::async_trait,
  clap::Parser,
  crates_io_api::{AsyncClient, Crate, CratesQuery},
  anyhow::anyhow,
  elasticsearch::{
    http::response::Response,
    http::transport::Transport,
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch, GetParts, IndexParts, SearchParts,
  },
  futures_lite::stream::StreamExt,
  lapin::{
    options::{
      BasicAckOptions, BasicConsumeOptions, BasicPublishOptions,
      QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
  },
  lazy_static::lazy_static,
  serde::Serialize,
  serde_json::{json, Value},
  sqlx::{migrate::MigrateDatabase, PgPool, Postgres},
  std::{process, str::FromStr, time::Duration},
  tracing::{error, info, trace},
};

#[derive(Debug, Default)]
struct CratePublishOptions {
  page: Option<u64>,
  page_size: Option<u64>,
}

struct CratePublisher {
  channel: Channel,
  client: AsyncClient,
}

impl CratePublisher {
  const PAGE_SIZE: u64 = 100;
  const USER_AGENT: &str = "crates.surf (admin@crates.surf)";

  fn new(channel: Channel) -> Result<Self> {
    Ok(Self {
      channel,
      client: AsyncClient::new(Self::USER_AGENT, Duration::from_millis(1000))?,
    })
  }

  async fn publish(self, options: CratePublishOptions) -> Result {
    let mut curr_page = options.page.unwrap_or(1);

    let mut query = CratesQuery::builder()
      .page_size(options.page_size.unwrap_or(Self::PAGE_SIZE))
      .build();

    query.set_page(curr_page);

    let mut result = self.client.crates(query.clone()).await?;

    while result.crates.len() != 0 {
      info!("Found crates on page {curr_page}...");

      let names = result
        .crates
        .iter()
        .map(|curr| curr.name.clone())
        .collect::<Vec<_>>();

      info!("Fetched crates: {:?}", names);

      for partial in result.crates {
        self
          .channel
          .basic_publish(
            "",
            "crates",
            BasicPublishOptions::default(),
            serde_json::to_string(&partial)?.as_bytes(),
            BasicProperties::default(),
          )
          .await?
          .await?;
      }

      curr_page += 1;

      result = self.client.crates(query.clone()).await?;
    }

    Ok(())
  }
}

lazy_static! {
  static ref SCHEMA: Value = json!({
    "mappings": {
      "properties": {
        "id": { "type": "keyword" },
        "name": { "type": "text" },
        "description": { "type": "text" },
        "license": { "type": "text" },
        "documentation": { "type": "text" },
        "homepage": { "type": "text" },
        "repository": { "type": "text" },
        "downloads": { "type": "long" },
        "recent_downloads": { "type": "long" },
        "categories": { "type": "text" },
        "keywords": { "type": "text" },
        "versions": { "type": "long" },
        "max_version": { "type": "text" },
        "max_stable_version": { "type": "text" },
        "links": {
          "properties": {
            "owner_team": { "type": "text" },
            "owner_user": { "type": "text" },
            "owners": { "type": "text" },
            "reverse_dependencies": { "type": "text" },
            "version_downloads": { "type": "text" },
            "versions": { "type": "text" }
          }
        },
        "created_at": { "type": "date" },
        "updated_at": { "type": "date" },
        "exact_match": { "type": "boolean" }
      }
    }
  });
}

pub(crate) trait ResponseExt {
  fn raise_for_status(self, status: StatusCode) -> Result<Self>
  where
    Self: Sized;
}

impl ResponseExt for Response {
  fn raise_for_status(self, status: StatusCode) -> Result<Self> {
    match self.status_code() == status {
      true => Ok(self),
      _ => Err(anyhow!("Failed to execute request: {:?}", self)),
    }
  }
}

#[derive(Debug)]
pub(crate) struct Index {
  client: Elasticsearch,
}

use axum::{
  extract::{Extension, Query},
  http::StatusCode,
  response::IntoResponse,
  routing::get,
  Json, Router,
};

impl Index {
  const INDEX_ID: &str = "crate-index";

  pub(crate) fn open() -> Result<Self> {
    Ok(Self {
      client: Elasticsearch::new(Transport::single_node(
        "http://localhost:9200",
      )?),
    })
  }

  pub(crate) async fn ingest(&self, item: Crate) -> Result {
    info!("Ingesting item: {:?}", item);

    if !self.client.has_index(Index::INDEX_ID).await? {
      self
        .client
        .create_index(Index::INDEX_ID, SCHEMA.to_owned())
        .await?
        .raise_for_status(StatusCode::OK)?;
    }

    trace!("Checking item {}...", item.name);

    if self.client.has_document(Index::INDEX_ID, &item.id).await? {
      return Ok(());
    }

    trace!("Indexing package {}...", item.name);

    self
      .client
      .index_document(Index::INDEX_ID, &item.id, &item)
      .await?
      .raise_for_status(StatusCode::CREATED)?;

    Ok(())
  }

  pub(crate) async fn search(&self, query: &str) -> Result<serde_json::Value> {
    info!("Received query: {query}");

    let response = self
      .client
      .query(
        Index::INDEX_ID,
        json!({
          "query": {
            "query_string": {
              "query": query
            }
          }
        }),
      )
      .await?
      .raise_for_status(StatusCode::OK)?;

    Ok(response.json().await?)
  }
}

#[async_trait]
pub(crate) trait ElasticsearchExt {
  async fn create_index<T: Serialize + Send>(
    &self,
    index_id: &str,
    body: T,
  ) -> Result<Response>;
  async fn has_document(
    &self,
    index_id: &str,
    document_id: &str,
  ) -> Result<bool>;
  async fn has_index(&self, index_id: &str) -> Result<bool>;
  async fn index_document<T: Serialize + Send>(
    &self,
    index_id: &str,
    document_id: &str,
    document: T,
  ) -> Result<Response>;
  async fn query<T: Serialize + Send>(
    &self,
    index_id: &str,
    body: T,
  ) -> Result<Response>;
}

#[async_trait]
impl ElasticsearchExt for Elasticsearch {
  async fn create_index<T: Serialize + Send>(
    &self,
    index_id: &str,
    body: T,
  ) -> Result<Response> {
    Ok(
      self
        .indices()
        .create(IndicesCreateParts::Index(index_id))
        .body(body)
        .send()
        .await?,
    )
  }

  async fn has_document(
    &self,
    index_id: &str,
    document_id: &str,
  ) -> Result<bool> {
    Ok(
      self
        .get(GetParts::IndexId(index_id, document_id))
        .send()
        .await?
        .status_code()
        .is_success(),
    )
  }

  async fn has_index(&self, index_id: &str) -> Result<bool> {
    Ok(
      self
        .indices()
        .exists(IndicesExistsParts::Index(&[index_id]))
        .send()
        .await?
        .status_code()
        .is_success(),
    )
  }

  async fn index_document<T: Serialize + Send>(
    &self,
    index_id: &str,
    document_id: &str,
    body: T,
  ) -> Result<Response> {
    Ok(
      self
        .index(IndexParts::IndexId(index_id, document_id))
        .body(body)
        .send()
        .await?,
    )
  }

  async fn query<T: Serialize + Send>(
    &self,
    index_id: &str,
    body: T,
  ) -> Result<Response> {
    Ok(
      self
        .search(SearchParts::Index(&[index_id]))
        .body(body)
        .send()
        .await?,
    )
  }
}

#[derive(Debug)]
struct CrateConsumer {
  channel: Channel,
  db: Db,
  index: Index,
}

impl CrateConsumer {
  const QUEUE: &str = "crates";
  const TAG: &str = "crate_consumer";

  fn new(channel: Channel, db: Db, index: Index) -> Self {
    Self { channel, db, index }
  }

  async fn listen(self) -> Result {
    let mut consumer = self
      .channel
      .basic_consume(
        Self::QUEUE,
        Self::TAG,
        BasicConsumeOptions::default(),
        FieldTable::default(),
      )
      .await?;

    tokio::task::spawn(async move {
      if let Err(error) = self.receive(&mut consumer).await {
        error!(?error, "Failed to receive");
      }
    });

    Ok(())
  }

  async fn receive(self, consumer: &mut Consumer) -> Result {
    trace!("Waiting for delivery...");

    while let Some(delivery) = consumer.next().await {
      let delivery = delivery?;

      delivery.ack(BasicAckOptions::default()).await?;

      let payload =
        serde_json::from_str::<Crate>(&std::str::from_utf8(&delivery.data)?)?;

      info!("Received crate: {:?}", payload.name);
    }

    Ok(())
  }
}

#[derive(Debug)]
struct Db {
  pool: PgPool,
}

impl Db {
  async fn connect(db_name: &str) -> Result<Self> {
    let db_url =
      format!("postgresql://postgres:postgres@localhost:5432/{}", db_name);

    if !Postgres::database_exists(&db_url).await? {
      Postgres::create_database(&db_url).await.unwrap();
    }

    let options = sqlx::postgres::PgConnectOptions::from_str(&db_url)?;

    let pool = PgPool::connect_with(options).await?;

    info!("Connected to PostgreSQL");

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Ran migrations");

    Ok(Self { pool })
  }
}

#[derive(Parser)]
struct Arguments {
  #[clap(long, default_value = "crates")]
  db_name: String,
}

impl Arguments {
  async fn run(self) -> Result {
    let amqp_addr = std::env::var("AMQP_ADDR")
      .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    let queue_conn =
      Connection::connect(&amqp_addr, ConnectionProperties::default()).await?;

    info!("Connected to RabbitMQ");

    let (channel_a, channel_b) = (
      queue_conn.create_channel().await?,
      queue_conn.create_channel().await?,
    );

    let queue = channel_a
      .queue_declare(
        "crates",
        QueueDeclareOptions::default(),
        FieldTable::default(),
      )
      .await?;

    info!(?queue, "Declared queue");

    let crate_consumer = CrateConsumer::new(
      channel_b,
      Db::connect(&self.db_name).await?,
      Index::open()?,
    );

    crate_consumer.listen().await?;

    let crate_publisher = CratePublisher::new(channel_a)?;

    crate_publisher
      .publish(CratePublishOptions::default())
      .await?;

    Ok(())
  }
}

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  if let Err(error) = Arguments::parse().run().await {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
