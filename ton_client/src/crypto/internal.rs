use crate::crypto;
use crate::error::ClientResult;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use hmac::*;
use sha2::Digest;
use sha2::Sha512;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretString(pub String);

#[derive(Debug, Default, Clone, Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretBuf(pub Vec<u8>);

#[derive(Debug, Clone)]
pub(crate) struct SecretBufConst<const N: usize>(pub [u8; N]);

impl<const N: usize> Drop for SecretBufConst<N> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub(crate) type Key192 = [u8; 24];
pub(crate) type Key256 = [u8; 32];
pub(crate) type Key264 = [u8; 33];
pub(crate) type Key512 = [u8; 64];

pub(crate) fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub(crate) fn ton_crc16(data: &[u8]) -> u16 {
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(data);
    crc.get_crc() as u16
}

pub(crate) fn decode_public_key(string: &String) -> ClientResult<PublicKey> {
    PublicKey::from_bytes(parse_key(string)?.as_slice())
        .map_err(|err| crypto::Error::invalid_public_key(err, string))
}

pub(crate) fn decode_secret_key(string: &String) -> ClientResult<SecretKey> {
    SecretKey::from_bytes(parse_key(string)?.as_slice())
        .map_err(|err| crypto::Error::invalid_secret_key(err, string))
}

fn parse_key(s: &String) -> ClientResult<Vec<u8>> {
    hex::decode(s).map_err(|err| crypto::Error::invalid_key(err, s))
}

pub(crate) fn key_from_slice<const N: usize>(slice: &[u8]) -> ClientResult<[u8; N]> {
    if slice.len() != N {
        return Err(crypto::Error::invalid_key_size(slice.len(), &[N]));
    }
    let mut key = [0u8; N];
    key.copy_from_slice(slice);
    Ok(key)
}

pub(crate) fn key512(slice: &[u8]) -> ClientResult<Key512> {
    key_from_slice(slice)
}

pub(crate) fn key256(slice: &[u8]) -> ClientResult<Key256> {
    key_from_slice(slice)
}

pub(crate) fn key192(slice: &[u8]) -> ClientResult<Key192> {
    key_from_slice(slice)
}

pub(crate) fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut hmac = Hmac::<Sha512>::new_from_slice(key).unwrap();
    hmac.update(&data);
    let mut result = [0u8; 64];
    result.copy_from_slice(&hmac.finalize().into_bytes());
    result
}

pub(crate) fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], c: u32) -> [u8; 64] {
    let mut result = [0u8; 64];
    pbkdf2::pbkdf2::<Hmac<Sha512>>(password, salt, c, &mut result);
    result
}

pub(crate) fn sign_using_secret(
    unsigned: &[u8],
    secret: &[u8],
) -> ClientResult<(Vec<u8>, Vec<u8>)> {
    let mut signed: Vec<u8> = Vec::new();
    signed.resize(unsigned.len() + sodalite::SIGN_LEN, 0);
    sodalite::sign_attached(&mut signed, unsigned, &key512(secret)?);
    let mut signature: Vec<u8> = Vec::new();
    signature.resize(64, 0);
    for (place, element) in signature.iter_mut().zip(signed.iter()) {
        *place = *element;
    }
    Ok((signed, signature))
}

pub(crate) fn sign_using_keys(unsigned: &[u8], keys: &Keypair) -> ClientResult<(Vec<u8>, Vec<u8>)> {
    let mut secret = Vec::<u8>::new();
    secret.extend(keys.secret.as_bytes());
    secret.extend(keys.public.as_bytes());
    sign_using_secret(unsigned, &secret)
}
