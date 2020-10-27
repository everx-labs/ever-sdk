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

use super::registrar::ModuleReg;
use super::runtime::RuntimeHandlers;

/// Provides information about library.
#[derive(ApiModule)]
#[api_module(name = "client")]
pub(crate) struct ClientModule;

fn register_client(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<ClientModule>(handlers);
    module.register_type::<crate::error::ClientError>();
    module.register_type::<crate::client::ClientConfig>();
    module.register_type::<crate::net::NetworkConfig>();
    module.register_type::<crate::crypto::CryptoConfig>();
    module.register_type::<crate::abi::AbiConfig>();

    module.register_sync_fn_without_args(
        crate::client::get_api_reference,
        crate::client::get_api_reference_api,
    );
    module.register_sync_fn_without_args(crate::client::version, crate::client::version_api);
    module.register_sync_fn_without_args(crate::client::build_info, crate::client::build_info_api);
    module.register();
}

/// Crypto functions.
#[derive(ApiModule)]
#[api_module(name = "crypto")]
pub(crate) struct CryptoModule;

fn register_crypto(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<CryptoModule>(handlers);

    module.register_type::<crate::crypto::SigningBoxHandle>();

    // Math

    module.register_sync_fn(crate::crypto::factorize, crate::crypto::math::factorize_api);
    module.register_sync_fn(
        crate::crypto::modular_power,
        crate::crypto::math::modular_power_api,
    );
    module.register_sync_fn(crate::crypto::ton_crc16, crate::crypto::math::ton_crc16_api);
    module.register_sync_fn(
        crate::crypto::generate_random_bytes,
        crate::crypto::math::generate_random_bytes_api,
    );

    // Keys

    module.register_sync_fn(
        crate::crypto::convert_public_key_to_ton_safe_format,
        crate::crypto::keys::convert_public_key_to_ton_safe_format_api,
    );

    module.register_sync_fn_without_args(
        crate::crypto::generate_random_sign_keys,
        crate::crypto::keys::generate_random_sign_keys_api,
    );
    module.register_sync_fn(crate::crypto::sign, crate::crypto::keys::sign_api);
    module.register_sync_fn(
        crate::crypto::verify_signature,
        crate::crypto::keys::verify_signature_api,
    );

    // Sha

    module.register_sync_fn(crate::crypto::sha256, crate::crypto::hash::sha256_api);
    module.register_sync_fn(crate::crypto::sha512, crate::crypto::hash::sha512_api);

    // Scrypt

    module.register_sync_fn(
        crate::crypto::encscrypt::scrypt,
        crate::crypto::encscrypt::scrypt_api,
    );

    // NaCl

    module.register_sync_fn(
        crate::crypto::nacl_sign_keypair_from_secret_key,
        crate::crypto::nacl::nacl_sign_keypair_from_secret_key_api,
    );
    module.register_sync_fn(crate::crypto::nacl_sign, crate::crypto::nacl::nacl_sign_api);
    module.register_sync_fn(
        crate::crypto::nacl_sign_open,
        crate::crypto::nacl::nacl_sign_open_api,
    );
    module.register_sync_fn(
        crate::crypto::nacl_sign_detached,
        crate::crypto::nacl::nacl_sign_detached_api,
    );

    module.register_sync_fn_without_args(
        crate::crypto::nacl_box_keypair,
        crate::crypto::nacl::nacl_box_keypair_api,
    );
    module.register_sync_fn(
        crate::crypto::nacl_box_keypair_from_secret_key,
        crate::crypto::nacl::nacl_box_keypair_from_secret_key_api,
    );
    module.register_sync_fn(crate::crypto::nacl_box, crate::crypto::nacl::nacl_box_api);
    module.register_sync_fn(
        crate::crypto::nacl_box_open,
        crate::crypto::nacl::nacl_box_open_api,
    );
    module.register_sync_fn(
        crate::crypto::nacl_secret_box,
        crate::crypto::nacl::nacl_secret_box_api,
    );
    module.register_sync_fn(
        crate::crypto::nacl_secret_box_open,
        crate::crypto::nacl::nacl_secret_box_open_api,
    );

    // Mnemonic

    module.register_sync_fn(
        crate::crypto::mnemonic_words,
        crate::crypto::mnemonic::mnemonic_words_api,
    );
    module.register_sync_fn(
        crate::crypto::mnemonic_from_random,
        crate::crypto::mnemonic::mnemonic_from_random_api,
    );
    module.register_sync_fn(
        crate::crypto::mnemonic_from_entropy,
        crate::crypto::mnemonic::mnemonic_from_entropy_api,
    );
    module.register_sync_fn(
        crate::crypto::mnemonic_verify,
        crate::crypto::mnemonic::mnemonic_verify_api,
    );
    module.register_sync_fn(
        crate::crypto::mnemonic_derive_sign_keys,
        crate::crypto::mnemonic::mnemonic_derive_sign_keys_api,
    );

    // HDKey

    module.register_sync_fn(
        crate::crypto::hdkey_xprv_from_mnemonic,
        crate::crypto::hdkey::hdkey_xprv_from_mnemonic_api,
    );
    module.register_sync_fn(
        crate::crypto::hdkey_derive_from_xprv,
        crate::crypto::hdkey::hdkey_derive_from_xprv_api,
    );
    module.register_sync_fn(
        crate::crypto::hdkey_derive_from_xprv_path,
        crate::crypto::hdkey::hdkey_derive_from_xprv_path_api,
    );
    module.register_sync_fn(
        crate::crypto::hdkey_secret_from_xprv,
        crate::crypto::hdkey::hdkey_secret_from_xprv_api,
    );
    module.register_sync_fn(
        crate::crypto::hdkey_public_from_xprv,
        crate::crypto::hdkey::hdkey_public_from_xprv_api,
    );
    module.register();
}

