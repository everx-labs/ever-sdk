/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crypto::keys::{KeyPair, key512, key256, key192};
use types::{ApiResult, ApiError};

// Keys

pub fn sign_keypair() -> ApiResult<KeyPair> {
    let mut sk = [0u8; 64];
    let mut pk = [0u8; 32];
    sodalite::sign_keypair(&mut pk, &mut sk);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk.as_ref())))
}

pub fn sign_keypair_from_secret_key(secret: &String) -> ApiResult<KeyPair> {
    let secret = hex::decode(secret).map_err(|err|
        ApiError::crypto_invalid_secret_key(err, secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 64];
    let mut pk = [0u8; 32];
    sodalite::sign_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk.as_ref())))
}

pub fn box_keypair() -> ApiResult<KeyPair> {
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair(&mut pk, &mut sk);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

pub fn box_keypair_from_secret_key(secret: &String) -> ApiResult<KeyPair> {
    let secret = hex::decode(secret).map_err(|err|
        ApiError::crypto_invalid_secret_key(err, secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

// Secret Box

fn prepare_to_convert(input: &Vec<u8>, nonce: &Vec<u8>, key: &Vec<u8>, pad_len: usize)
    -> ApiResult<(Vec<u8>, Vec<u8>, [u8; 24], [u8;32])> {
    let mut padded_input = Vec::new();
    padded_input.resize(pad_len, 0);
    padded_input.extend(input);
    let mut padded_output = Vec::new();
    padded_output.resize(padded_input.len(), 0);
    Ok((padded_output, padded_input, key192(&nonce)?, key256(&key)?))
}

pub fn secret_box(input: Vec<u8>, nonce: Vec<u8>, key: Vec<u8>) -> ApiResult<Vec<u8>> {
    let (
        mut padded_output,
        padded_input,
        nonce,
        key
    ) = prepare_to_convert(&input, &nonce, &key, 32)?;

    sodalite::secretbox(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key
    ).map_err(|_|ApiError::crypto_nacl_secret_box_failed("secret box failed"))?;
    padded_output.drain(..16);
    Ok(padded_output)
}

pub fn secret_box_open(input: Vec<u8>, nonce: Vec<u8>, key: Vec<u8>) -> ApiResult<Vec<u8>> {
    let (
        mut padded_output,
        padded_input,
        nonce,
        key
    ) = prepare_to_convert(&input, &nonce, &key, 16)?;

    sodalite::secretbox_open(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key
    ).map_err(|_|ApiError::crypto_nacl_secret_box_failed("secret box open failed"))?;
    padded_output.drain(..32);
    Ok(padded_output)
}

// Box

pub fn box_(input: Vec<u8>, nonce: Vec<u8>, their_public: Vec<u8>, secret: Vec<u8>) -> ApiResult<Vec<u8>> {
    let (
        mut padded_output,
        padded_input,
        nonce,
        secret
    ) = prepare_to_convert(&input, &nonce, &secret, 32)?;

    sodalite::box_(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key256(&their_public)?,
        &secret
    ).map_err(|_|ApiError::crypto_nacl_box_failed("box failed"))?;
    padded_output.drain(..16);
    Ok(padded_output)
}

pub fn box_open(input: Vec<u8>, nonce: Vec<u8>, their_public: Vec<u8>, secret: Vec<u8>) -> ApiResult<Vec<u8>> {
    let (
        mut padded_output,
        padded_input,
        nonce,
        secret
    ) = prepare_to_convert(&input, &nonce, &secret, 16)?;
    sodalite::box_open(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key256(&their_public)?,
        &secret
    ).map_err(|_|ApiError::crypto_nacl_box_failed("box open failed"))?;
    padded_output.drain(..32);
    Ok(padded_output)
}

// Sign

pub fn sign(input: Vec<u8>, secret: Vec<u8>) -> ApiResult<Vec<u8>> {
    let mut output: Vec<u8> = Vec::new();
    output.resize(input.len() + sodalite::SIGN_LEN, 0);
    sodalite::sign_attached(
        &mut output,
        &input,
        &key512(&secret)?
    );
    Ok(output)
}

pub fn sign_open(input: Vec<u8>, public: Vec<u8>) -> ApiResult<Vec<u8>> {
    let mut output: Vec<u8> = Vec::new();
    output.resize(input.len(), 0);
    let len = sodalite::sign_attached_open(
        &mut output,
        &input,
        &key256(&public)?
    ).map_err(|_|ApiError::crypto_nacl_sign_failed("box sign open failed"))?;
    output.resize(len, 0);
    Ok(output)
}

pub fn sign_detached(input: Vec<u8>, secret: Vec<u8>) -> ApiResult<Vec<u8>> {
    let signed = sign(input, secret)?;
    let mut sign: Vec<u8> = Vec::new();
    sign.resize(64, 0);
    for (place, element) in sign.iter_mut().zip(signed.iter()) {
        *place = *element;
    }
    Ok(sign)
}

