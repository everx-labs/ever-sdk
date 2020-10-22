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

mod boxes;
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
#[cfg(test)]
mod tests;

pub use crate::crypto::boxes::SigningBoxHandle;
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
    nacl_secret_box_open, nacl_sign, nacl_sign_detached, nacl_sign_keypair_from_secret_key,
    nacl_sign_open, ParamsOfNaclBox, ParamsOfNaclBoxKeyPairFromSecret, ParamsOfNaclBoxOpen,
    ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen, ParamsOfNaclSign, ParamsOfNaclSignDetached,
    ParamsOfNaclSignKeyPairFromSecret, ParamsOfNaclSignOpen, ResultOfNaclBox, ResultOfNaclBoxOpen,
    ResultOfNaclSign, ResultOfNaclSignDetached, ResultOfNaclSignOpen,
};
pub(crate) const DEFAULT_MNEMONIC_DICTIONARY: u8 = 1;
pub(crate) const DEFAULT_MNEMONIC_WORD_COUNT: u8 = 12;
pub(crate) const DEFAULT_HDKEY_DERIVATION_PATH: &str = "m/44'/396'/0'/0/0";
pub(crate) const DEFAULT_HDKEY_COMPLIANT: bool = true;
