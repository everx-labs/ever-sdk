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
use crate::api::dispatch::{ApiDispatcher, ModuleReg, Registrar};

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
        reg.api_sync_fn_without_args(
            crate::api::api_reference::get_api_reference,
            crate::api::api_reference::get_api_reference_api,
        );
        reg.api_sync_fn_without_args(crate::client::version, crate::client::version_api);
    }
}

/// Crypto functions.
#[derive(ApiModule)]
#[api_module(name = "crypto")]
pub(crate) struct CryptoModule;

impl ModuleReg for CryptoModule {
    fn reg(reg: &mut Registrar) {
        // Math

        reg.api_sync_fn(crate::crypto::factorize, crate::crypto::math::factorize_api);
        reg.api_sync_fn(
            crate::crypto::modular_power,
            crate::crypto::math::modular_power_api,
        );
        reg.api_sync_fn(crate::crypto::ton_crc16, crate::crypto::math::ton_crc16_api);
        reg.api_sync_fn(
            crate::crypto::generate_random_bytes,
            crate::crypto::math::generate_random_bytes_api,
        );

        // Keys

        reg.api_sync_fn(
            crate::crypto::convert_public_key_to_ton_safe_format,
            crate::crypto::keys::convert_public_key_to_ton_safe_format_api,
        );

        reg.api_sync_fn_without_args(
            crate::crypto::generate_random_sign_keys,
            crate::crypto::keys::generate_random_sign_keys_api,
        );
        reg.api_sync_fn(crate::crypto::sign, crate::crypto::keys::sign_api);
        reg.api_sync_fn(
            crate::crypto::verify_signature,
            crate::crypto::keys::verify_signature_api,
        );

        // Sha

        reg.api_sync_fn(crate::crypto::sha256, crate::crypto::hash::sha256_api);
        reg.api_sync_fn(crate::crypto::sha512, crate::crypto::hash::sha512_api);

        // Scrypt

        reg.api_sync_fn(
            crate::crypto::scrypt::scrypt,
            crate::crypto::scrypt::scrypt_api,
        );

        // NaCl

        reg.api_sync_fn(
            crate::crypto::nacl_sign_keypair_from_secret_key,
            crate::crypto::nacl::nacl_sign_keypair_from_secret_key_api,
        );
        reg.api_sync_fn(crate::crypto::nacl_sign, crate::crypto::nacl::nacl_sign_api);
        reg.api_sync_fn(
            crate::crypto::nacl_sign_open,
            crate::crypto::nacl::nacl_sign_open_api,
        );
        reg.api_sync_fn(
            crate::crypto::nacl_sign_detached,
            crate::crypto::nacl::nacl_sign_detached_api,
        );

        reg.api_sync_fn_without_args(
            crate::crypto::nacl_box_keypair,
            crate::crypto::nacl::nacl_box_keypair_api,
        );
        reg.api_sync_fn(
            crate::crypto::nacl_box_keypair_from_secret_key,
            crate::crypto::nacl::nacl_box_keypair_from_secret_key_api,
        );
        reg.api_sync_fn(crate::crypto::nacl_box, crate::crypto::nacl::nacl_box_api);
        reg.api_sync_fn(
            crate::crypto::nacl_box_open,
            crate::crypto::nacl::nacl_box_open_api,
        );
        reg.api_sync_fn(
            crate::crypto::nacl_secret_box,
            crate::crypto::nacl::nacl_secret_box_api,
        );
        reg.api_sync_fn(
            crate::crypto::nacl_secret_box_open,
            crate::crypto::nacl::nacl_secret_box_open_api,
        );

        // Mnemonic

        reg.api_sync_fn(
            crate::crypto::mnemonic_words,
            crate::crypto::mnemonic::mnemonic_words_api,
        );
        reg.api_sync_fn(
            crate::crypto::mnemonic_from_random,
            crate::crypto::mnemonic::mnemonic_from_random_api,
        );
        reg.api_sync_fn(
            crate::crypto::mnemonic_from_entropy,
            crate::crypto::mnemonic::mnemonic_from_entropy_api,
        );
        reg.api_sync_fn(
            crate::crypto::mnemonic_verify,
            crate::crypto::mnemonic::mnemonic_verify_api,
        );
        reg.api_sync_fn(
            crate::crypto::mnemonic_derive_sign_keys,
            crate::crypto::mnemonic::mnemonic_derive_sign_keys_api,
        );

        // HDKey

        reg.api_sync_fn(
            crate::crypto::hdkey_xprv_from_mnemonic,
            crate::crypto::hdkey::hdkey_xprv_from_mnemonic_api,
        );
        reg.api_sync_fn(
            crate::crypto::hdkey_derive_from_xprv,
            crate::crypto::hdkey::hdkey_derive_from_xprv_api,
        );
        reg.api_sync_fn(
            crate::crypto::hdkey_derive_from_xprv_path,
            crate::crypto::hdkey::hdkey_derive_from_xprv_path_api,
        );
        reg.api_sync_fn(
            crate::crypto::hdkey_secret_from_xprv,
            crate::crypto::hdkey::hdkey_secret_from_xprv_api,
        );
        reg.api_sync_fn(
            crate::crypto::hdkey_public_from_xprv,
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
        reg.api_type::<crate::abi::Abi>();
        reg.api_type::<crate::abi::AbiHandle>();
        reg.api_type::<crate::abi::FunctionHeader>();
        reg.api_type::<crate::abi::CallSet>();
        reg.api_type::<crate::abi::DeploySet>();

        reg.api_async_fn(
            crate::abi::encode_message,
            crate::abi::encode::encode_message_api,
        );
        reg.api_sync_fn(
            crate::abi::attach_signature,
            crate::abi::encode::attach_signature_api,
        );
        reg.api_sync_fn(
            crate::abi::decode_message,
            crate::abi::decode::decode_message_api,
        );
    }
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "boc")]
pub(crate) struct BocModule;
impl ModuleReg for BocModule {
    fn reg(reg: &mut Registrar) {
        reg.api_sync_fn(
            crate::boc::parse_message,
            crate::boc::parse::parse_message_api,
        );
        reg.api_sync_fn(
            crate::boc::parse_transaction,
            crate::boc::parse::parse_transaction_api,
        );
        reg.api_sync_fn(
            crate::boc::parse_account,
            crate::boc::parse::parse_account_api,
        );
        reg.api_sync_fn(crate::boc::parse_block, crate::boc::parse::parse_block_api);
        reg.api_sync_fn(
            crate::boc::get_blockchain_config,
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
        reg.api_async_fn(
            crate::net::query_collection,
            crate::net::query_collection_api,
        );
        reg.api_async_fn(
            crate::net::wait_for_collection,
            crate::net::wait_for_collection_api,
        );
        reg.api_async_fn(crate::net::unsubscribe, crate::net::unsubscribe_api);
        reg.api_async_fn_with_callback(
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
        reg.api_type::<crate::processing::MessageSource>();
        reg.api_type::<crate::processing::ProcessingEvent>();
        reg.api_type::<crate::processing::ResultOfProcessMessage>();
        reg.api_type::<crate::processing::DecodedOutput>();

        reg.api_async_fn_with_callback(
            crate::api::processing::send_message,
            crate::api::processing::send_message_api,
        );
        reg.api_async_fn_with_callback(
            crate::api::processing::wait_for_transaction,
            crate::api::processing::wait_for_transaction_api,
        );
        reg.api_async_fn_with_callback(
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
        reg.api_async_fn(
            crate::tvm::execute_message,
            crate::tvm::execute_message::execute_message_api,
        );
        reg.api_sync_fn(
            crate::tvm::execute_get,
            crate::tvm::execute_get::execute_get_api,
        );
    }
}

/// Misc utility Functions.
#[derive(ApiModule)]
#[api_module(name = "utils")]
pub struct UtilsModule;

impl ModuleReg for UtilsModule {
    fn reg(reg: &mut Registrar) {
        reg.api_type::<crate::utils::AddressStringFormat>();
        reg.api_sync_fn(
            crate::utils::convert_address,
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
