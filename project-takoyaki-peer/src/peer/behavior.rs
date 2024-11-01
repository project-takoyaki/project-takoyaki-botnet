use libp2p::{autonat, gossipsub, identify, kad, swarm::NetworkBehaviour, upnp};

#[derive(NetworkBehaviour)]
pub struct Behavior {
  pub autonat_client: autonat::v2::client::Behaviour,
  pub autonat_server: autonat::v2::server::Behaviour,
  pub gossipsub: gossipsub::Behaviour,
  pub identify: identify::Behaviour,
  pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
  pub upnp: upnp::tokio::Behaviour,
}
