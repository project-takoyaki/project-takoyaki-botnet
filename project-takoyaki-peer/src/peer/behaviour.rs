use std::{error::Error, time::Duration};

use libp2p::{autonat, gossipsub, identify, identity::Keypair, kad, swarm::NetworkBehaviour, upnp};
use obfstr::obfstr;
use rand::rngs::OsRng;

use crate::config::NETWORK_NAME;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
  pub autonat_client: autonat::v2::client::Behaviour,
  pub autonat_server: autonat::v2::server::Behaviour,
  pub gossipsub: gossipsub::Behaviour,
  pub identify: identify::Behaviour,
  pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
  pub upnp: upnp::tokio::Behaviour,
}

impl Behaviour {
  pub fn new(keypair: Keypair) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      autonat_client: autonat::v2::client::Behaviour::new(
        OsRng,
        autonat::v2::client::Config::default().with_probe_interval(Duration::from_secs(2)),
      ),
      autonat_server: autonat::v2::server::Behaviour::new(OsRng),
      gossipsub: gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub::ConfigBuilder::default().max_transmit_size(262144).build()?,
      )?,
      identify: identify::Behaviour::new(identify::Config::new(
        format!("{}{}", obfstr!(NETWORK_NAME), "/1.0.0").into(),
        keypair.public(),
      )),
      kademlia: kad::Behaviour::new(
        keypair.public().to_peer_id(),
        kad::store::MemoryStore::new(keypair.public().to_peer_id()),
      ),
      upnp: upnp::tokio::Behaviour::default(),
    })
  }
}
