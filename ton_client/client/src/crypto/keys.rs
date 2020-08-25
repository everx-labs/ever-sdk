/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::types::{ApiResult, ApiError, hex_decode};
use crate::crypto::math::ton_crc16;
use std::sync::Mutex;
use ton_block::MsgAddressInt;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use std::collections::HashMap;
use base64::URL_SAFE;
use hmac::*;
use sha2::Sha512;
use std::str::FromStr;

pub type Key192 = [u8; 24];
pub type Key256 = [u8; 32];
pub type Key264 = [u8; 33];
pub type Key512 = [u8; 64];

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
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

pub fn account_encode(value: &MsgAddressInt) -> String {
    value.to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum AccountAddressType {
    AccountId,
    Hex,
    Base64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Base64AddressParams {
    pub url: bool,
    pub test: bool,
    pub bounce: bool
}

pub(crate) fn account_encode_ex(
    value: &MsgAddressInt,
    addr_type: AccountAddressType,
    base64_params: Option<Base64AddressParams>
) -> ApiResult<String> {
    match addr_type {
        AccountAddressType::AccountId => Ok(value.get_address().to_hex_string()),
        AccountAddressType::Hex => Ok(value.to_string()),
        AccountAddressType::Base64 => {
            let params = base64_params.ok_or(ApiError::contracts_address_conversion_failed(
                "No base64 address parameters provided".to_owned()))?;
            encode_base64(value, params.bounce, params.test, params.url)
        }
    }
}

pub fn account_decode(string: &str) -> ApiResult<MsgAddressInt> {
    match MsgAddressInt::from_str(string) {
        Ok(address) => Ok(address),
        Err(_) if string.len() == 48 => {
            decode_std_base64(string)
        },
        Err(err) => Err(ApiError::crypto_invalid_address(err, string))
    }
}

fn decode_std_base64(data: &str) -> ApiResult<MsgAddressInt> {
    // conversion from base64url
    let data = data.replace('_', "/").replace('-', "+");

    let vec = base64::decode(&data)
        .map_err(|err| ApiError::crypto_invalid_address(err, &data))?;

    // check CRC and address tag
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(&vec[..34]);

    if crc.get_crc_vec_be() != &vec[34..36] || vec[0] & 0x3f != 0x11 {
        return Err(ApiError::crypto_invalid_address("CRC mismatch", &data).into());
    };

    MsgAddressInt::with_standart(None, vec[1] as i8, vec[2..34].into())
        .map_err(|err| ApiError::crypto_invalid_address(err, &data).into())
}

fn encode_base64(address: &MsgAddressInt, bounceable: bool, test: bool, as_url: bool) -> ApiResult<String> {
    if let MsgAddressInt::AddrStd(address) = address {
        let mut tag = if bounceable { 0x11 } else { 0x51 };
        if test { tag |= 0x80 };
        let mut vec = vec![tag];
        vec.extend_from_slice(&address.workchain_id.to_be_bytes());
        vec.append(&mut address.address.get_bytestring(0));

        let mut crc = crc_any::CRC::crc16xmodem();
        crc.digest(&vec);
        vec.extend_from_slice(&crc.get_crc_vec_be());

        let result = base64::encode(&vec);

        if as_url {
            Ok(result.replace('/', "_").replace('+', "-"))
        } else {
            Ok(result)
        }
    } else {
        Err(ApiError::crypto_invalid_address("Non-std address", &address.to_string()).into())
    }
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

