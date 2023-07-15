use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Server {
  #[clap(long, default_value = "crates")]
  db_name: String,
  #[clap(long)]
  index: bool,
}

#[derive(Deserialize)]
pub(crate) struct Params {
  pub(crate) query: String,
}

impl Server {
  pub(crate) async fn run(self) -> Result {
    let port = std::env::var("PORT")
      .unwrap_or_else(|_| "8000".into())
      .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let rabbit_conn = Connection::connect(
      &std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into()),
      ConnectionProperties::default(),
    )
    .await?;

    info!("Connected to RabbitMQ");

    let (channel_a, channel_b) = (
      rabbit_conn.create_channel().await?,
      rabbit_conn.create_channel().await?,
    );

    let queue = channel_a
      .queue_declare(
        "crates",
        QueueDeclareOptions::default(),
        FieldTable::default(),
      )
      .await?;

    info!(?queue, "Declared queue");

    let (db, index) =
      (Db::connect(&self.db_name).await?, Arc::new(Index::open()?));

    if self.index {
      let consumer = Consumer::new(channel_b, db, index.clone());

      consumer.listen().await?;

      let publisher = Publisher::new(channel_a)?;

      tokio::spawn(async move {
        if let Err(err) = publisher.publish(PublishOptions::default()).await {
          error!(?err, "Failed to publish");
        }
      });
    }

    info!("Listening on {}...", addr);

    axum::Server::bind(&addr)
      .serve(
        Router::new()
          .route("/api/search", get(Self::search))
          .layer(Extension(index))
          .layer(
            CorsLayer::new()
              .allow_methods([Method::GET])
              .allow_origin(Any),
          )
          .into_make_service(),
      )
      .await?;

    Ok(())
  }

  async fn search(
    Query(params): Query<Params>,
    index: Extension<Arc<Index>>,
  ) -> impl IntoResponse {
    match index.search(&params.query).await {
      Ok(payload) => (StatusCode::OK, Json(Some(payload))),
      Err(error) => {
        eprintln!("Error serving request for query {}: {error}", params.query);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
      }
    }
  }
}
