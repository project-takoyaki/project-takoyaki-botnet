mod config;
mod peer;

use std::error::Error;

use libp2p::identity::Keypair;
use log::warn;

use crate::peer::Peer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  pretty_env_logger::formatted_builder()
    .filter_level(log::LevelFilter::Info)
    .init();

  warn!(
    "Logging is currently enabled. Enable the 'max_level_off' feature flag for the 'log' crate to disable logging."
  );

  Peer::new(Keypair::generate_ed25519(), 0, Vec::new()).await?.run().await
}
