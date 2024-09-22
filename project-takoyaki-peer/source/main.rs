mod node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  /* initialize logging as early as possible */
  pretty_env_logger::formatted_builder()
    .filter_level(log::LevelFilter::Info)
    .init();

  log::warn!("Logging is enabled. Enable the 'max_level_off' feature flag to disable logging.");

  node::init().await?;

  Ok(())
}
