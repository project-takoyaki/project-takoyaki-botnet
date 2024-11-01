use anyhow::Result;
use bincode::{Decode, Encode};
use crystals_dilithium::dilithium3::{self, SecretKey};
use obfstr::obfbytes;

use crate::config::DILITHIUM_PUBLIC_KEY;

#[derive(Decode, Encode)]
pub struct Payload {
  pub signature: Vec<u8>,
  pub body: Vec<u8>,
}

impl Payload {
  pub fn new(dilithium_private_key: &SecretKey, body: &[u8]) -> Self {
    Self {
      signature: dilithium_private_key.sign(body).to_vec(),
      body: body.to_vec(),
    }
  }

  pub fn verify(&self) -> bool {
    dilithium3::PublicKey::from_bytes(obfbytes!(&DILITHIUM_PUBLIC_KEY)).verify(&self.body, &self.signature)
  }

  pub fn from_slice(slice: &[u8]) -> Result<Self> {
    Ok(bincode::decode_from_slice(slice, bincode::config::standard())?.0)
  }

  pub fn to_vec(&self) -> Result<Vec<u8>> {
    Ok(bincode::encode_to_vec(self, bincode::config::standard())?)
  }
}
