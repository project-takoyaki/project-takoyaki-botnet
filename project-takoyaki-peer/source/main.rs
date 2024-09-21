fn main() {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  log::warn!("Logging is enabled. Enable the 'max_level_off' feature flag to disable logging.");
}
