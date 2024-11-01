use std::path::PathBuf;

use aes_gcm::{
  aead::{Aead, OsRng},
  AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
};
use anyhow::{anyhow, Result};
use bincode::{de::Decoder, enc::Encoder, Decode, Encode};
use libp2p::{identity::Keypair, Multiaddr};
use log::{error, info, warn};
use obfstr::{obfbytes, obfstr, obfstring};
use tokio::{fs, net::TcpListener};

use crate::config::{NETWORK_NAME, STORAGE_ENCRYPTION_KEY};

pub struct Storage {
  pub keypair: Keypair,
  pub known_addresses: Vec<Multiaddr>,
  pub listen_port: u16,
}

impl Encode for Storage {
  fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
    let keypair_bytes = self.keypair.to_protobuf_encoding().map_err(|_| bincode::error::EncodeError::OtherString(obfstring!("Failed to encode keypair")))?;
    keypair_bytes.encode(encoder)?;

    let address_strings: Vec<String> = self.known_addresses.iter().map(|addr| addr.to_string()).collect();
    address_strings.encode(encoder)?;

    self.listen_port.encode(encoder)?;

    Ok(())
  }
}

impl Decode for Storage {
  fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
    let keypair_bytes: Vec<u8> = Vec::<u8>::decode(decoder)?;
    let keypair = Keypair::from_protobuf_encoding(&keypair_bytes).map_err(|_| bincode::error::DecodeError::OtherString(obfstring!("Invalid keypair")))?;

    let address_strings: Vec<String> = Vec::<String>::decode(decoder)?;
    let known_addresses: Vec<Multiaddr> = address_strings
      .iter()
      .map(|string| string.parse().map_err(|_| bincode::error::DecodeError::OtherString(obfstring!("Invalid multiaddress"))))
      .collect::<Result<_, _>>()?;

    let listen_port = u16::decode(decoder)?;

    Ok(Self { keypair, known_addresses, listen_port })
  }
}

impl Storage {
  pub async fn load() -> Result<Self> {
    let storage_path = Self::get_storage_path()?;

    let encrypted_data = match fs::read(storage_path.as_path()).await {
      Ok(data) => data,
      Err(_) => {
        warn!("Failed to read storage from disk at {storage_path:?}.");
        return Self::new().await;
      }
    };

    let decrypted_data = match Self::decrypt(&encrypted_data) {
      Ok(data) => data,
      Err(_) => {
        error!("Failed to decrypt storage from disk at {storage_path:?}.");
        return Self::new().await;
      }
    };

    match bincode::decode_from_slice(&decrypted_data, bincode::config::standard()) {
      Ok((storage, _)) => {
        info!("Storage loaded from disk at {storage_path:?}.");
        return Ok(storage);
      }
      Err(_) => {
        error!("Failed to decode decrypted storage from disk at {storage_path:?}.");
        return Self::new().await;
      }
    }
  }

  pub async fn save(&self) -> Result<()> {
    let storage_path = Self::get_storage_path()?;
    let encoded_data = bincode::encode_to_vec(self, bincode::config::standard())?;
    let encrypted_data = Self::encrypt(&encoded_data)?;

    match fs::write(storage_path.as_path(), encrypted_data).await {
      Ok(_) => info!("Storage saved to disk at {storage_path:?}."),
      Err(_) => error!("Failed to save storage to disk at {storage_path:?}."),
    }

    Ok(())
  }

  async fn new() -> Result<Self> {
    info!("Initializing new storage with default configuration.");

    let storage = Self::default().await?;
    storage.save().await?;

    Ok(storage)
  }

  async fn default() -> Result<Self> {
    let tcp_listener = TcpListener::bind("0.0.0.0:0").await?;
    let listen_port = tcp_listener.local_addr()?.port();

    Ok(Self {
      keypair: Keypair::generate_ed25519(),
      listen_port,
      known_addresses: Vec::new(),
    })
  }

  fn get_storage_path() -> Result<PathBuf> {
    let mut executable_path = std::env::current_exe()?;
    executable_path.pop();

    Ok(executable_path.join(format!("{}{}", obfstr!(NETWORK_NAME), obfstr!(".bin"))))
  }

  fn get_storage_cipher() -> Aes256Gcm {
    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(obfbytes!(&STORAGE_ENCRYPTION_KEY)))
  }

  fn decrypt(slice: &[u8]) -> Result<Vec<u8>> {
    let nonce = Nonce::from_slice(&slice[..12]);
    let ciphertext = &slice[12..];
    let cipher = Self::get_storage_cipher();
    let decrypted_data = cipher.decrypt(nonce, ciphertext).map_err(|_| anyhow!("{}", obfstr!("Decryption failed")))?;

    Ok(decrypted_data)
  }

  fn encrypt(slice: &[u8]) -> Result<Vec<u8>> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher = Self::get_storage_cipher();
    let ciphertext = cipher.encrypt(&nonce, slice).map_err(|_| anyhow!("{}", obfstr!("Encryption failed")))?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
  }
}
