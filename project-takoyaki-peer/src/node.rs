use crate::config::{
  GOSSIPSUB_TOPIC, IDENTIFY_PROTOCOL_VERSION, KADEMLIA_PROTOCOL_VERSION, SEED_NODES,
  SWARM_PRESHARED_KEY,
};

use futures::StreamExt;
use libp2p::{
  autonat,
  core::upgrade::Version,
  gossipsub, identify, identity, kad, noise,
  pnet::{PnetConfig, PreSharedKey},
  swarm::{NetworkBehaviour, SwarmEvent},
  tcp, upnp, yamux, StreamProtocol, Transport,
};
use log::{error, info, warn};
use rand::rngs::OsRng;
use std::{error::Error, time::Duration};
use tokio::{
  io::{self, AsyncBufReadExt},
  select,
};

pub async fn init(keypair: identity::Keypair, listen_port: u16) -> Result<(), Box<dyn Error>> {
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
      let mut kad_config = kad::Config::new(StreamProtocol::new(KADEMLIA_PROTOCOL_VERSION));
      let memory_store = kad::store::MemoryStore::new(keypair.public().to_peer_id());

      kad_config.set_query_timeout(Duration::from_secs(300));

      Ok(Behaviour {
        autonat_client: autonat::v2::client::Behaviour::new(
          OsRng,
          autonat::v2::client::Config::default().with_probe_interval(Duration::from_secs(2)),
        ),
        autonat_server: autonat::v2::server::Behaviour::new(OsRng),
        gossipsub: gossipsub::Behaviour::new(
          gossipsub::MessageAuthenticity::Signed(keypair.clone()),
          gossipsub_config,
        )?,
        identify: identify::Behaviour::new(identify::Config::new(
          IDENTIFY_PROTOCOL_VERSION.into(),
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
    .with_swarm_config(|swarm_config| {
      swarm_config.with_idle_connection_timeout(Duration::from_secs(60))
    })
    .build();

  /* TODO: add a backup list of known peers */
  /* add seed nodes for bootstrapping */
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
  swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{listen_port}").parse()?)?;

  let mut stdin = io::BufReader::new(io::stdin()).lines();

  loop {
    select! {
      Ok(Some(line)) = stdin.next_line() => {
        if let Err(error) = swarm
          .behaviour_mut()
          .gossipsub
          .publish(gossipsub_topic.clone(), line.as_bytes())
        {
          error!("Published failed with error '{error:?}'")
        }
      }

      event = swarm.select_next_some() => {
        handle_event(event, &mut swarm).await;
      }
    }
  }
}

async fn handle_event(event: SwarmEvent<BehaviourEvent>, swarm: &mut libp2p::Swarm<Behaviour>) {
  match event {
    SwarmEvent::NewListenAddr { address, .. } => {
      info!("Listening for incoming peer connections on address '{address}'.");
    }

    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
      propagation_source,
      message_id,
      message,
    })) => {
      println!(
        "Got message: '{}' with id: {message_id} from peer: {propagation_source}",
        String::from_utf8_lossy(&message.data),
      );
    }

    SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received { info, .. })) => {
      for address in info.listen_addrs {
        swarm
          .behaviour_mut()
          .kademlia
          .add_address(&info.public_key.to_peer_id(), address);
      }

      swarm
        .behaviour_mut()
        .gossipsub
        .add_explicit_peer(&info.public_key.to_peer_id());
    }

    SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::NewExternalAddr(address))) => {
      info!("Mapped external address '{address}' through UPnP.");
    }

    SwarmEvent::ExternalAddrConfirmed { address } => {
      info!("External address '{address}' confirmed.");

      swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));
    }

    SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::GatewayNotFound)) => {
      warn!("UPnP-capable gateway not found.");
    }

    SwarmEvent::Behaviour(BehaviourEvent::Upnp(upnp::Event::NonRoutableGateway)) => {
      warn!("Non-routable gateway detected.");
    }

    _ => {}
  }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
  autonat_client: autonat::v2::client::Behaviour,
  autonat_server: autonat::v2::server::Behaviour,
  gossipsub: gossipsub::Behaviour,
  identify: identify::Behaviour,
  kademlia: kad::Behaviour<kad::store::MemoryStore>,
  upnp: upnp::tokio::Behaviour,
}
