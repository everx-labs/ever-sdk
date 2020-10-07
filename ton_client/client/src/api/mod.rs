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
 *
 */

pub mod api_reference;
pub(crate) mod dispatch;
pub(crate) mod interop;
mod net;
pub(crate) mod processing;

use crate::abi::{
    attach_signature, decode_message, encode_message, Abi, AbiHandle, CallSet, DeploySet,
    FunctionHeader,
};
use crate::api::api_reference::get_api_reference;
use crate::api::dispatch::{ApiDispatcher, ModuleReg, Registrar};
use crate::boc::{
    get_blockchain_config, parse_account, parse_block, parse_message, parse_transaction,
};
use crate::crypto::{
    convert_public_key_to_ton_safe_format, factorize, generate_random_bytes,
    generate_random_sign_keys, hdkey_derive_from_xprv, hdkey_derive_from_xprv_path,
    hdkey_public_from_xprv, hdkey_secret_from_xprv, hdkey_xprv_from_mnemonic,
    mnemonic_derive_sign_keys, mnemonic_from_entropy, mnemonic_from_random, mnemonic_verify,
    mnemonic_words, modular_power, nacl_box, nacl_box_keypair, nacl_box_keypair_from_secret_key,
    nacl_box_open, nacl_secret_box, nacl_secret_box_open, nacl_sign, nacl_sign_detached,
    nacl_sign_keypair_from_secret_key, nacl_sign_open, sha256, sha512, sign, ton_crc16,
    verify_signature,
};
use crate::net::{query_collection, unsubscribe, wait_for_collection};
use crate::processing::{DecodedOutput, MessageSource, ProcessingEvent, ResultOfProcessMessage};
use crate::tvm::{execute_get, execute_message};
use crate::utils::{convert_address, AddressStringFormat};

lazy_static! {
    static ref DISPATCHER: ApiDispatcher = create_dispatcher();
}

pub(crate) fn get_dispatcher() -> &'static ApiDispatcher {
    &DISPATCHER
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "client")]
pub(crate) struct ClientModule;

impl ModuleReg for ClientModule {
    fn reg(reg: &mut Registrar) {
        reg.f_no_args(
            get_api_reference,
            crate::api::api_reference::get_api_reference_api,
        );
        reg.f_no_args(crate::client::version, crate::client::version_api);
    }
}

/// Crypto functions.
#[derive(ApiModule)]
#[api_module(name = "crypto")]
pub(crate) struct CryptoModule;

impl ModuleReg for CryptoModule {
    fn reg(reg: &mut Registrar) {
        // Math

        reg.f(factorize, crate::crypto::math::factorize_api);
        reg.f(modular_power, crate::crypto::math::modular_power_api);
        reg.f(ton_crc16, crate::crypto::math::ton_crc16_api);
        reg.f(
            generate_random_bytes,
            crate::crypto::math::generate_random_bytes_api,
        );

        // Keys

        reg.f(
            convert_public_key_to_ton_safe_format,
            crate::crypto::keys::convert_public_key_to_ton_safe_format_api,
        );

        reg.f_no_args(
            generate_random_sign_keys,
            crate::crypto::keys::generate_random_sign_keys_api,
        );
        reg.f(sign, crate::crypto::keys::sign_api);
        reg.f(verify_signature, crate::crypto::keys::verify_signature_api);

        // Sha

        reg.f(sha256, crate::crypto::hash::sha256_api);
        reg.f(sha512, crate::crypto::hash::sha512_api);

        // Scrypt

        reg.f(
            crate::crypto::scrypt::scrypt,
            crate::crypto::scrypt::scrypt_api,
        );

        // NaCl

        reg.f(
            nacl_sign_keypair_from_secret_key,
            crate::crypto::nacl::nacl_sign_keypair_from_secret_key_api,
        );
        reg.f(nacl_sign, crate::crypto::nacl::nacl_sign_api);
        reg.f(nacl_sign_open, crate::crypto::nacl::nacl_sign_open_api);
        reg.f(
            nacl_sign_detached,
            crate::crypto::nacl::nacl_sign_detached_api,
        );

        reg.f_no_args(nacl_box_keypair, crate::crypto::nacl::nacl_box_keypair_api);
        reg.f(
            nacl_box_keypair_from_secret_key,
            crate::crypto::nacl::nacl_box_keypair_from_secret_key_api,
        );
        reg.f(nacl_box, crate::crypto::nacl::nacl_box_api);
        reg.f(nacl_box_open, crate::crypto::nacl::nacl_box_open_api);
        reg.f(nacl_secret_box, crate::crypto::nacl::nacl_secret_box_api);
        reg.f(
            nacl_secret_box_open,
            crate::crypto::nacl::nacl_secret_box_open_api,
        );

        // Mnemonic

        reg.f(mnemonic_words, crate::crypto::mnemonic::mnemonic_words_api);
        reg.f(
            mnemonic_from_random,
            crate::crypto::mnemonic::mnemonic_from_random_api,
        );
        reg.f(
            mnemonic_from_entropy,
            crate::crypto::mnemonic::mnemonic_from_entropy_api,
        );
        reg.f(
            mnemonic_verify,
            crate::crypto::mnemonic::mnemonic_verify_api,
        );
        reg.f(
            mnemonic_derive_sign_keys,
            crate::crypto::mnemonic::mnemonic_derive_sign_keys_api,
        );

        // HDKey

        reg.f(
            hdkey_xprv_from_mnemonic,
            crate::crypto::hdkey::hdkey_xprv_from_mnemonic_api,
        );
        reg.f(
            hdkey_derive_from_xprv,
            crate::crypto::hdkey::hdkey_derive_from_xprv_api,
        );
        reg.f(
            hdkey_derive_from_xprv_path,
            crate::crypto::hdkey::hdkey_derive_from_xprv_path_api,
        );
        reg.f(
            hdkey_secret_from_xprv,
            crate::crypto::hdkey::hdkey_secret_from_xprv_api,
        );
        reg.f(
            hdkey_public_from_xprv,
            crate::crypto::hdkey::hdkey_public_from_xprv_api,
        );
    }
}

