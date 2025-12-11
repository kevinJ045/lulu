use super::bytes::LuluByteArray;
use crate::ops::std::create_std_module;
use mlua::prelude::*;

use aes_gcm::{
  Aes256Gcm, Key, Nonce,
  aead::{Aead, KeyInit},
};
use rand::RngCore;

pub fn into_module() {
  create_std_module("crypto")
    .add_function("random_key", |_, size: Option<usize>| {
      let size = size.unwrap_or(32);
      if size != 32 {
        return Err(LuaError::external("AES-256 key must be 32 bytes"));
      }

      let mut key_bytes = [0u8; 32];
      rand::thread_rng().fill_bytes(&mut key_bytes);
      Ok(LuluByteArray {
        bytes: key_bytes.to_vec(),
      })
    })
    .add_function("random_nonce", |_, size: Option<usize>| {
      let size = size.unwrap_or(12);
      if size != 12 {
        return Err(LuaError::external("AES-GCM nonce must be 12 bytes"));
      }

      let mut nonce_bytes = [0u8; 12];
      rand::thread_rng().fill_bytes(&mut nonce_bytes);
      Ok(LuluByteArray {
        bytes: nonce_bytes.to_vec(),
      })
    })
    .add_function(
      "encrypt",
      |_, (key_ud, nonce_ud, data_ud): (LuaAnyUserData, LuaAnyUserData, LuaAnyUserData)| {
        let key_bytes = key_ud.borrow::<LuluByteArray>()?;
        if key_bytes.bytes.len() != 32 {
          return Err(LuaError::external("AES-256 key must be 32 bytes"));
        }
        let nonce_bytes = nonce_ud.borrow::<LuluByteArray>()?;
        if nonce_bytes.bytes.len() != 12 {
          return Err(LuaError::external("AES-GCM nonce must be 12 bytes"));
        }
        let data_bytes = data_ud.borrow::<LuluByteArray>()?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes.bytes);
        let nonce = Nonce::from_slice(&nonce_bytes.bytes);

        let cipher = Aes256Gcm::new(key);
        let encrypted = cipher
          .encrypt(nonce, data_bytes.bytes.as_slice())
          .map_err(|e| LuaError::external(e.to_string()))?;

        Ok(LuluByteArray { bytes: encrypted })
      },
    )
    .add_function(
      "decrypt",
      |_, (key_ud, nonce_ud, ct_ud): (LuaAnyUserData, LuaAnyUserData, LuaAnyUserData)| {
        let key_bytes = key_ud.borrow::<LuluByteArray>()?;
        if key_bytes.bytes.len() != 32 {
          return Err(LuaError::external("AES-256 key must be 32 bytes"));
        }
        let nonce_bytes = nonce_ud.borrow::<LuluByteArray>()?;
        if nonce_bytes.bytes.len() != 12 {
          return Err(LuaError::external("AES-GCM nonce must be 12 bytes"));
        }
        let ct_bytes = ct_ud.borrow::<LuluByteArray>()?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes.bytes);
        let nonce = Nonce::from_slice(&nonce_bytes.bytes);

        let cipher = Aes256Gcm::new(key);
        let decrypted = cipher
          .decrypt(nonce, ct_bytes.bytes.as_slice())
          .map_err(|e| LuaError::external(e.to_string()))?;

        Ok(LuluByteArray { bytes: decrypted })
      },
    )
    .into();
}
