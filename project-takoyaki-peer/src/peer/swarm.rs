use std::{error::Error, time::Duration};

use libp2p::{
  core::upgrade::Version,
  identity::Keypair,
  noise,
  pnet::{PnetConfig, PreSharedKey},
  tcp, yamux, Swarm, Transport,
};
use obfstr::obfbytes;

use crate::config::SWARM_PRESHARED_KEY;

use super::behaviour::Behaviour;

pub async fn build_swarm(keypair: Keypair) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
  let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
    .with_tokio()
    .with_other_transport(|keypair| {
      let swarm_key = PreSharedKey::new(*obfbytes!(&SWARM_PRESHARED_KEY));

      let noise_config = noise::Config::new(keypair).unwrap();
      let yamux_config = yamux::Config::default();

      let base_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .and_then(move |socket, _| PnetConfig::new(swarm_key).handshake(socket));

      base_transport
        .upgrade(Version::V1Lazy)
        .authenticate(noise_config)
        .multiplex(yamux_config)
    })?
    .with_dns()?
    .with_behaviour(|keypair| Behaviour::new(keypair.clone()).unwrap())?
    .with_swarm_config(|swarm_config| swarm_config.with_idle_connection_timeout(Duration::from_secs(60)))
    .build();

  Ok(swarm)
}
