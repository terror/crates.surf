use {
  crate::{
    arguments::Arguments,
    channel_ext::ChannelExt,
    consumer::Consumer,
    db::Db,
    elasticsearch_ext::ElasticsearchExt,
    index::Index,
    publisher::{PublishOptions, Publisher},
    response_ext::ResponseExt,
  },
  anyhow::anyhow,
  async_trait::async_trait,
  axum::http::StatusCode,
  clap::Parser,
  crates_io_api::{AsyncClient, Crate, CratesQuery},
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
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
    Consumer as LapinConsumer,
  },
  lazy_static::lazy_static,
  serde::Serialize,
  serde_json::{json, Value},
  sqlx::{migrate::MigrateDatabase, PgPool, Postgres},
  std::{process, str::FromStr, time::Duration},
  tracing::{error, info, trace},
};

mod arguments;
mod channel_ext;
mod consumer;
mod db;
mod elasticsearch_ext;
mod index;
mod publisher;
mod response_ext;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  if let Err(error) = Arguments::parse().run().await {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
