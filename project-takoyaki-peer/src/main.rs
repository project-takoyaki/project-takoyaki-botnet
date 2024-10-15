mod config;
mod peer;
mod vault;

use std::error::Error;

use clap::Parser;
use libp2p::{identity::Keypair, Multiaddr};
use log::{info, warn};

use crate::{peer::Peer, vault::Vault};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  pretty_env_logger::formatted_builder()
    .filter_level(log::LevelFilter::Info)
    .init();

  warn!(
    "Logging is currently enabled. Enable the 'max_level_off' feature flag for the 'log' crate to disable logging."
  );

  let args = Args::parse();

  let vault = Vault::load().await?;

  let keypair = Keypair::from_protobuf_encoding(&vault.keypair)?;
  let listen_port = vault.listen_port;

  let mut known_peers = Vec::<Multiaddr>::new();

  for peer in &vault.known_peers {
    known_peers.push(peer.parse()?);
  }

  if let Some(address) = args.bootstrap_address {
    known_peers.push(address);
  }

  info!(
    "Peer::new({}, {}, {:?})",
    keypair.public().to_peer_id(),
    listen_port,
    known_peers
  );

  Peer::new(keypair, listen_port, known_peers).await?.run().await?;

  Ok(())
}

#[derive(Parser)]
#[clap(disable_help_flag = true, ignore_errors = true)]
struct Args {
  #[clap(long)]
  bootstrap_address: Option<Multiaddr>,
}
