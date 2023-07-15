use super::*;

#[derive(Debug, Default)]
pub(crate) struct PublishOptions {
  page: Option<u64>,
  page_size: Option<u64>,
}

pub(crate) struct Publisher {
  channel: Channel,
  client: AsyncClient,
}

impl Publisher {
  const PAGE_SIZE: u64 = 100;
  const USER_AGENT: &str = "crates.surf (admin@crates.surf)";

  pub(crate) fn new(channel: Channel) -> Result<Self> {
    Ok(Self {
      channel,
      client: AsyncClient::new(Self::USER_AGENT, Duration::from_millis(1000))?,
    })
  }

  pub(crate) async fn publish(self, options: PublishOptions) -> Result {
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
        self.channel.publish("crates", partial).await?;
      }

      curr_page += 1;

      result = self.client.crates(query.clone()).await?;
    }

    Ok(())
  }
}