/// Provides message encoding and decoding according to the ABI
/// specification.
#[derive(ApiModule)]
#[api_module(name = "abi")]
pub(crate) struct AbiModule;

fn register_abi(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<AbiModule>(handlers);
    module.register_type::<crate::abi::Abi>();
    module.register_type::<crate::abi::AbiHandle>();
    module.register_type::<crate::abi::FunctionHeader>();
    module.register_type::<crate::abi::CallSet>();
    module.register_type::<crate::abi::DeploySet>();
    module.register_type::<crate::abi::Signer>();
    module.register_type::<crate::abi::MessageBodyType>();
    module.register_type::<crate::abi::StateInitSource>();
    module.register_type::<crate::abi::StateInitParams>();
    module.register_type::<crate::abi::MessageSource>();

    module.register_async_fn(
        crate::abi::encode_message_body,
        crate::abi::encode_message::encode_message_body_api,
    );
    module.register_sync_fn(
        crate::abi::attach_signature_to_message_body,
        crate::abi::encode_message::attach_signature_to_message_body_api,
    );
    module.register_async_fn(
        crate::abi::encode_message,
        crate::abi::encode_message::encode_message_api,
    );
    module.register_sync_fn(
        crate::abi::attach_signature,
        crate::abi::encode_message::attach_signature_api,
    );
    module.register_sync_fn(
        crate::abi::decode_message,
        crate::abi::decode_message::decode_message_api,
    );
    module.register_sync_fn(
        crate::abi::decode_message_body,
        crate::abi::decode_message::decode_message_body_api,
    );
    module.register_async_fn(
        crate::abi::encode_account,
        crate::abi::encode_account::encode_account_api,
    );
    module.register();
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "boc")]
pub(crate) struct BocModule;

fn register_boc(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<BocModule>(handlers);
    module.register_sync_fn(
        crate::boc::parse_message,
        crate::boc::parse::parse_message_api,
    );
    module.register_sync_fn(
        crate::boc::parse_transaction,
        crate::boc::parse::parse_transaction_api,
    );
    module.register_sync_fn(
        crate::boc::parse_account,
        crate::boc::parse::parse_account_api,
    );
    module.register_sync_fn(crate::boc::parse_block, crate::boc::parse::parse_block_api);
    module.register_sync_fn(
        crate::boc::get_blockchain_config,
        crate::boc::blockchain_config::get_blockchain_config_api,
    );
    module.register();
}

/// Network access.
#[derive(ApiModule)]
#[api_module(name = "net")]
pub(crate) struct NetModule;

fn register_net(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<NetModule>(handlers);
    module.register_type::<crate::net::OrderBy>();
    module.register_type::<crate::net::SortDirection>();

    module.register_async_fn(
        crate::net::query_collection,
        crate::net::query_collection_api,
    );
    module.register_async_fn(
        crate::net::wait_for_collection,
        crate::net::wait_for_collection_api,
    );
    module.register_async_fn(crate::net::unsubscribe, crate::net::unsubscribe_api);
    module.register_async_fn_with_callback(
        super::net::subscribe_collection,
        super::net::subscribe_collection_api,
    );
    module.register();
}

/// Message processing module.
///
/// This module incorporates functions related to complex message
/// processing scenarios.
#[derive(ApiModule)]
#[api_module(name = "processing")]
pub struct ProcessingModule;

fn register_processing(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<ProcessingModule>(handlers);
    module.register_type::<crate::processing::ProcessingEvent>();
    module.register_type::<crate::processing::ResultOfProcessMessage>();
    module.register_type::<crate::processing::DecodedOutput>();

    module.register_async_fn_with_callback(
        super::processing::send_message,
        super::processing::send_message_api,
    );
    module.register_async_fn_with_callback(
        super::processing::wait_for_transaction,
        super::processing::wait_for_transaction_api,
    );
    module.register_async_fn_with_callback(
        super::processing::process_message,
        super::processing::process_message_api,
    );
    module.register();
}

#[derive(ApiModule)]
#[api_module(name = "tvm")]
pub struct TvmModule;

fn register_tvm(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<TvmModule>(handlers);
    module.register_type::<crate::tvm::types::ExecutionOptions>();
    module.register_type::<crate::tvm::AccountForExecutor>();
    module.register_async_fn(
        crate::tvm::run_executor,
        crate::tvm::run_message::run_executor_api,
    );
    module.register_async_fn(
        crate::tvm::run_tvm,
        crate::tvm::run_message::run_tvm_api,
    );
    module.register_async_fn(
        crate::tvm::run_get,
        crate::tvm::run_get::run_get_api,
    );
    module.register();
}

/// Misc utility Functions.
#[derive(ApiModule)]
#[api_module(name = "utils")]
pub struct UtilsModule;

fn register_utils(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<UtilsModule>(handlers);
    module.register_type::<crate::utils::AddressStringFormat>();
    module.register_sync_fn(
        crate::utils::convert_address,
        crate::utils::conversion::convert_address_api,
    );
    module.register();
}

pub(crate) fn register_modules(handlers: &mut RuntimeHandlers) {
    register_client(handlers);
    register_crypto(handlers);
    register_abi(handlers);
    register_boc(handlers);
    register_processing(handlers);
    register_utils(handlers);
    register_tvm(handlers);

        register_net(handlers);
}
