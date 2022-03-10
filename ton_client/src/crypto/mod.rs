/*
* Copyright 2018-2021 TON Labs LTD.
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

pub(crate) mod boxes;
pub(crate) mod encscrypt;
mod errors;
pub(crate) mod hash;
pub(crate) mod hdkey;
pub(crate) mod internal;
pub(crate) mod keys;
pub(crate) mod math;
pub(crate) mod mnemonic;
pub(crate) mod nacl;

pub use errors::{Error, ErrorCode};
pub(crate) mod encryption;
#[cfg(test)]
mod tests;

pub use crate::crypto::boxes::crypto_box::{
    clear_crypto_box_secret_cache, create_crypto_box, get_crypto_box_info,
    get_crypto_box_seed_phrase, get_encryption_box_from_crypto_box,
    get_signing_box_from_crypto_box, remove_crypto_box, AppPasswordProvider,
    BoxEncryptionAlgorithm, ChaCha20Params, CryptoBoxHandle, CryptoBoxSecret, NaclBoxParams,
    NaclSecretBoxParams, ParamsOfCreateCryptoBox, ParamsOfGetSigningBoxFromCryptoBox,
    RegisteredCryptoBox, ResultOfGetCryptoBoxInfo, ResultOfGetCryptoBoxSeedPhrase,
    ResultOfGetPassword,
};
pub use crate::crypto::boxes::encryption_box::aes::{AesInfo, AesParams};
pub use crate::crypto::boxes::encryption_box::{
    create_encryption_box, encryption_box_decrypt, encryption_box_encrypt, encryption_box_get_info,
    register_encryption_box, remove_encryption_box, CipherMode, EncryptionAlgorithm, EncryptionBox,
    EncryptionBoxHandle, EncryptionBoxInfo, ParamsOfEncryptionBoxDecrypt,
    ParamsOfEncryptionBoxEncrypt, ParamsOfEncryptionBoxGetInfo, RegisteredEncryptionBox,
    ResultOfEncryptionBoxDecrypt, ResultOfEncryptionBoxEncrypt, ResultOfEncryptionBoxGetInfo,
};
pub use crate::crypto::boxes::signing_box::{
    get_signing_box, register_signing_box, remove_signing_box, signing_box_get_public_key,
    signing_box_sign, ParamsOfSigningBoxSign, RegisteredSigningBox, ResultOfSigningBoxGetPublicKey,
    ResultOfSigningBoxSign, SigningBox, SigningBoxHandle,
};
pub use crate::crypto::encscrypt::{scrypt, ParamsOfScrypt, ResultOfScrypt};
pub use crate::crypto::hash::{sha256, sha512, ParamsOfHash, ResultOfHash};
pub use crate::crypto::hdkey::{
    hdkey_derive_from_xprv, hdkey_derive_from_xprv_path, hdkey_public_from_xprv,
    hdkey_secret_from_xprv, hdkey_xprv_from_mnemonic, ParamsOfHDKeyDeriveFromXPrv,
    ParamsOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeyPublicFromXPrv, ParamsOfHDKeySecretFromXPrv,
    ParamsOfHDKeyXPrvFromMnemonic, ResultOfHDKeyDeriveFromXPrv, ResultOfHDKeyDeriveFromXPrvPath,
    ResultOfHDKeyPublicFromXPrv, ResultOfHDKeySecretFromXPrv, ResultOfHDKeyXPrvFromMnemonic,
};
pub use crate::crypto::keys::{
    convert_public_key_to_ton_safe_format, generate_random_sign_keys, sign, verify_signature,
    KeyPair, ParamsOfConvertPublicKeyToTonSafeFormat, ParamsOfSign, ParamsOfVerifySignature,
    ResultOfConvertPublicKeyToTonSafeFormat, ResultOfSign, ResultOfVerifySignature,
};
pub use crate::crypto::math::{
    factorize, generate_random_bytes, modular_power, ton_crc16, ParamsOfFactorize,
    ParamsOfGenerateRandomBytes, ParamsOfModularPower, ParamsOfTonCrc16, ResultOfFactorize,
    ResultOfGenerateRandomBytes, ResultOfModularPower, ResultOfTonCrc16,
};
pub use crate::crypto::mnemonic::{
    mnemonic_derive_sign_keys, mnemonic_from_entropy, mnemonic_from_random, mnemonic_verify,
    mnemonic_words, ParamsOfMnemonicDeriveSignKeys, ParamsOfMnemonicFromEntropy,
    ParamsOfMnemonicFromRandom, ParamsOfMnemonicVerify, ParamsOfMnemonicWords,
    ResultOfMnemonicFromEntropy, ResultOfMnemonicFromRandom, ResultOfMnemonicVerify,
    ResultOfMnemonicWords,
};
pub use crate::crypto::nacl::{
    nacl_box, nacl_box_keypair, nacl_box_keypair_from_secret_key, nacl_box_open, nacl_secret_box,
    nacl_secret_box_open, nacl_sign, nacl_sign_detached, nacl_sign_detached_verify,
    nacl_sign_keypair_from_secret_key, nacl_sign_open, ParamsOfNaclBox,
    ParamsOfNaclBoxKeyPairFromSecret, ParamsOfNaclBoxOpen, ParamsOfNaclSecretBox,
    ParamsOfNaclSecretBoxOpen, ParamsOfNaclSign, ParamsOfNaclSignDetached,
    ParamsOfNaclSignDetachedVerify, ParamsOfNaclSignKeyPairFromSecret, ParamsOfNaclSignOpen,
    ResultOfNaclBox, ResultOfNaclBoxOpen, ResultOfNaclSign, ResultOfNaclSignDetached,
    ResultOfNaclSignDetachedVerify, ResultOfNaclSignOpen,
};
pub use encryption::{chacha20, ParamsOfChaCha20, ResultOfChaCha20};

use serde::{Deserialize, Deserializer};

pub fn default_mnemonic_dictionary() -> u8 {
    1
}

pub fn default_mnemonic_word_count() -> u8 {
    12
}

pub fn default_hdkey_derivation_path() -> String {
    "m/44'/396'/0'/0/0".into()
}

pub fn default_hdkey_compliant() -> bool {
    true
}

fn deserialize_mnemonic_dictionary<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u8, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_mnemonic_dictionary()))
}

fn deserialize_mnemonic_word_count<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u8, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_mnemonic_word_count()))
}

fn deserialize_hdkey_derivation_path<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_hdkey_derivation_path()))
}

#[derive(Deserialize, Debug, Clone, ApiType)]
/// Crypto config.
pub struct CryptoConfig {
    /// Mnemonic dictionary that will be used by default in crypto functions.
    /// If not specified, 1 dictionary will be used.
    #[serde(
        default = "default_mnemonic_dictionary",
        deserialize_with = "deserialize_mnemonic_dictionary"
    )]
    pub mnemonic_dictionary: u8,

    /// Mnemonic word count that will be used by default in crypto functions.
    /// If not specified the default value will be 12.
    #[serde(
        default = "default_mnemonic_word_count",
        deserialize_with = "deserialize_mnemonic_word_count"
    )]
    pub mnemonic_word_count: u8,

    /// Derivation path that will be used by default in crypto functions.
    /// If not specified `m/44'/396'/0'/0/0` will be used.
    #[serde(
        default = "default_hdkey_derivation_path",
        deserialize_with = "deserialize_hdkey_derivation_path"
    )]
    pub hdkey_derivation_path: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            mnemonic_dictionary: default_mnemonic_dictionary(),
            mnemonic_word_count: default_mnemonic_word_count(),
            hdkey_derivation_path: default_hdkey_derivation_path(),
        }
    }
}