/// Functions for encoding and decoding messages due to ABI
/// specification.
#[derive(ApiModule)]
#[api_module(name = "abi")]
pub(crate) struct AbiModule;

impl ModuleReg for AbiModule {
    fn reg(reg: &mut Registrar) {
        reg.t::<Abi>();
        reg.t::<AbiHandle>();
        reg.t::<FunctionHeader>();
        reg.t::<CallSet>();
        reg.t::<DeploySet>();

        reg.async_f(encode_message, crate::abi::encode::encode_message_api);
        reg.f(attach_signature, crate::abi::encode::attach_signature_api);
        reg.f(decode_message, crate::abi::decode::decode_message_api);
    }
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "boc")]
pub(crate) struct BocModule;
impl ModuleReg for BocModule {
    fn reg(reg: &mut Registrar) {
        reg.f(parse_message, crate::boc::parse::parse_message_api);
        reg.f(parse_transaction, crate::boc::parse::parse_transaction_api);
        reg.f(parse_account, crate::boc::parse::parse_account_api);
        reg.f(parse_block, crate::boc::parse::parse_block_api);
        reg.f(
            get_blockchain_config,
            crate::boc::blockchain_config::get_blockchain_config_api,
        );
    }
}

/// Network access.
#[derive(ApiModule)]
#[api_module(name = "net")]
pub(crate) struct NetModule;

impl ModuleReg for NetModule {
    fn reg(reg: &mut Registrar) {
        reg.async_f(query_collection, crate::net::query_collection_api);
        reg.async_f(wait_for_collection, crate::net::wait_for_collection_api);
        reg.async_f(unsubscribe, crate::net::unsubscribe_api);
        reg.async_f_callback(
            crate::api::net::subscribe_collection,
            crate::api::net::subscribe_collection_api,
        );
    }
}

/// Message processing module.
///
/// This module incorporates functions related to complex message
/// processing scenarios.
#[derive(ApiModule)]
#[api_module(name = "processing")]
pub struct ProcessingModule;

impl ModuleReg for ProcessingModule {
    fn reg(reg: &mut Registrar) {
        reg.t::<MessageSource>();
        reg.t::<ProcessingEvent>();
        reg.t::<ResultOfProcessMessage>();
        reg.t::<DecodedOutput>();

        reg.async_f_callback(
            crate::api::processing::send_message,
            crate::api::processing::send_message_api,
        );
        reg.async_f_callback(
            crate::api::processing::wait_for_transaction,
            crate::api::processing::wait_for_transaction_api,
        );
        reg.async_f_callback(
            crate::api::processing::process_message,
            crate::api::processing::process_message_api,
        );
    }
}

#[derive(ApiModule)]
#[api_module(name = "tvm")]
pub struct TvmModule;

impl ModuleReg for TvmModule {
    fn reg(reg: &mut Registrar) {
        reg.async_f(
            execute_message,
            crate::tvm::execute_message::execute_message_api,
        );
        reg.f(execute_get, crate::tvm::execute_get::execute_get_api);
    }
}

/// Misc utility Functions.
#[derive(ApiModule)]
#[api_module(name = "utils")]
pub struct UtilsModule;

impl ModuleReg for UtilsModule {
    fn reg(reg: &mut Registrar) {
        reg.t::<AddressStringFormat>();
        reg.f(
            convert_address,
            crate::utils::conversion::convert_address_api,
        );
    }
}

pub(crate) fn create_dispatcher() -> ApiDispatcher {
    let mut handlers = ApiDispatcher::new();
    handlers.register::<ClientModule>();
    handlers.register::<CryptoModule>();
    handlers.register::<AbiModule>();
    handlers.register::<BocModule>();
    handlers.register::<ProcessingModule>();
    handlers.register::<UtilsModule>();
    handlers.register::<TvmModule>();

    #[cfg(feature = "node_interaction")]
    handlers.register::<NetModule>();

    handlers
}
