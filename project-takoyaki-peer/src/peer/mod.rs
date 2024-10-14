mod behaviour;
mod events;
mod swarm;

use std::{error::Error, net::Ipv4Addr};

use futures::StreamExt;
use libp2p::{gossipsub, identity::Keypair, multiaddr::Protocol, Multiaddr, PeerId, Swarm};

use crate::config::NETWORK_NAME;

use self::{behaviour::Behaviour, events::handle_event, swarm::build_swarm};

pub struct Peer {
  swarm: Swarm<Behaviour>,
  listen_port: u16,
  known_peers: Vec<Multiaddr>,
}

impl Peer {
  pub async fn new(keypair: Keypair, listen_port: u16, known_peers: Vec<Multiaddr>) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      swarm: build_swarm(keypair).await?,
      listen_port,
      known_peers,
    })
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.swarm.listen_on(
      Multiaddr::empty()
        .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Tcp(self.listen_port)),
    )?;

    for address in &self.known_peers {
      if let Some(peer_id) = address.iter().find_map(|protocol| {
        if let Protocol::P2p(peer_id) = protocol {
          PeerId::from_multihash(peer_id.as_ref().clone()).ok()
        } else {
          None
        }
      }) {
        self
          .swarm
          .behaviour_mut()
          .kademlia
          .add_address(&peer_id, address.clone());
      }
    }

    self
      .swarm
      .behaviour_mut()
      .gossipsub
      .subscribe(&gossipsub::IdentTopic::new(NETWORK_NAME))?;

    loop {
      let event = self.swarm.select_next_some().await;

      handle_event(&mut self.swarm, event).await;
    }
  }
}
