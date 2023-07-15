use super::*;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  Serve(Server),
}
