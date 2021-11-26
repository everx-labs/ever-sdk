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
use crate::crypto::internal::{key256, key512, sha256, Key256, Key264};
use crate::crypto::mnemonic::{check_phrase, mnemonics};
use crate::error::{ClientError, ClientResult};
use base58::*;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use hmac::*;
use pbkdf2::pbkdf2;
use libsecp256k1::{SecretKey, PublicKey};
use sha2::{Digest, Sha512};
use crate::crypto::default_hdkey_compliant;

//----------------------------------------------------------------- crypto.hdkey_xprv_from_mnemonic

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfHDKeyXPrvFromMnemonic {
    /// String with seed phrase
    pub phrase: String,
    /// Dictionary identifier
    pub dictionary: Option<u8>,
    /// Mnemonic word count
    pub word_count: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfHDKeyXPrvFromMnemonic {
    /// Serialized extended master private key
    pub xprv: String,
}

/// Generates an extended master private key that will be the root for all the derived keys
#[api_function]
pub fn hdkey_xprv_from_mnemonic(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfHDKeyXPrvFromMnemonic,
) -> ClientResult<ResultOfHDKeyXPrvFromMnemonic> {
    check_phrase(
        &*mnemonics(&context.config.crypto, params.dictionary, params.word_count)?,
        &params.phrase,
    )?;
    Ok(ResultOfHDKeyXPrvFromMnemonic {
        xprv: HDPrivateKey::from_mnemonic(&params.phrase)?.serialize_to_string(),
    })
}

//------------------------------------------------------------------- crypto.hdkey_secret_from_xprv

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfHDKeySecretFromXPrv {
    /// Serialized extended private key
    pub xprv: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfHDKeySecretFromXPrv {
    /// Private key - 64 symbols hex string
    pub secret: String,
}

/// Extracts the private key from the serialized extended private key
#[api_function]
pub fn hdkey_secret_from_xprv(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfHDKeySecretFromXPrv,
) -> ClientResult<ResultOfHDKeySecretFromXPrv> {
    Ok(ResultOfHDKeySecretFromXPrv {
        secret: hex::encode(HDPrivateKey::from_serialized_string(&params.xprv)?.secret()),
    })
}

//------------------------------------------------------------------- crypto.hdkey_public_from_xprv

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfHDKeyPublicFromXPrv {
    /// Serialized extended private key
    pub xprv: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfHDKeyPublicFromXPrv {
    /// Public key - 64 symbols hex string
    pub public: String,
}

/// Extracts the public key from the serialized extended private key
#[api_function]
pub fn hdkey_public_from_xprv(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfHDKeyPublicFromXPrv,
) -> ClientResult<ResultOfHDKeyPublicFromXPrv> {
    let key = HDPrivateKey::from_serialized_string(&params.xprv)?;
    let secret = ed25519_dalek::SecretKey::from_bytes(&key.secret())
        .map_err(|_| crypto::Error::bip32_invalid_key(&params.xprv))?;
    let public = ed25519_dalek::PublicKey::from(&secret);
    Ok(ResultOfHDKeyPublicFromXPrv {
        public: hex::encode(public.as_bytes()),
    })
}

//------------------------------------------------------------ crypto.hdkey_derive_from_xprv

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfHDKeyDeriveFromXPrv {
    /// Serialized extended private key
    pub xprv: String,
    /// Child index (see BIP-0032)
    pub child_index: u32,
    /// Indicates the derivation of hardened/not-hardened key (see BIP-0032)
    pub hardened: bool,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfHDKeyDeriveFromXPrv {
    /// Serialized extended private key
    pub xprv: String,
}

/// Returns extended private key derived from the specified extended private key and child index
#[api_function]
pub fn hdkey_derive_from_xprv(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfHDKeyDeriveFromXPrv,
) -> ClientResult<ResultOfHDKeyDeriveFromXPrv> {
    let xprv = HDPrivateKey::from_serialized_string(&params.xprv)?;
    let derived = xprv.derive(
        params.child_index,
        params.hardened,
        default_hdkey_compliant(),
    )?;
    Ok(ResultOfHDKeyDeriveFromXPrv {
        xprv: derived.serialize_to_string(),
    })
}

//-------------------------------------------------------------- crypto.hdkey_derive_from_xprv_path

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfHDKeyDeriveFromXPrvPath {
    /// Serialized extended private key
    pub xprv: String,
    /// Derivation path, for instance "m/44'/396'/0'/0/0"
    pub path: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfHDKeyDeriveFromXPrvPath {
    /// Derived serialized extended private key
    pub xprv: String,
}

/// Derives the extended private key from the specified key and path
#[api_function]
pub fn hdkey_derive_from_xprv_path(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfHDKeyDeriveFromXPrvPath,
) -> ClientResult<ResultOfHDKeyDeriveFromXPrvPath> {
    let xprv = HDPrivateKey::from_serialized_string(&params.xprv)?;
    Ok(ResultOfHDKeyDeriveFromXPrvPath {
        xprv: xprv
            .derive_path(&params.path, default_hdkey_compliant())?
            .serialize_to_string(),
    })
}

// Internals

#[derive(Default, Clone)]
pub(crate) struct HDPrivateKey {
    depth: u8,
    parent_fingerprint: [u8; 4],
    child_number: [u8; 4],
    child_chain: Key256,
    key: Key256,
}

static XPRV_VERSION: [u8; 4] = [0x04, 0x88, 0xAD, 0xE4];

impl HDPrivateKey {
    pub(crate) fn master(child_chain: &Key256, key: &Key256) -> HDPrivateKey {
        HDPrivateKey {
            depth: 0,
            parent_fingerprint: [0; 4],
            child_number: [0; 4],
            child_chain: *child_chain,
            key: *key,
        }
    }

    pub(crate) fn from_mnemonic(phrase: &String) -> ClientResult<HDPrivateKey> {
        let salt = "mnemonic";
        let mut seed = vec![0u8; 64];
        pbkdf2::<Hmac<Sha512>>(phrase.as_bytes(), salt.as_bytes(), 2048, &mut seed);
        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(b"Bitcoin seed").unwrap();
        hmac.update(&seed);
        let child_chain_with_key = key512(&hmac.finalize().into_bytes())?;
        Ok(HDPrivateKey::master(
            &key256(&child_chain_with_key[32..])?,
            &key256(&child_chain_with_key[..32])?,
        ))
    }

    pub(crate) fn secret(&self) -> Key256 {
        self.key
    }

    fn public(&self) -> Key264 {
        let secret_key = SecretKey::parse(&self.key).unwrap();
        let public_key = PublicKey::from_secret_key(&secret_key);
        public_key.serialize_compressed()
    }

    fn map_secp_error(error: libsecp256k1::Error) -> ClientError {
        match error {
            libsecp256k1::Error::InvalidSignature => {
                crypto::Error::bip32_invalid_key("InvalidSignature")
            }
            libsecp256k1::Error::InvalidPublicKey => {
                crypto::Error::bip32_invalid_key("InvalidPublicKey")
            }
            libsecp256k1::Error::InvalidSecretKey => {
                crypto::Error::bip32_invalid_key("InvalidSecretKey")
            }
            libsecp256k1::Error::InvalidRecoveryId => {
                crypto::Error::bip32_invalid_key("InvalidRecoveryId")
            }
            libsecp256k1::Error::InvalidMessage => crypto::Error::bip32_invalid_key("InvalidMessage"),
            libsecp256k1::Error::InvalidInputLength => {
                crypto::Error::bip32_invalid_key("InvalidInputLength")
            }
            libsecp256k1::Error::TweakOutOfRange => {
                crypto::Error::bip32_invalid_key("TweakOutOfRange")
            }
        }
    }

    pub(crate) fn derive(
        &self,
        child_index: u32,
        hardened: bool,
        compliant: bool,
    ) -> ClientResult<HDPrivateKey> {
        let mut child: HDPrivateKey = Default::default();
        child.depth = self.depth + 1;

        let public = self.public();
        let mut sha_hasher = sha2::Sha256::new();
        sha_hasher.update(&public.as_ref());
        let sha: Key256 = sha_hasher.finalize().into();
        let fingerprint = Ripemd160::new().update(&sha).digest();

        child.parent_fingerprint.copy_from_slice(&fingerprint[0..4]);

        let child_index = if hardened {
            0x80000000 | child_index
        } else {
            child_index
        };
        BigEndian::write_u32(&mut child.child_number, child_index);

        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.child_chain)
            .map_err(|err| crypto::Error::bip32_invalid_key(err))?;

        let secret_key = SecretKey::parse(&self.key).unwrap();
        if hardened && !compliant {
            // The private key serialization in this case will not be exactly 32 bytes and can be
            // any smaller value, and the value is not zero-padded.
            hmac.update(&[0]);
            hmac.update(&secret_key.serialize());
        } else if hardened {
            // This will use a 32 byte zero padded serialization of the private key
            hmac.update(&[0]);
            hmac.update(&secret_key.serialize());
        } else {
            hmac.update(&public);
        }
        hmac.update(&child.child_number);
        let result = hmac.finalize().into_bytes();
        let (child_key_bytes, chain_code) = result.split_at(32);

        let mut child_secret_key =
            SecretKey::parse_slice(&child_key_bytes).map_err(|err| Self::map_secp_error(err))?;
        let self_secret_key =
            SecretKey::parse(&self.key).map_err(|err| Self::map_secp_error(err))?;
        child_secret_key
            .tweak_add_assign(&self_secret_key)
            .map_err(|err| Self::map_secp_error(err))?;

        child.child_chain.copy_from_slice(&chain_code);
        child.key.copy_from_slice(&child_secret_key.serialize());
        Ok(child)
    }

    pub(crate) fn derive_path(&self, path: &String, compliant: bool) -> ClientResult<HDPrivateKey> {
        let mut child: HDPrivateKey = self.clone();
        for step in path.split("/") {
            if step == "m" {
            } else {
                let hardened = step.ends_with('\'');
                let index: u32 = (if hardened {
                    &step[0..(step.len() - 1)]
                } else {
                    step
                })
                .parse()
                .map_err(|_| crypto::Error::bip32_invalid_derive_path(path))?;
                child = child.derive(index, hardened, compliant)?;
            }
        }
        Ok(child)
    }

    // Serialization

    fn from_serialized(bytes: &[u8]) -> ClientResult<HDPrivateKey> {
        if bytes.len() != 82 {
            return Err(crypto::Error::bip32_invalid_key(bytes.to_base58()));
        }
        let mut version = [0u8; 4];
        version.clone_from_slice(&bytes[0..4]);
        if version != XPRV_VERSION {
            return Err(crypto::Error::bip32_invalid_key(bytes.to_base58()));
        }
        let mut xprv: HDPrivateKey = Default::default();
        xprv.depth = bytes[4];
        xprv.parent_fingerprint.copy_from_slice(&bytes[5..9]);
        xprv.child_number.copy_from_slice(&bytes[9..13]);
        xprv.child_chain.copy_from_slice(&bytes[13..45]);
        if bytes[45] != 0 {
            return Err(crypto::Error::bip32_invalid_key(bytes.to_base58()));
        }
        xprv.key.copy_from_slice(&bytes[46..78]);
        Ok(xprv)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(&XPRV_VERSION);
        bytes.push(self.depth);
        bytes.extend(&self.parent_fingerprint);
        bytes.extend(&self.child_number);
        bytes.extend(&self.child_chain);
        bytes.push(0);
        bytes.extend(&self.key);
        bytes.extend(&sha256(&sha256(&bytes))[0..4]);
        bytes
    }

    fn from_serialized_string(string: &String) -> ClientResult<HDPrivateKey> {
        Self::from_serialized(
            &string
                .from_base58()
                .map_err(|_| crypto::Error::bip32_invalid_key(string))?,
        )
    }

    fn serialize_to_string(&self) -> String {
        self.serialize().to_base58()
    }
}

struct Ripemd160 {
    pending: Vec<u8>,
    pending_total: usize,
    pad_length: usize,
    _delta8: usize,
    _delta32: usize,

    h: [u32; 5],
}

impl Ripemd160 {
    fn new() -> Ripemd160 {
        Ripemd160 {
            h: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            pending: Vec::new(),
            pending_total: 0,
            pad_length: RIPEMD160_PAD_LENGTH / 8,
            _delta8: RIPEMD160_BLOCK_SIZE / 8,
            _delta32: RIPEMD160_BLOCK_SIZE / 32,
        }
    }

    fn join32(msg: &[u8]) -> Vec<u32> {
        assert_eq!(msg.len() % 4, 0usize);
        let mut res: Vec<u32> = Vec::new();
        res.resize(msg.len() / 4, 0);
        for i in 0..res.len() {
            res[i] = LittleEndian::read_u32(&msg[i * 4..(i + 1) * 4]);
        }
        res
    }

    fn split32(msg: &[u32]) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        res.resize(msg.len() * 4, 0);
        for i in 0..msg.len() {
            LittleEndian::write_u32(&mut res[i * 4..(i + 1) * 4], msg[i]);
        }
        res
    }

    fn update(&mut self, msg: &[u8]) -> &mut Self {
        self.pending.extend_from_slice(msg);
        self.pending_total += msg.len();
        if self.pending.len() >= self._delta8 {
            let msg = self.pending.split_off(self.pending.len() % self._delta8);
            let msg = Ripemd160::join32(&msg);
            let mut i = 0;
            while i < msg.len() {
                self._update(&msg[i..(i + self._delta32)]);
                i += self._delta32;
            }
        }
        self
    }

    fn digest(&mut self) -> Vec<u8> {
        self.update(&self._pad());
        assert_eq!(self.pending.len(), 0);
        self._digest()
    }

    fn rotl32(w: u32, b: u32) -> u32 {
        w.rotate_left(b)
    }

    fn sum32(a: u32, b: u32) -> u32 {
        a.wrapping_add(b)
    }
    fn sum32_3(a: u32, b: u32, c: u32) -> u32 {
        a.wrapping_add(b).wrapping_add(c)
    }

    fn sum32_4(a: u32, b: u32, c: u32, d: u32) -> u32 {
        a.wrapping_add(b).wrapping_add(c).wrapping_add(d)
    }

    fn _update(&mut self, msg: &[u32]) {
        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];
        let mut ah = a;
        let mut bh = b;
        let mut ch = c;
        let mut dh = d;
        let mut eh = e;
        let start: u32 = 0;
        for j in 0u32..80 {
            let mut t = Ripemd160::sum32(
                Ripemd160::rotl32(
                    Ripemd160::sum32_4(
                        a,
                        Ripemd160::f(j.into(), b, c, d),
                        msg[(RIPEMD160_R[j as usize] as u32 + start) as usize],
                        Ripemd160::k(j),
                    ),
                    RIPEMD160_S[j as usize].into(),
                ),
                e,
            );
            a = e;
            e = d;
            d = Ripemd160::rotl32(c, 10);
            c = b;
            b = t;
            t = Ripemd160::sum32(
                Ripemd160::rotl32(
                    Ripemd160::sum32_4(
                        ah,
                        Ripemd160::f(79 - j, bh, ch, dh),
                        msg[(RIPEMD160_RH[j as usize] as u32 + start) as usize],
                        Ripemd160::kh(j),
                    ),
                    RIPEMD160_SH[j as usize].into(),
                ),
                eh,
            );
            ah = eh;
            eh = dh;
            dh = Ripemd160::rotl32(ch, 10);
            ch = bh;
            bh = t;
        }
        let t = Ripemd160::sum32_3(self.h[1], c, dh);
        self.h[1] = Ripemd160::sum32_3(self.h[2], d, eh);
        self.h[2] = Ripemd160::sum32_3(self.h[3], e, ah);
        self.h[3] = Ripemd160::sum32_3(self.h[4], a, bh);
        self.h[4] = Ripemd160::sum32_3(self.h[0], b, ch);
        self.h[0] = t;
    }

    fn _digest(&self) -> Vec<u8> {
        Ripemd160::split32(&self.h)
    }

    fn _pad(&self) -> Vec<u8> {
        let len = self.pending_total;
        let bytes = self._delta8;
        let k = bytes - ((len + self.pad_length) % bytes);
        let mut res: Vec<u8> = Vec::new();
        res.resize(k + self.pad_length, 0);
        res[0] = 0x80;
        LittleEndian::write_u32(&mut res[k..(k + 4)], (len as u32) << 3);
        res
    }

    fn f(j: u32, x: u32, y: u32, z: u32) -> u32 {
        if j <= 15 {
            x ^ y ^ z
        } else if j <= 31 {
            (x & y) | ((!x) & z)
        } else if j <= 47 {
            (x | (!y)) ^ z
        } else if j <= 63 {
            (x & z) | (y & (!z))
        } else {
            x ^ (y | (!z))
        }
    }

    fn k(j: u32) -> u32 {
        if j <= 15 {
            0x00000000
        } else if j <= 31 {
            0x5a827999
        } else if j <= 47 {
            0x6ed9eba1
        } else if j <= 63 {
            0x8f1bbcdc
        } else {
            0xa953fd4e
        }
    }

    fn kh(j: u32) -> u32 {
        if j <= 15 {
            0x50a28be6
        } else if j <= 31 {
            0x5c4dd124
        } else if j <= 47 {
            0x6d703ef3
        } else if j <= 63 {
            0x7a6d76e9
        } else {
            0x00000000
        }
    }
}

static RIPEMD160_R: [u8; 80] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5,
    2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8, 12, 4,
    13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
];

static RIPEMD160_RH: [u8; 80] = [
    5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12,
    4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11, 15, 0, 5,
    12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
];

static RIPEMD160_S: [u8; 80] = [
    11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15,
    9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14, 15, 14,
    15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6,
];

static RIPEMD160_SH: [u8; 80] = [
    8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12,
    7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11, 14, 14,
    6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11,
];

static RIPEMD160_BLOCK_SIZE: usize = 512;
static RIPEMD160_PAD_LENGTH: usize = 64;
