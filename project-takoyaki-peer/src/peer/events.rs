use libp2p::{identify, kad, swarm::SwarmEvent, Swarm};
use log::info;

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
