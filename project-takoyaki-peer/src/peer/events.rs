use libp2p::{identify, kad, swarm::SwarmEvent, Swarm};
use log::info;

use crate::vault::Vault;

use super::behaviour::{Behaviour, BehaviourEvent};

pub async fn handle_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) {
  info!("{event:?}");

  match event {
    /* built-in swarm events */
    SwarmEvent::ExternalAddrConfirmed { .. } => {
      swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));
    }

    SwarmEvent::ExternalAddrExpired { .. } => {
      swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Client));
    }

    SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
      let vault = Vault::load().await.unwrap();

      let mut known_peers = vault.known_peers.clone();

      let address = address.with_p2p(peer_id).unwrap().to_string();

      if !known_peers.contains(&address) {
        known_peers.push(address);

        Vault {
          keypair: vault.keypair.clone(),
          listen_port: vault.listen_port.clone(),
          known_peers,
        }
        .save()
        .await
        .unwrap();
      }
    }

    SwarmEvent::OutgoingConnectionError { peer_id, .. } => {
      if let Some(peer_id) = peer_id {
        let vault = Vault::load().await.unwrap();

        let mut known_peers = vault.known_peers.clone();

        let error_addresses: Vec<String> = known_peers
          .iter()
          .filter(|addr| addr.contains(&peer_id.to_string()))
          .cloned()
          .collect();

        if !error_addresses.is_empty() {
          for address in error_addresses {
            known_peers.retain(|peer| peer != &address);
          }

          Vault {
            keypair: vault.keypair.clone(),
            listen_port: vault.listen_port.clone(),
            known_peers,
          }
          .save()
          .await
          .unwrap();
        }
      }
    }

    /* identify events */
    SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received { info, .. })) => {
      swarm
        .behaviour_mut()
        .gossipsub
        .add_explicit_peer(&info.public_key.to_peer_id());
    }

    _ => {}
  }
}
