/*
* Copyright 2018-2021 TON DEV SOLUTIONS LTD.
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

use crate::client::ClientContext;
use crate::crypto;
use crate::crypto::internal::{key192, key256, key512};
use crate::crypto::keys::KeyPair;
use crate::crypto::{internal, Error};
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;
use ed25519_dalek::Verifier;

// Signing

//------------------------------------------------------------------------ sign_keypair_from_secret
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSignKeyPairFromSecret {
    /// Secret key - unprefixed 0-padded to 64 symbols hex string
    pub secret: String,
}

/// Generates a key pair for signing from the secret key
/// 
/// **NOTE:** In the result the secret key is actually the concatenation 
/// of secret and public keys (128 symbols hex string) by design of [NaCL](http://nacl.cr.yp.to/sign.html).
/// See also [the stackexchange question](https://crypto.stackexchange.com/questions/54353/).
#[api_function]
pub fn nacl_sign_keypair_from_secret_key(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSignKeyPairFromSecret,
) -> ClientResult<KeyPair> {
    let secret = hex::decode(&params.secret)
        .map_err(|err| crypto::Error::invalid_secret_key(err, &params.secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 64];
    let mut pk = [0u8; 32];
    sodalite::sign_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk.as_ref())))
}

//--------------------------------------------------------------------------------------- nacl_sign
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSign {
    /// Data that must be signed encoded in `base64`.
    pub unsigned: String,
    /// Signer's secret key - unprefixed 0-padded to 128 symbols hex string
    /// (concatenation of 64 symbols secret and 64 symbols public keys).
    /// See `nacl_sign_keypair_from_secret_key`.
    pub secret: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclSign {
    /// Signed data, encoded in `base64`.
    pub signed: String,
}

/// Signs data using the signer's secret key.
#[api_function]
pub fn nacl_sign(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSign,
) -> ClientResult<ResultOfNaclSign> {
    let signed = sign(
        base64_decode(&params.unsigned)?,
        hex_decode(&params.secret)?,
    )?;
    Ok(ResultOfNaclSign {
        signed: base64::encode(&signed),
    })
}

//------------------------------------------------------------------------------ nacl_sign_detached
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSignDetached {
    /// Data that must be signed encoded in `base64`.
    pub unsigned: String,
    /// Signer's secret key - unprefixed 0-padded to 128 symbols hex string
    /// (concatenation of 64 symbols secret and 64 symbols public keys).
    /// See `nacl_sign_keypair_from_secret_key`.
    pub secret: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclSignDetached {
    /// Signature encoded in `hex`.
    pub signature: String,
}

/// Signs the message using the secret key and returns a signature.
///
/// Signs the message `unsigned` using the secret key `secret`
/// and returns a signature `signature`.

#[api_function]
pub fn nacl_sign_detached(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSign,
) -> ClientResult<ResultOfNaclSignDetached> {
    let (_, signature) = internal::sign_using_secret(
        &base64_decode(&params.unsigned)?,
        &hex_decode(&params.secret)?,
    )?;
    Ok(ResultOfNaclSignDetached {
        signature: hex::encode(signature),
    })
}

//---------------------------------------------------------------------------------- nacl_sign_open
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSignOpen {
    /// Signed data that must be unsigned. Encoded with `base64`.
    pub signed: String,
    /// Signer's public key - unprefixed 0-padded to 64 symbols hex string
    pub public: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclSignOpen {
    /// Unsigned data, encoded in `base64`.
    pub unsigned: String,
}

/// Verifies the signature and returns the unsigned message
///
/// Verifies the signature in `signed` using the signer's public key `public`
/// and returns the message `unsigned`.
///
/// If the signature fails verification, crypto_sign_open raises an exception.
#[api_function]
pub fn nacl_sign_open(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSignOpen,
) -> ClientResult<ResultOfNaclSignOpen> {
    let mut unsigned: Vec<u8> = Vec::new();
    let signed = base64_decode(&params.signed)?;
    unsigned.resize(signed.len(), 0);
    let len = sodalite::sign_attached_open(
        &mut unsigned,
        &signed,
        &key256(&hex_decode(&params.public)?)?,
    )
    .map_err(|_| crypto::Error::nacl_sign_failed("box sign open failed"))?;
    unsigned.resize(len, 0);
    Ok(ResultOfNaclSignOpen {
        unsigned: base64::encode(&unsigned),
    })
}

//----------------------------------------------------------------------- nacl_sign_detached_verify

///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSignDetachedVerify {
    /// Unsigned data that must be verified. Encoded with `base64`.
    pub unsigned: String,
    /// Signature that must be verified. Encoded with `hex`.
    pub signature: String,
    /// Signer's public key - unprefixed 0-padded to 64 symbols hex string.
    pub public: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclSignDetachedVerify {
    /// `true` if verification succeeded or `false` if it failed
    pub(crate) succeeded: bool,
}

/// Verifies the signature with public key and `unsigned` data.
#[api_function]
pub fn nacl_sign_detached_verify(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSignDetachedVerify,
) -> ClientResult<ResultOfNaclSignDetachedVerify> {
    let public = ed25519_dalek::PublicKey::from_bytes(&hex_decode(&params.public)?)
        .map_err(|err| Error::invalid_public_key(err, &params.public))?;
    let message = base64_decode(&params.unsigned)?;
    let signature = ed25519_dalek::Signature::from_bytes(&key512(&hex_decode(&params.signature)?)?)
        .map_err(|err| Error::invalid_signature(err, &params.signature))?;
    let succeeded = public.verify(&message, &signature).is_ok();
    Ok(ResultOfNaclSignDetachedVerify { succeeded })
}

// Box

fn prepare_to_convert(
    input: &Vec<u8>,
    nonce: &Vec<u8>,
    key: &Vec<u8>,
    pad_len: usize,
) -> ClientResult<(Vec<u8>, Vec<u8>, [u8; 24], [u8; 32])> {
    let mut padded_input = Vec::new();
    padded_input.resize(pad_len, 0);
    padded_input.extend(input);
    let mut padded_output = Vec::new();
    padded_output.resize(padded_input.len(), 0);
    Ok((padded_output, padded_input, key192(&nonce)?, key256(&key)?))
}

//-------------------------------------------------------------------------------- nacl_box_keypair

/// Generates a random NaCl key pair
#[api_function]
pub fn nacl_box_keypair(_context: std::sync::Arc<ClientContext>) -> ClientResult<KeyPair> {
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair(&mut pk, &mut sk);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

//-------------------------------------------------------------------- nacl_box_keypair_from_secret
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclBoxKeyPairFromSecret {
    /// Secret key - unprefixed 0-padded to 64 symbols hex string
    pub secret: String,
}

#[api_function]
/// Generates key pair from a secret key
pub fn nacl_box_keypair_from_secret_key(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclBoxKeyPairFromSecret,
) -> ClientResult<KeyPair> {
    let secret = hex::decode(&params.secret)
        .map_err(|err| crypto::Error::invalid_secret_key(err, &params.secret))?;
    let seed = key256(&secret)?;
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 32];
    sodalite::box_keypair_seed(&mut pk, &mut sk, &seed);
    Ok(KeyPair::new(hex::encode(pk), hex::encode(sk)))
}

//---------------------------------------------------------------------------------------- nacl_box
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclBox {
    /// Data that must be encrypted encoded in `base64`.
    pub decrypted: String,
    /// Nonce, encoded in `hex`
    pub nonce: String,
    /// Receiver's public key - unprefixed 0-padded to 64 symbols hex string
    pub their_public: String,
    /// Sender's private key - unprefixed 0-padded to 64 symbols hex string
    pub secret: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclBox {
    /// Encrypted data encoded in `base64`.
    pub encrypted: String,
}

/// Public key authenticated encryption
///
/// Encrypt and authenticate a message using the senders secret key, the receivers public
/// key, and a nonce.
#[api_function]
pub fn nacl_box(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclBox,
) -> ClientResult<ResultOfNaclBox> {
    let (mut padded_output, padded_input, nonce, secret) = prepare_to_convert(
        &base64_decode(&params.decrypted)?,
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
    )
    .map_err(|_| crypto::Error::nacl_box_failed("box failed"))?;
    padded_output.drain(..16);
    Ok(ResultOfNaclBox {
        encrypted: base64::encode(&padded_output),
    })
}

//----------------------------------------------------------------------------------- nacl_box_open
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclBoxOpen {
    /// Data that must be decrypted. Encoded with `base64`.
    pub encrypted: String,
    // Nonce
    pub nonce: String,
    /// Sender's public key - unprefixed 0-padded to 64 symbols hex string
    pub their_public: String,
    /// Receiver's private key - unprefixed 0-padded to 64 symbols hex string
    pub secret: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfNaclBoxOpen {
    /// Decrypted data encoded in `base64`.
    pub decrypted: String,
}

/// Decrypt and verify the cipher text using the receivers secret key, the senders public
/// key, and the nonce.
#[api_function]
pub fn nacl_box_open(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclBoxOpen,
) -> ClientResult<ResultOfNaclBoxOpen> {
    let (mut padded_output, padded_input, nonce, secret) = prepare_to_convert(
        &base64_decode(&params.encrypted)?,
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
    )
    .map_err(|_| crypto::Error::nacl_box_failed("box open failed"))?;
    padded_output.drain(..32);
    Ok(ResultOfNaclBoxOpen {
        decrypted: base64::encode(&padded_output),
    })
}

// Secret Box

//--------------------------------------------------------------------------------- nacl_secret_box
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSecretBox {
    /// Data that must be encrypted. Encoded with `base64`.
    pub decrypted: String,
    /// Nonce in `hex`
    pub nonce: String,
    /// Secret key - unprefixed 0-padded to 64 symbols hex string
    pub key: String,
}

#[api_function]
/// Encrypt and authenticate message using nonce and secret key.
pub fn nacl_secret_box(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSecretBox,
) -> ClientResult<ResultOfNaclBox> {
    let (mut padded_output, padded_input, nonce, key) = prepare_to_convert(
        &base64_decode(&params.decrypted)?,
        &hex_decode(&params.nonce)?,
        &hex_decode(&params.key)?,
        32,
    )?;

    sodalite::secretbox(&mut padded_output, &padded_input, &nonce, &key)
        .map_err(|_| crypto::Error::nacl_secret_box_failed("secret box failed"))?;
    padded_output.drain(..16);
    Ok(ResultOfNaclBox {
        encrypted: base64::encode(&padded_output),
    })
}

//---------------------------------------------------------------------------- nacl_secret_box_open
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfNaclSecretBoxOpen {
    /// Data that must be decrypted. Encoded with `base64`.
    pub encrypted: String,
    /// Nonce in `hex`
    pub nonce: String,
    /// Public key - unprefixed 0-padded to 64 symbols hex string
    pub key: String,
}

#[api_function]
/// Decrypts and verifies cipher text using `nonce` and secret `key`.
pub fn nacl_secret_box_open(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfNaclSecretBoxOpen,
) -> ClientResult<ResultOfNaclBoxOpen> {
    let (mut padded_output, padded_input, nonce, key) = prepare_to_convert(
        &base64_decode(&params.encrypted)?,
        &hex_decode(&params.nonce)?,
        &hex_decode(&params.key)?,
        16,
    )?;

    sodalite::secretbox_open(&mut padded_output, &padded_input, &nonce, &key)
        .map_err(|_| crypto::Error::nacl_secret_box_failed("secret box open failed"))?;
    padded_output.drain(..32);
    Ok(ResultOfNaclBoxOpen {
        decrypted: base64::encode(&padded_output),
    })
}

// Internals

fn sign(unsigned: Vec<u8>, secret: Vec<u8>) -> ClientResult<Vec<u8>> {
    let mut signed: Vec<u8> = Vec::new();
    signed.resize(unsigned.len() + sodalite::SIGN_LEN, 0);
    sodalite::sign_attached(&mut signed, &unsigned, &key512(&secret)?);
    Ok(signed)
}
