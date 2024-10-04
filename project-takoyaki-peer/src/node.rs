use crate::config::{
  GOSSIPSUB_TOPIC, IDENTIFY_PROTOCOL_NAME, KADEMLIA_PROTOCOL_NAME, SEED_NODES, SWARM_PRESHARED_KEY,
};

use futures::StreamExt;
use libp2p::{
  core::upgrade::Version,
  gossipsub, identify, identity, kad, noise,
  pnet::{PnetConfig, PreSharedKey},
  swarm::{NetworkBehaviour, SwarmEvent},
  tcp, upnp, yamux, StreamProtocol, Transport,
};
use log::{info, warn};
use std::{error::Error, process::exit, time::Duration};

pub async fn init(keypair: identity::Keypair, port: u16) -> Result<(), Box<dyn Error>> {
  /* build the swarm */
  let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
    .with_tokio()
    .with_other_transport(|keypair| {
      let swarm_key = PreSharedKey::new(SWARM_PRESHARED_KEY);

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
    .with_behaviour(|keypair| {
      /* GossipSub */
      let gossipsub_config = gossipsub::ConfigBuilder::default()
        .max_transmit_size(262144)
        .build()?;

      /* Kademlia */
      let mut kad_config = kad::Config::new(StreamProtocol::new(KADEMLIA_PROTOCOL_NAME));
      let memory_store = kad::store::MemoryStore::new(keypair.public().to_peer_id());

      kad_config.set_query_timeout(Duration::from_secs(300));

      Ok(Behaviour {
        gossipsub: gossipsub::Behaviour::new(
          gossipsub::MessageAuthenticity::Signed(keypair.clone()),
          gossipsub_config,
        )?,
        identify: identify::Behaviour::new(identify::Config::new(
          IDENTIFY_PROTOCOL_NAME.into(),
          keypair.public(),
        )),
        kademlia: kad::Behaviour::with_config(
          keypair.public().to_peer_id(),
          memory_store,
          kad_config,
        ),
        upnp: upnp::tokio::Behaviour::default(),
      })
    })?
    .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
    .build();

  /* add seed nodes for bootstrapping */
  /* TODO: add a backup list of known peers */
  for address in &SEED_NODES {
    info!("Adding seed node '{address}' to the routing table.");

    swarm.behaviour_mut().kademlia.add_address(
      &address.split("/").last().unwrap().parse()?,
      address.parse()?,
    );
  }

  /* subscribe to gossipsub topic */
  info!("Subscribing to '{GOSSIPSUB_TOPIC}'");

  let gossipsub_topic = gossipsub::IdentTopic::new(GOSSIPSUB_TOPIC);

  swarm
    .behaviour_mut()
    .gossipsub
    .subscribe(&gossipsub_topic)?;

  /* attempt to listen on an adress */
  swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse()?)?;

  /* TODO: seperate this event loop */
  loop {
    match swarm.select_next_some().await {
      SwarmEvent::NewListenAddr { address, .. } => {
        info!("Listening for incoming peer connections on address '{address}'.");
      }

      SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent {
        peer_id, ..
      })) => {
        println!("Sent identify info to {peer_id:?}")
      }

      SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
        info, ..
      })) => {
        println!("Received {info:?}")
      }

      SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::NewExternalAddr(address))) => {
        info!("Mapped external address '{address}' through UPnP. Switching to server mode.");

        swarm
          .behaviour_mut()
          .kademlia
          .set_mode(Some(kad::Mode::Server));
      }

      SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::GatewayNotFound)) => {
        warn!("UPnP-capable gateway not found. Attempting libp2p hole punching.");
        exit(1);
        /* TODO: hole punching fallback */
      }

      SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::NonRoutableGateway)) => {
        warn!("Non-routable gateway detected. Attempting libp2p hole punching.");
        exit(1);
        /* TODO: hole punching fallback */
      }

      _ => {}
    }
  }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
  gossipsub: gossipsub::Behaviour,
  identify: identify::Behaviour,
  kademlia: kad::Behaviour<kad::store::MemoryStore>,
  upnp: upnp::tokio::Behaviour,
}
