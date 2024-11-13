use std::{env, fs, path::Path};

fn main() {
  let build_profile = env::var("PROFILE").unwrap();
  let output_directory = env::var("OUT_DIR").unwrap();
  let mut build_directory = Path::new(&output_directory);

  while let Some(parent) = build_directory.parent() {
    if parent.file_name().and_then(|name| name.to_str()).map_or(false, |name| name == build_profile) {
      build_directory = parent;
      break;
    }

    build_directory = parent;
  }

  let assets_directory = env::current_dir().unwrap().join("assets");
  let network_name = fs::read_to_string(assets_directory.join("network-name")).unwrap().trim().to_string();

  fs::copy(assets_directory.join("dilithium-private-key"), build_directory.join(format!("{network_name}.key"))).unwrap();

  println!("cargo:rerun-if-changed=assets/network-name");
  println!("cargo:rerun-if-changed=assets/dilithium-private-key");
}
