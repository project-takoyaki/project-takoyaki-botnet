mod config;
mod node;

use crate::config::{DEFAULT_PORT, KEYPAIR_OVERRIDE_FILENAME};

use clap::{ArgAction, Parser};
use libp2p::identity;
use log::{info, warn, LevelFilter};
use pretty_env_logger::formatted_builder;
use std::{
  error::Error,
  fs::{self, File},
  io::{Read, Write},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  /* initialize logging as early as possible */
  formatted_builder().filter_level(LevelFilter::Info).init();

  warn!("Logging is enabled. Use the 'max_level_off' feature flag to disable logging.");

  let args = Args::parse();

  /* check for a persistent keypair */
  let keypair = if fs::metadata(KEYPAIR_OVERRIDE_FILENAME).is_ok() {
    let mut keypair_file = File::open(KEYPAIR_OVERRIDE_FILENAME)?;
    let mut keypair_bytes = Vec::new();

    keypair_file.read_to_end(&mut keypair_bytes)?;
    identity::Keypair::from_protobuf_encoding(&keypair_bytes)?
  } else {
    let keypair = identity::Keypair::generate_ed25519();

    keypair
  };

  if let Some(true) = args.save_keypair {
    let mut keypair_file = File::create(KEYPAIR_OVERRIDE_FILENAME)?;
    keypair_file.write_all(&keypair.to_protobuf_encoding()?)?;

    info!("Keypair saved to '{KEYPAIR_OVERRIDE_FILENAME}'.");
  }

  let port = args.listen_port.unwrap_or(DEFAULT_PORT);

  info!(
    "Starting node '{}' on port '{port}'.",
    keypair.public().to_peer_id().to_base58(),
  );

  node::init(keypair, port).await
}

#[derive(Parser)]
#[clap(disable_help_flag = true, ignore_errors = true)]
struct Args {
  #[clap(long, action=ArgAction::SetTrue)]
  save_keypair: Option<bool>,

  #[clap(long)]
  listen_port: Option<u16>,
}
