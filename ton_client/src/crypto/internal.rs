use crate::crypto;
use crate::error::ClientResult;
use ed25519_dalek::{SigningKey, VerifyingKey};
use hmac::*;
use sha2::Digest;
use sha2::Sha512;
use zeroize::Zeroize;

const XMODEM: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

#[derive(Serialize, Deserialize, Debug, PartialEq, Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretString(pub String);

#[derive(Debug, Default, Clone, Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretBuf(pub Vec<u8>);

impl std::ops::Deref for SecretBuf {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SecretBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[u8]> for SecretBuf {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SecretBufConst<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for SecretBufConst<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> Drop for SecretBufConst<N> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<const N: usize> std::ops::Deref for SecretBufConst<N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> std::ops::DerefMut for SecretBufConst<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> AsRef<[u8]> for SecretBufConst<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for SecretBufConst<N> {
    fn from(data: [u8; N]) -> Self {
        Self(data)
    }
}

pub(crate) type Key192 = SecretBufConst<24>;
pub(crate) type Key256 = SecretBufConst<32>;
pub(crate) type Key264 = SecretBufConst<33>;
pub(crate) type Key512 = SecretBufConst<64>;

pub(crate) fn hex_decode_secret(hex: &str) -> ClientResult<SecretBuf> {
    crate::encoding::hex_decode(hex).map(|data| SecretBuf(data))
}

pub(crate) fn hex_decode_secret_const<const N: usize>(hex: &str) -> ClientResult<SecretBufConst<N>> {
    key_from_slice(&hex_decode_secret(hex)?)
}

pub(crate) fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub(crate) fn ton_crc16(data: &[u8]) -> u16 {
    XMODEM.checksum(data)
}

pub(crate) fn decode_public_key(string: &String) -> ClientResult<VerifyingKey> {
    VerifyingKey::from_bytes(
        &hex_decode_secret_const(string)
            .map_err(|err| crypto::Error::invalid_public_key(err, string))?.0
    )
    .map_err(|err| crypto::Error::invalid_public_key(err, string))
}

pub(crate) fn decode_secret_key(string: &String) -> ClientResult<SigningKey> {
    Ok(SigningKey::from_bytes(
        &hex_decode_secret_const(string)
            .map_err(|err| crypto::Error::invalid_secret_key(err, string))?.0
    ))
}

pub(crate) fn key_from_slice<const N: usize>(slice: &[u8]) -> ClientResult<SecretBufConst<N>> {
    if slice.len() != N {
        return Err(crypto::Error::invalid_key_size(slice.len(), &[N]));
    }
    let mut key = SecretBufConst([0u8; N]);
    key.0.copy_from_slice(slice);
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
    sodalite::sign_attached(&mut signed, unsigned, &key512(secret)?.0);
    let mut signature: Vec<u8> = Vec::new();
    signature.resize(64, 0);
    for (place, element) in signature.iter_mut().zip(signed.iter()) {
        *place = *element;
    }
    Ok((signed, signature))
}

pub(crate) fn sign_using_keys(unsigned: &[u8], sign_key: &SigningKey) -> ClientResult<(Vec<u8>, Vec<u8>)> {
    let mut secret = SecretBuf(Vec::with_capacity(ed25519_dalek::KEYPAIR_LENGTH));
    secret.0.extend(&SecretBufConst(sign_key.to_bytes()).0);
    secret.0.extend(sign_key.verifying_key().as_bytes());
    sign_using_secret(unsigned, &secret)
}
