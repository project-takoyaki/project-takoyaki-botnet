use std::{fs::File, io::Write};

use aes_gcm::aead::rand_core::{OsRng, RngCore};
use anyhow::Result;
use crystals_dilithium::dilithium3;
use log::info;

fn main() -> Result<()> {
  pretty_env_logger::formatted_builder().filter_level(log::LevelFilter::Info).init();

  let mut swarm_preshared_key = [0u8; 32];
  OsRng.fill_bytes(&mut swarm_preshared_key);

  let mut storage_encryption_key = [0u8; 32];
  OsRng.fill_bytes(&mut storage_encryption_key);

  let dilithium_keypair = dilithium3::Keypair::generate(None);
  let dilithium_public_key = dilithium_keypair.public.to_bytes();
  let dilithium_private_key = dilithium_keypair.secret.to_bytes();

  let config = format!(
    "{}\n{}\n{}\npub const NETWORK_NAME: &str = \"project-takoyaki\";\n",
    format_byte_array("SWARM_PRESHARED_KEY", &swarm_preshared_key),
    format_byte_array("STORAGE_ENCRYPTION_KEY", &storage_encryption_key),
    format_byte_array("DILITHIUM_PUBLIC_KEY", &dilithium_public_key)
  );

  let mut executable_path = std::env::current_exe()?;
  executable_path.pop();

  let config_path = executable_path.join("config.rs");
  let mut config_file = File::create(config_path.clone())?;
  config_file.write_all(config.as_bytes())?;

  info!("Saved config to disk at {config_path:?}.");

  let private_key_path = executable_path.join("project-takoyaki.key");
  let mut key_file = File::create(private_key_path.clone())?;
  key_file.write_all(&dilithium_private_key)?;

  info!("Saved Dilithium private key to disk at {private_key_path:?}.");

  Ok(())
}

fn format_byte_array(name: &str, source: &[u8]) -> String {
  let bytes: String = source.iter().map(|byte| format!(" 0x{:02x},", byte)).collect();
  format!("pub const {}: [u8; {}] = [\n {}\n];\n", name, source.len(), bytes)
}
