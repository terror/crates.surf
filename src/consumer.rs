use super::*;

#[derive(Debug)]
pub(crate) struct Consumer {
  channel: Channel,
  _db: Db,
  index: Index,
}

impl Consumer {
  const QUEUE: &str = "crates";
  const TAG: &str = "crate_consumer";

  pub(crate) fn new(channel: Channel, db: Db, index: Index) -> Self {
    Self {
      channel,
      _db: db,
      index,
    }
  }

  pub(crate) async fn listen(self) -> Result {
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

  async fn receive(self, consumer: &mut LapinConsumer) -> Result {
    trace!("Waiting for delivery...");

    while let Some(delivery) = consumer.next().await {
      let delivery = delivery?;

      delivery.ack(BasicAckOptions::default()).await?;

      let payload =
        serde_json::from_str::<Crate>(&std::str::from_utf8(&delivery.data)?)?;

      info!("Received crate: {:?}", payload.name);

      self.index.ingest(payload).await?;
    }

    Ok(())
  }
}
