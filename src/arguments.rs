use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[clap(long, default_value = "crates")]
  db_name: String,
}

impl Arguments {
  pub(crate) async fn run(self) -> Result {
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

    let consumer = Consumer::new(
      channel_b,
      Db::connect(&self.db_name).await?,
      Index::open()?,
    );

    consumer.listen().await?;

    let publisher = Publisher::new(channel_a)?;

    publisher.publish(PublishOptions::default()).await?;

    Ok(())
  }
}
