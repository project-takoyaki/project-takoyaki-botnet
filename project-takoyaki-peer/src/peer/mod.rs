mod behavior;
mod cli;
mod commands;
mod events;
mod swarm;

use std::net::Ipv4Addr;

use anyhow::Result;
use crystals_dilithium::dilithium3::SecretKey;
use futures::StreamExt;
use libp2p::{gossipsub::IdentTopic, multiaddr::Protocol, Multiaddr, PeerId, Swarm};
use log::info;
use obfstr::obfstr;
use tokio::{
  io::{self, AsyncBufReadExt},
  select,
};

use crate::{config::NETWORK_NAME, storage::Storage};

use self::{behavior::Behavior, cli::CLI, commands::CommandHandler, events::handle_event, swarm::build_swarm};

pub struct Peer {
  storage: Storage,
  swarm: Swarm<Behavior>,
}

impl Peer {
  pub fn new(storage: Storage) -> Result<Self> {
    let keypair = storage.keypair.clone();
    let swarm = build_swarm(&keypair)?;
    info!("Built new swarm on {:?}.", storage.keypair.public().to_peer_id());

    Ok(Self { storage, swarm })
  }

  pub async fn run(&mut self, bootstrap_address: Option<Multiaddr>, dilithium_private_key: Option<SecretKey>) -> Result<()> {
    if let Some(bootstrap_address) = bootstrap_address {
      self.add_address(bootstrap_address);
    }

    self.storage.known_addresses.clone().iter().for_each(|address| self.add_address(address.clone()));
    self.swarm.listen_on(Multiaddr::empty().with(Protocol::from(Ipv4Addr::UNSPECIFIED)).with(Protocol::Tcp(self.storage.listen_port)))?;

    let mut command_handler = CommandHandler::new();
    command_handler.register_commands()?;

    let gossipsub_topic = IdentTopic::new(obfstr!(NETWORK_NAME));
    self.swarm.behaviour_mut().gossipsub.subscribe(&gossipsub_topic)?;

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut cli = CLI::new(dilithium_private_key, gossipsub_topic.clone());

    loop {
      select! {
        Ok(Some(buffer)) = stdin.next_line() => cli.handle_input(&mut self.swarm, &mut command_handler, &buffer).await?,
        event = self.swarm.select_next_some() => handle_event(&mut self.swarm, &mut self.storage, &mut command_handler, event).await?,
      }
    }
  }

  fn add_address(&mut self, address: Multiaddr) {
    if let Some(Protocol::P2p(peer_id)) = address.iter().last() {
      if let Ok(peer_id) = PeerId::from_multihash(*peer_id.as_ref()) {
        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, address);
      }
    }
  }
}
