use std::{error::Error, net::TcpListener};

use aes_gcm::{
  aead::{Aead, OsRng},
  AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
};
use libp2p::identity::Keypair;
use obfstr::{obfbytes, obfstr};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::OnceCell};

use crate::config::{NETWORK_NAME, VAULT_ENCRYPTION_KEY};

static VAULT_CACHE: OnceCell<Vault> = OnceCell::const_new();

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Vault {
  pub keypair: Vec<u8>,
  pub listen_port: u16,
  pub known_peers: Vec<String>,
}

impl Vault {
  pub async fn load() -> Result<&'static Self, Box<dyn Error>> {
    VAULT_CACHE
      .get_or_try_init(|| async {
        let mut executable_path = std::env::current_exe()?;
        executable_path.pop();

        let vault_path = executable_path.join(format!("{}{}", obfstr!(NETWORK_NAME), obfstr!(".vault")));

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(obfbytes!(&VAULT_ENCRYPTION_KEY)));

        if let Ok(encrypted_data) = fs::read(&vault_path).await {
          let nonce = Nonce::from_slice(&encrypted_data[..12]);
          let ciphertext = &encrypted_data[12..];
          let decrypted_data = cipher.decrypt(nonce, ciphertext).unwrap();

          let vault: Vault = serde_json::from_slice(&decrypted_data)?;
          Ok(vault)
        } else {
          let listener = TcpListener::bind("0.0.0.0:0")?;
          let port = listener.local_addr()?.port();

          drop(listener);

          let vault = Vault {
            keypair: Keypair::generate_ed25519().to_protobuf_encoding()?,
            listen_port: port,
            known_peers: Vec::new(),
          };

          vault.save().await?;

          Ok(vault)
        }
      })
      .await
      .map(|vault| vault)
  }

  pub async fn save(&self) -> Result<(), Box<dyn Error>> {
    if let Some(cached_vault) = VAULT_CACHE.get() {
      if self == cached_vault {
        return Ok(());
      }
    }

    let mut executable_path = std::env::current_exe()?;
    executable_path.pop();

    let vault_path = executable_path.join(format!("{}{}", obfstr!(NETWORK_NAME), obfstr!(".vault")));

    let serialized_data = serde_json::to_vec(self)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(obfbytes!(&VAULT_ENCRYPTION_KEY)));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, serialized_data.as_ref()).unwrap();

    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);

    fs::write(vault_path, encrypted_data).await?;

    Ok(())
  }
}
