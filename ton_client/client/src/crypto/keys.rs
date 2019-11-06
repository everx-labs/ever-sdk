use std::sync::Mutex;
use tvm::types::UInt256;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use types::{ApiResult, ApiError, hex_decode};
use std::collections::HashMap;
use base64::URL_SAFE;
use hmac::*;
use sha2::Sha512;
use crypto::math::ton_crc16;

pub type Key192 = [u8; 24];
pub type Key256 = [u8; 32];
pub type Key264 = [u8; 33];
pub type Key512 = [u8; 64];

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct KeyPair {
    pub public: String,
    pub secret: String,
}

impl KeyPair {
    pub fn new(public: String, secret: String) -> KeyPair {
        KeyPair { public, secret }
    }

    pub fn decode(&self) -> ApiResult<Keypair> {
        Ok(Keypair {
            public: decode_public_key(&self.public)?,
            secret: decode_secret_key(&self.secret)?,
        })
    }
}

type KeyPairHandle = String;

pub struct KeyStore {
    next_handle: u32,
    keys: HashMap<KeyPairHandle, KeyPair>
}

lazy_static! {
    static ref KEY_STORE: Mutex<KeyStore> = Mutex::new(KeyStore::new());
}

impl KeyStore {
    pub fn new() -> KeyStore {
        KeyStore {
            next_handle: 1,
            keys: HashMap::new(),
        }
    }

    pub fn add(keys: &KeyPair) -> KeyPairHandle {
        let mut store = KEY_STORE.lock().unwrap();
        let handle: String = format!("{:x}", store.next_handle);
        store.next_handle += 1;
        store.keys.insert(handle.clone(), (*keys).clone());
        handle
    }

    pub fn get(handle: &KeyPairHandle) -> Option<KeyPair> {
        let store = KEY_STORE.lock().unwrap();
        store.keys.get(handle).map(|key_ref|(*key_ref).clone())
    }

    pub fn remove(handle: &KeyPairHandle) {
        let mut store = KEY_STORE.lock().unwrap();
        store.keys.remove(handle);
    }

    pub fn clear() {
        let mut store = KEY_STORE.lock().unwrap();
        store.keys.clear();
    }

    pub fn decode_secret(secret: &Option<String>, handle: &Option<String>) -> ApiResult<Vec<u8>> {
        if let Some(secret) = secret {
            hex_decode(secret)
        } else if let Some(handle) = handle {
            if let Some(keys) = Self::get(handle) {
                hex_decode(&keys.secret)
            } else {
                Err(ApiError::crypto_invalid_keystore_handle())
            }
        } else {
            Err(ApiError::crypto_missing_key_source())
        }
    }
}


pub fn decode_public_key(string: &String) -> ApiResult<PublicKey> {
    PublicKey::from_bytes(parse_key(string)?.as_slice())
        .map_err(|err| ApiError::crypto_invalid_public_key(err, string))
}

pub fn decode_secret_key(string: &String) -> ApiResult<SecretKey> {
    SecretKey::from_bytes(parse_key(string)?.as_slice())
        .map_err(|err| ApiError::crypto_invalid_secret_key(err, string))
}

//pub fn u256_zero() -> UInt256 { [0; 32].into() }

pub fn u256_encode(value: &UInt256) -> String {
    hex::encode(value.as_slice())
}

pub fn u256_from_slice_data(slice: &tvm::stack::SliceData) -> UInt256 {
    UInt256::from(slice.storage().as_slice())
}

pub fn account_encode(value: &tvm::types::AccountId) -> String {
    hex::encode(&u256_from_slice_data(value))
}

pub fn account_decode(string: &String) -> ApiResult<ton_sdk::AccountAddress> {
    ton_sdk::AccountAddress::from_str(&string)
        .map_err(|err| {
                let err = format!("{:?}", err);
                ApiError::crypto_invalid_address(err, string)
            })
}

// Internals

fn parse_key(s: &String) -> ApiResult<Vec<u8>> {
    hex::decode(s).map_err(|err| ApiError::crypto_invalid_key(err, s))
}

// Internals

pub(crate) fn key512(slice: &[u8]) -> ApiResult<Key512> {
    if slice.len() != 64 {
        return Err(ApiError::crypto_invalid_key_size(slice.len(), 64));
    }
    let mut key = [0u8; 64];
    for (place, element) in key.iter_mut().zip(slice.iter()) {
        *place = *element;
    }
    Ok(key)
}

pub(crate) fn key256(slice: &[u8]) -> ApiResult<Key256> {
    if slice.len() != 32 {
        return Err(ApiError::crypto_invalid_key_size(slice.len(), 32));
    }
    let mut key = [0u8; 32];
    for (place, element) in key.iter_mut().zip(slice.iter()) {
        *place = *element;
    }
    Ok(key)
}

pub(crate) fn key192(slice: &[u8]) -> ApiResult<Key192> {
    if slice.len() != 24 {
        return Err(ApiError::crypto_invalid_key_size(slice.len(), 24));
    }
    let mut key = [0u8; 24];
    for (place, element) in key.iter_mut().zip(slice.iter()) {
        *place = *element;
    }
    Ok(key)
}


pub(crate) fn key_to_ton_string(key: &[u8]) -> String {
    let mut public_key: Vec<u8> = Vec::new();
    public_key.push(0x3e);
    public_key.push(0xe6);
    public_key.extend_from_slice(key);
    let hash = ton_crc16(&public_key);
    public_key.push((hash >> 8) as u8);
    public_key.push((hash & 255) as u8);
    return base64::encode_config(&public_key, URL_SAFE);
}

pub(crate) fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut hmac = Hmac::<Sha512>::new_varkey(key).unwrap();
    hmac.input(&data);
    let mut result = [0u8; 64];
    result.copy_from_slice(&hmac.result().code());
    result

}

pub(crate) fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], c: usize) -> [u8; 64] {
    let mut result = [0u8; 64];
    pbkdf2::pbkdf2::<Hmac<Sha512>>(password, salt, c, &mut result);
    result
}
