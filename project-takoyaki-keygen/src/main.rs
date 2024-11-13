use std::{
  fs,
  path::{Path, PathBuf},
};

use aes_gcm::aead::{rand_core::RngCore, OsRng};
use anyhow::Result;
use cliclack::{clear_screen, confirm, input, intro, log, outro};
use console::style;
use crystals_dilithium::dilithium3;

fn main() -> Result<()> {
  clear_screen()?;

  intro(style("project-takoyaki-keygen").cyan())?;

  let output_directory: String = input("Where would you like to put the generated files?")
    .default_input("./")
    .validate_interactively(|input: &String| {
      let path = Path::new(input);
      if !path.exists() || !path.is_dir() {
        Err("Invalid directory")
      } else {
        Ok(())
      }
    })
    .interact()?;
  let network_name: String = input(format!("What {} would you like configured?", style("network-name").blue())).default_input("project-takoyaki").interact()?;
  let generate_dilithium_keypair = confirm(format!("Would you like to generate {} and {}?", style("dilithium-private-key").blue(), style("dilithium-public-key").blue()))
    .initial_value(true)
    .interact()?;
  let generate_storage_encryption_key = confirm(format!("Would you like to generate {}?", style("storage-encryption-key").blue())).initial_value(true).interact()?;
  let generate_swarm_preshared_key = confirm(format!("Would you like to generate {}?", style("swarm-preshared-key").blue())).initial_value(true).interact()?;

  log::info(format!("Saving network configuration files for {}.", style(&network_name).green()))?;

  let output_path = PathBuf::from(output_directory).canonicalize()?;

  let spinner = cliclack::spinner();
  spinner.start(format!("Saving {}.", style("network-name").blue()));
  fs::write(output_path.join("network-name"), network_name.as_bytes())?;
  spinner.stop(format!("Saved {}.", style("network-name").blue()));

  if generate_dilithium_keypair {
    let spinner = cliclack::spinner();
    spinner.start(format!("Generating {} and {}.", style("dilithium-public-key").blue(), style("dilithium-private-key").blue()));

    let dilithium_keypair = dilithium3::Keypair::generate(None);
    fs::write(output_path.join("dilithium-public-key"), dilithium_keypair.public.to_bytes())?;
    fs::write(output_path.join("dilithium-private-key"), dilithium_keypair.secret.to_bytes())?;

    spinner.stop(format!("Saved {} and {}.", style("dilithium-public-key").blue(), style("dilithium-private-key").blue()));
  }

  if generate_storage_encryption_key {
    let spinner = cliclack::spinner();
    spinner.start(format!("Generating {}.", style("storage-encryption-key").blue()));

    let mut storage_encryption_key = [0u8; 32];
    OsRng.fill_bytes(&mut storage_encryption_key);
    fs::write(output_path.join("storage-encryption-key"), &storage_encryption_key)?;

    spinner.stop(format!("Saved {}.", style("storage-encryption-key").blue()));
  }

  if generate_swarm_preshared_key {
    let spinner = cliclack::spinner();
    spinner.start(format!("Generating {}.", style("swarm-preshared-key").blue()));

    let mut swarm_preshared_key = [0u8; 32];
    OsRng.fill_bytes(&mut swarm_preshared_key);
    fs::write(output_path.join("swarm-preshared-key"), &swarm_preshared_key)?;

    spinner.stop(format!("Saved {}.", style("swarm-preshared-key").blue()));
  }

  let display_path_lossy = output_path.to_string_lossy();
  let display_path = if display_path_lossy.starts_with(r"\\?\") { &display_path_lossy[r"\\?\".len()..] } else { &display_path_lossy }.to_string();

  outro(format!("Saved network configuration files in {}.", style(display_path).green()))?;

  Ok(())
}
