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

use crate::crypto::keys::{KeyPair};
use crate::crypto::internal::{key512, key256, key192};
use crate::error::{ApiResult, ApiError};
use crate::encoding::{InputData, OutputEncoding, hex_decode};
use crate::client::ClientContext;

// Signing

//------------------------------------------------------------------------------- nacl_sign_keypair

/// Randomly generates a secret key and a corresponding public key
pub fn nacl_sign_keypair(_context: std::sync::Arc<ClientContext>) -> ApiResult<KeyPair> {
    let mut sk = [0u8; 64];
    let mut pk = [0u8; 32];
    sodalite::sign_keypair(&mut pk, &mut sk);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk.as_ref())))
}

//------------------------------------------------------------------------ sign_keypair_from_secret

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSignKeyPairFromSecret {
    /// Signer's secret key.
    pub secret: String,
}

pub fn nacl_sign_keypair_from_secret_key(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSignKeyPairFromSecret,
) -> ApiResult<KeyPair> {
    let secret = hex::decode(&params.secret).map_err(|err|
        ApiError::crypto_invalid_secret_key(err, &params.secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 64];
    let mut pk = [0u8; 32];
    sodalite::sign_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk.as_ref())))
}


//--------------------------------------------------------------------------------------- nacl_sign

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSign {
    /// Data that must be signed.
    pub unsigned: InputData,
    /// Signer's secret key.
    pub secret: String,
    /// Encoding for signed data. Default is `base64`.
    pub output_encoding: Option<OutputEncoding>,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfNaclSign {
    /// Signed data, encoded with `output_encoding`.
    pub signed: String,
}

/// Signs a data using the signer's secret key.
pub fn nacl_sign(_context: std::sync::Arc<ClientContext>, params: ParamsOfNaclSign) -> ApiResult<ResultOfNaclSign> {
    let signed = sign(params.unsigned.decode()?, hex_decode(&params.secret)?)?;
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Base64);
    Ok(ResultOfNaclSign {
        signed: encoding.encode(signed)?,
    })
}

