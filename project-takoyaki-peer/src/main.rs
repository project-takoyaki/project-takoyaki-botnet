mod config;
mod payload;
mod peer;
mod storage;

use std::env;

use aes_gcm::aead::OsRng;
use anyhow::Result;
use crystals_dilithium::dilithium3::{SecretKey, SECRETKEYBYTES};
use libp2p::Multiaddr;
use log::{error, info, warn};
use obfstr::obfstr;
use rand::RngCore;
use tokio::fs;

use crate::{config::NETWORK_NAME, payload::Payload, peer::Peer, storage::Storage};

#[tokio::main]
async fn main() -> Result<()> {
  pretty_env_logger::formatted_builder().filter_level(log::LevelFilter::Info).init();

  warn!("Logging is currently enabled. Enable the 'max_level_off' feature flag for the 'log' crate to disable logging.");

  let storage = Storage::load().await?;
  let mut peer = match Peer::new(storage) {
    Ok(peer) => peer,
    Err(error) => {
      error!("Failed to initialize peer: {error:?}");
      return Err(error);
    }
  };

  let bootstrap_address = env::var(obfstr!("BOOTSTRAP_ADDRESS")).ok().and_then(|address| match address.parse::<Multiaddr>() {
    Ok(address) => {
      info!("Parsed bootstrap address {address}.");
      Some(address)
    }
    Err(_) => {
      warn!("Failed to parse bootstrap address {address}.");
      None
    }
  });

  let dilithium_private_key = load_dilithium_private_key().await;
  if let Err(error) = peer.run(bootstrap_address, dilithium_private_key).await {
    error!("Peer encountered an error during execution: {error:?}");
    return Err(error);
  }

  Ok(())
}

async fn load_dilithium_private_key() -> Option<SecretKey> {
  let executable_path = std::env::current_exe().ok()?;
  let executable_directory = executable_path.parent()?;

  let dilithium_private_key_path = executable_directory.join(format!("{}.key", obfstr!(NETWORK_NAME)));
  let dilithium_private_key_bytes = match fs::read(&dilithium_private_key_path).await {
    Ok(bytes) => bytes,
    Err(_) => return None,
  };

  info!("Loaded Dilithium private key from disk at {}.", dilithium_private_key_path.display());

  let key_bytes: [u8; SECRETKEYBYTES] = match dilithium_private_key_bytes.try_into() {
    Ok(bytes) => bytes,
    Err(_) => {
      error!("Invalid Dilithium private key: expected {SECRETKEYBYTES} bytes.");
      return None;
    }
  };

  let secret_key = SecretKey { bytes: key_bytes };
  let mut body = [0u8; 32];
  OsRng.fill_bytes(&mut body);

  let payload = Payload::new(&secret_key, &body.to_vec());
  if payload.verify() {
    info!("Authenticated with Dilithium private key on {NETWORK_NAME:?}.");
    Some(secret_key)
  } else {
    error!("Failed to verify Dilithium private key: public key mismatch or corrupted key data.");
    None
  }
}
