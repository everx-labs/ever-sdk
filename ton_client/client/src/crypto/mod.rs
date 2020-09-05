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

pub mod math;
pub mod hash;
pub mod scrypt;
pub mod nacl;
pub mod keys;
pub mod mnemonic;
pub mod hdkey;
pub mod boxes;
pub(crate) mod internal;

#[cfg(test)]
mod tests;

use crate::dispatch::DispatchTable;
use crate::crypto::math::{factorize, modular_power, ton_crc16, generate_random_bytes};
use crate::crypto::keys::{convert_public_key_to_ton_safe_format, generate_random_sign_keys, verify_signature, sign};
use crate::crypto::hash::{sha256, sha512};
use crate::crypto::nacl::{
    nacl_sign, nacl_sign_open, nacl_sign_keypair_from_secret_key,
    nacl_sign_detached, nacl_box_keypair, nacl_box_keypair_from_secret_key, nacl_box,
    nacl_box_open, nacl_secret_box, nacl_secret_box_open,
};
use crate::crypto::mnemonic::{
    mnemonic_words, mnemonic_from_random, mnemonic_from_entropy, mnemonic_verify,
    mnemonic_derive_sign_keys,
};
use crate::crypto::hdkey::{
    hdkey_xprv_from_mnemonic, hdkey_derive_from_xprv, hdkey_derive_from_xprv_path,
    hdkey_secret_from_xprv, hdkey_public_from_xprv,
};

pub(crate) const DEFAULT_MNEMONIC_DICTIONARY: u8 = 1;
pub(crate) const DEFAULT_MNEMONIC_WORD_COUNT: u8 = 12;
pub(crate) const DEFAULT_HDKEY_DERIVATION_PATH: &str = "m/44'/396'/0'/0/0";
pub(crate) const DEFAULT_HDKEY_COMPLIANT: bool = true;

pub(crate) fn register(handlers: &mut DispatchTable) {

    // Math

    handlers.spawn("crypto.factorize", factorize);
    handlers.spawn("crypto.modular_power", modular_power);
    handlers.spawn("crypto.ton_crc16", ton_crc16);
    handlers.call("crypto.generate_random_bytes", generate_random_bytes);

    // Keys

    handlers.spawn(
        "crypto.convert_public_key_to_ton_safe_format",
        convert_public_key_to_ton_safe_format,
    );

    handlers.call_no_args("crypto.generate_random_sign_keys", generate_random_sign_keys);
    handlers.spawn("crypto.sign", sign);
    handlers.spawn("crypto.verify_signature", verify_signature);

    // Sha

    handlers.spawn("crypto.sha256", sha256);
    handlers.spawn("crypto.sha512", sha512);

    // Scrypt

    handlers.spawn("crypto.scrypt", scrypt::scrypt);

    // NaCl

    handlers.call("crypto.nacl_sign_keypair_from_secret", nacl_sign_keypair_from_secret_key);
    handlers.spawn("crypto.nacl_sign", nacl_sign);
    handlers.spawn("crypto.nacl_sign_open", nacl_sign_open);
    handlers.spawn("crypto.nacl_sign_detached", nacl_sign_detached);

    handlers.call_no_args("crypto.nacl_box_keypair", nacl_box_keypair);
    handlers.call("crypto.nacl_box_keypair_from_secret", nacl_box_keypair_from_secret_key);
    handlers.spawn("crypto.nacl_box", nacl_box);
    handlers.spawn("crypto.nacl_box_open", nacl_box_open);
    handlers.spawn("crypto.nacl_secret_box", nacl_secret_box);
    handlers.spawn("crypto.nacl_secret_box_open", nacl_secret_box_open);

    // Mnemonic

    handlers.spawn("crypto.mnemonic_words", mnemonic_words);
    handlers.spawn("crypto.mnemonic_from_random", mnemonic_from_random);
    handlers.spawn("crypto.mnemonic_from_entropy", mnemonic_from_entropy);
    handlers.spawn("crypto.mnemonic_verify", mnemonic_verify);
    handlers.spawn("crypto.mnemonic_derive_sign_keys", mnemonic_derive_sign_keys);

    // HDKey

    handlers.spawn("crypto.hdkey_xprv_from_mnemonic", hdkey_xprv_from_mnemonic);
    handlers.spawn("crypto.hdkey_derive_from_xprv", hdkey_derive_from_xprv);
    handlers.spawn("crypto.hdkey_derive_from_xprv_path", hdkey_derive_from_xprv_path);
    handlers.spawn("crypto.hdkey_secret_from_xprv", hdkey_secret_from_xprv);
    handlers.spawn("crypto.hdkey_public_from_xprv", hdkey_public_from_xprv);
}
