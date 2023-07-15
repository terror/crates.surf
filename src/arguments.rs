use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[clap(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) async fn run(self) -> Result {
    match self.subcommand {
      Subcommand::Serve(server) => server.run().await,
    }
  }
}
