use super::*;

#[async_trait]
pub(crate) trait ChannelExt {
  async fn publish<T: Serialize + Send + Sync>(
    &self,
    route: &str,
    payload: T,
  ) -> Result<Confirmation>;
}

#[async_trait]
impl ChannelExt for Channel {
  async fn publish<T: Serialize + Send + Sync>(
    &self,
    route: &str,
    payload: T,
  ) -> Result<Confirmation> {
    Ok(
      self
        .basic_publish(
          "",
          route,
          BasicPublishOptions::default(),
          serde_json::to_string(&payload)?.as_bytes(),
          BasicProperties::default(),
        )
        .await?
        .await?,
    )
  }
}
