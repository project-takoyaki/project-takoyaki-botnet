use std::time::Duration;

use anyhow::Result;
use libp2p::{
  autonat,
  core::upgrade::Version,
  gossipsub, identify,
  identity::Keypair,
  kad, noise,
  pnet::{PnetConfig, PreSharedKey},
  tcp, upnp, yamux, Swarm, Transport,
};
use obfstr::{obfbytes, obfstr};
use rand::rngs::OsRng;

use crate::config::{NETWORK_NAME, SWARM_PRESHARED_KEY};

use super::behavior::Behavior;

pub fn build_swarm(keypair: &Keypair) -> Result<Swarm<Behavior>> {
  let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
    .with_tokio()
    .with_other_transport(|keypair| {
      let swarm_key = PreSharedKey::new(*obfbytes!(&SWARM_PRESHARED_KEY));
      let noise_config = noise::Config::new(keypair)?;
      let yamux_config = yamux::Config::default();
      let base_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true)).and_then(move |socket, _| PnetConfig::new(swarm_key).handshake(socket));
      let secure_transport = base_transport.upgrade(Version::V1Lazy).authenticate(noise_config).multiplex(yamux_config);

      Ok(secure_transport)
    })?
    .with_dns()?
    .with_behaviour(|keypair| {
      Ok(Behavior {
        autonat_client: autonat::v2::client::Behaviour::new(OsRng, autonat::v2::client::Config::default().with_probe_interval(Duration::from_secs(2))),
        autonat_server: autonat::v2::server::Behaviour::new(OsRng),
        gossipsub: gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(keypair.clone()), gossipsub::ConfigBuilder::default().max_transmit_size(2097152).build()?)?,
        identify: identify::Behaviour::new(identify::Config::new(format!("{}{}", obfstr!(NETWORK_NAME), obfstr!("/1.0.0")).into(), keypair.public())),
        kademlia: kad::Behaviour::new(keypair.public().to_peer_id(), kad::store::MemoryStore::new(keypair.public().to_peer_id())),
        upnp: upnp::tokio::Behaviour::default(),
      })
    })?
    .with_swarm_config(|swarm_config| swarm_config.with_idle_connection_timeout(Duration::from_secs(60)))
    .build();

  Ok(swarm)
}