//------------------------------------------------------------------------------ nacl_sign_detached

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSignDetached {
    /// Data that must be signed.
    pub unsigned: InputData,
    /// Signer's secret key.
    pub secret: String,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfNaclSignDetached {
    /// Hex encoded sign.
    pub sign: String,
}

pub fn nacl_sign_detached(_context: std::sync::Arc<ClientContext>, params: ParamsOfNaclSign) -> ApiResult<ResultOfNaclSignDetached> {
    let signed = sign(params.unsigned.decode()?, hex_decode(&params.secret)?)?;
    let mut sign: Vec<u8> = Vec::new();
    sign.resize(64, 0);
    for (place, element) in sign.iter_mut().zip(signed.iter()) {
        *place = *element;
    }
    Ok(ResultOfNaclSignDetached {
        sign: hex::encode(sign),
    })
}

//---------------------------------------------------------------------------------- nacl_sign_open

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSignOpen {
    /// Signed data that must be unsigned.
    pub signed: InputData,
    /// Signer's public key.
    pub public: String,
    /// Encoding for verified data. Default is `base64`.
    pub output_encoding: Option<OutputEncoding>,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfNaclSignOpen {
    /// Unsigned data, encoded with `output_encoding`.
    pub unsigned: String,
}

pub fn nacl_sign_open(_context: std::sync::Arc<ClientContext>, params: ParamsOfNaclSignOpen) -> ApiResult<ResultOfNaclSignOpen> {
    let mut unsigned: Vec<u8> = Vec::new();
    let signed = params.signed.decode()?;
    unsigned.resize(signed.len(), 0);
    let len = sodalite::sign_attached_open(
        &mut unsigned,
        &signed,
        &key256(&hex_decode(&params.public)?)?,
    ).map_err(|_| ApiError::crypto_nacl_sign_failed("box sign open failed"))?;
    unsigned.resize(len, 0);
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Hex);
    Ok(ResultOfNaclSignOpen {
        unsigned: encoding.encode(unsigned)?,
    })
}

// Box

fn prepare_to_convert(input: &Vec<u8>, nonce: &Vec<u8>, key: &Vec<u8>, pad_len: usize)
    -> ApiResult<(Vec<u8>, Vec<u8>, [u8; 24], [u8; 32])> {
    let mut padded_input = Vec::new();
    padded_input.resize(pad_len, 0);
    padded_input.extend(input);
    let mut padded_output = Vec::new();
    padded_output.resize(padded_input.len(), 0);
    Ok((padded_output, padded_input, key192(&nonce)?, key256(&key)?))
}

//-------------------------------------------------------------------------------- nacl_box_keypair

pub fn nacl_box_keypair(_context: std::sync::Arc<ClientContext>) -> ApiResult<KeyPair> {
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair(&mut pk, &mut sk);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

//-------------------------------------------------------------------- nacl_box_keypair_from_secret

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclBoxKeyPairFromSecret {
    /// Hex encoded secret key.
    pub secret: String,
}

pub fn nacl_box_keypair_from_secret_key(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclBoxKeyPairFromSecret,
) -> ApiResult<KeyPair> {
    let secret = hex::decode(&params.secret).map_err(|err|
        ApiError::crypto_invalid_secret_key(err, &params.secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

//---------------------------------------------------------------------------------------- nacl_box

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclBox {
    pub decrypted: InputData,
    pub nonce: String,
    pub their_public: String,
    pub secret: String,
    pub output_encoding: Option<OutputEncoding>,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfNaclBox {
    pub encrypted: String,
}

pub fn nacl_box(_context: std::sync::Arc<ClientContext>, params: ParamsOfNaclBox) -> ApiResult<ResultOfNaclBox> {
    let (mut padded_output, padded_input, nonce, secret) =
        prepare_to_convert(
            &params.decrypted.decode()?,
            &hex_decode(&params.nonce)?,
            &hex_decode(&params.secret)?,
            32,
        )?;

    sodalite::box_(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key256(&hex_decode(&params.their_public)?)?,
        &secret,
    ).map_err(|_| ApiError::crypto_nacl_box_failed("box failed"))?;
    padded_output.drain(..16);
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Base64);
    Ok(ResultOfNaclBox { encrypted: encoding.encode(padded_output)? })
}

//----------------------------------------------------------------------------------- nacl_box_open

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclBoxOpen {
    pub encrypted: InputData,
    pub nonce: String,
    pub their_public: String,
    pub secret: String,
    pub output_encoding: Option<OutputEncoding>,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfNaclBoxOpen {
    pub decrypted: String,
}

pub fn nacl_box_open(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclBoxOpen,
) -> ApiResult<ResultOfNaclBoxOpen> {
    let (mut padded_output, padded_input, nonce, secret) =
        prepare_to_convert(
            &params.encrypted.decode()?,
            &hex_decode(&params.nonce)?,
            &hex_decode(&params.secret)?,
            16,
        )?;
    sodalite::box_open(
        &mut padded_output,
        &padded_input,
        &nonce,
        &key256(&hex_decode(&params.their_public)?)?,
        &secret,
    ).map_err(|_| ApiError::crypto_nacl_box_failed("box open failed"))?;
    padded_output.drain(..32);
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Base64);
    Ok(ResultOfNaclBoxOpen { decrypted: encoding.encode(padded_output)? })
}

// Secret Box

//--------------------------------------------------------------------------------- nacl_secret_box

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSecretBox {
    pub decrypted: InputData,
    pub nonce: String,
    pub key: String,
    pub output_encoding: Option<OutputEncoding>,
}

pub fn nacl_secret_box(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSecretBox,
) -> ApiResult<ResultOfNaclBox> {
    let (mut padded_output, padded_input, nonce, key) =
        prepare_to_convert(
            &params.decrypted.decode()?,
            &hex_decode(&params.nonce)?,
            &hex_decode(&params.key)?,
            32,
        )?;

    sodalite::secretbox(&mut padded_output, &padded_input, &nonce, &key)
        .map_err(|_| ApiError::crypto_nacl_secret_box_failed("secret box failed"))?;
    padded_output.drain(..16);
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Base64);
    Ok(ResultOfNaclBox { encrypted: encoding.encode(padded_output)? })
}

//---------------------------------------------------------------------------- nacl_secret_box_open

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfNaclSecretBoxOpen {
    pub encrypted: InputData,
    pub nonce: String,
    pub key: String,
    pub output_encoding: Option<OutputEncoding>,
}

pub fn nacl_secret_box_open(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSecretBoxOpen,
) -> ApiResult<ResultOfNaclBoxOpen> {
    let (mut padded_output, padded_input, nonce, key) =
        prepare_to_convert(
            &params.encrypted.decode()?,
            &hex_decode(&params.nonce)?,
            &hex_decode(&params.key)?,
            16,
        )?;

    sodalite::secretbox_open(&mut padded_output, &padded_input, &nonce, &key)
        .map_err(|_| ApiError::crypto_nacl_secret_box_failed("secret box open failed"))?;
    padded_output.drain(..32);
    let encoding = params.output_encoding.unwrap_or(OutputEncoding::Base64);
    Ok(ResultOfNaclBoxOpen { decrypted: encoding.encode(padded_output)? })
}

// Internals

fn sign(unsigned: Vec<u8>, secret: Vec<u8>) -> ApiResult<Vec<u8>> {
    let mut signed: Vec<u8> = Vec::new();
    signed.resize(unsigned.len() + sodalite::SIGN_LEN, 0);
    sodalite::sign_attached(&mut signed, &unsigned, &key512(&secret)?);
    Ok(signed)
}
