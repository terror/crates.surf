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
    server::Server,
    subcommand::Subcommand,
  },
  anyhow::anyhow,
  async_trait::async_trait,
  axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
  },
  clap::Parser,
  crates_io_api::{AsyncClient, Crate, CratesQuery},
  elasticsearch::{
    http::response::Response,
    http::transport::Transport,
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch, GetParts, IndexParts, SearchParts,
  },
  futures_lite::stream::StreamExt,
  http::Method,
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
  serde::{Deserialize, Serialize},
  serde_json::{json, Value},
  sqlx::{migrate::MigrateDatabase, PgPool, Postgres},
  std::{net::SocketAddr, process, str::FromStr, sync::Arc, time::Duration},
  tower_http::cors::{Any, CorsLayer},
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
mod server;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  if let Err(error) = Arguments::parse().run().await {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
