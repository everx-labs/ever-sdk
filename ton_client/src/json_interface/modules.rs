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
 *
 */

use super::registrar::ModuleReg;
use super::runtime::RuntimeHandlers;
use crate::boc::BuilderOp;

/// Provides information about library.
#[derive(ApiModule)]
#[api_module(name = "client")]
pub(crate) struct ClientModule;

fn register_client(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<ClientModule>(handlers);
    module.register_error_code::<crate::client::errors::ErrorCode>();
    module.register_type::<crate::error::ClientError>();
    module.register_type::<crate::client::ClientConfig>();
    module.register_type::<crate::net::NetworkConfig>();
    module.register_type::<crate::net::NetworkQueriesProtocol>();
    module.register_type::<crate::crypto::CryptoConfig>();
    module.register_type::<crate::abi::AbiConfig>();
    module.register_type::<crate::boc::BocConfig>();
    module.register_type::<crate::proofs::ProofsConfig>();
    module.register_type::<crate::client::BuildInfoDependency>();
    module.register_type::<crate::client::ParamsOfAppRequest>();
    module.register_type::<crate::client::AppRequestResult>();

    module.register_sync_fn_without_args(
        crate::client::get_api_reference,
        crate::client::get_api_reference_api,
    );
    module.register_sync_fn_without_args(crate::client::version, crate::client::version_api);
    module.register_sync_fn_without_args(crate::client::config, crate::client::config_api);
    module.register_sync_fn_without_args(crate::client::build_info, crate::client::build_info_api);
    module.register_async_fn(
        crate::client::resolve_app_request,
        crate::client::resolve_app_request_api,
    );
    module.register();
}

/// Crypto functions.
#[derive(ApiModule)]
#[api_module(name = "crypto")]
pub(crate) struct CryptoModule;

fn register_crypto(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<CryptoModule>(handlers);

    module.register_error_code::<crate::crypto::ErrorCode>();
    module.register_type::<crate::crypto::SigningBoxHandle>();
    module.register_type::<crate::crypto::EncryptionBoxHandle>();
    module.register_type::<crate::crypto::EncryptionBoxInfo>();
    module.register_type::<crate::crypto::EncryptionAlgorithm>();
    module.register_type::<crate::crypto::CipherMode>();
    module.register_type::<crate::crypto::AesParamsEB>();
    module.register_type::<crate::crypto::AesInfo>();
    module.register_type::<crate::crypto::ChaCha20ParamsEB>();
    module.register_type::<crate::crypto::NaclBoxParamsEB>();
    module.register_type::<crate::crypto::NaclSecretBoxParamsEB>();
    module.register_type::<crate::crypto::CryptoBoxSecret>();
    module.register_type::<crate::crypto::CryptoBoxHandle>();
    module.register_type::<crate::crypto::BoxEncryptionAlgorithm>();
    module.register_type::<crate::crypto::ChaCha20ParamsCB>();
    module.register_type::<crate::crypto::NaclBoxParamsCB>();
    module.register_type::<crate::crypto::NaclSecretBoxParamsCB>();

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
    module.register_sync_fn(
        crate::crypto::nacl_sign_detached_verify,
        crate::crypto::nacl::nacl_sign_detached_verify_api,
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

    // Encryption

    module.register_sync_fn(
        crate::crypto::chacha20,
        crate::crypto::encryption::chacha20_api,
    );

    // Boxes

    // Crypto box
    module.register_async_fn_with_app_object(
        super::crypto::create_crypto_box,
        super::crypto::create_crypto_box_api,
    );
    module.register_async_fn(
        crate::crypto::remove_crypto_box,
        crate::crypto::boxes::crypto_box::remove_crypto_box_api,
    );
    module.register_async_fn(
        crate::crypto::get_crypto_box_info,
        crate::crypto::boxes::crypto_box::get_crypto_box_info_api,
    );
    module.register_async_fn(
        crate::crypto::get_crypto_box_seed_phrase,
        crate::crypto::boxes::crypto_box::get_crypto_box_seed_phrase_api,
    );
    module.register_async_fn(
        crate::crypto::get_signing_box_from_crypto_box,
        crate::crypto::boxes::crypto_box::get_signing_box_from_crypto_box_api,
    );
    module.register_async_fn(
        crate::crypto::get_encryption_box_from_crypto_box,
        crate::crypto::boxes::crypto_box::get_encryption_box_from_crypto_box_api,
    );
    module.register_async_fn(
        crate::crypto::clear_crypto_box_secret_cache,
        crate::crypto::boxes::crypto_box::clear_crypto_box_secret_cache_api,
    );

    // Signing box
    module.register_async_fn_with_app_object_no_args(
        super::crypto::register_signing_box,
        super::crypto::register_signing_box_api,
    );
    module.register_async_fn(
        crate::crypto::get_signing_box,
        crate::crypto::boxes::signing_box::get_signing_box_api,
    );
    module.register_async_fn(
        crate::crypto::signing_box_get_public_key,
        crate::crypto::boxes::signing_box::signing_box_get_public_key_api,
    );
    module.register_async_fn(
        crate::crypto::signing_box_sign,
        crate::crypto::boxes::signing_box::signing_box_sign_api,
    );
    module.register_sync_fn(
        crate::crypto::remove_signing_box,
        crate::crypto::boxes::signing_box::remove_signing_box_api,
    );

    // Encryption box
    module.register_async_fn_with_app_object_no_args(
        super::crypto::register_encryption_box,
        super::crypto::register_encryption_box_api,
    );
    module.register_sync_fn(
        crate::crypto::remove_encryption_box,
        crate::crypto::boxes::encryption_box::remove_encryption_box_api,
    );
    module.register_async_fn(
        crate::crypto::encryption_box_get_info,
        crate::crypto::boxes::encryption_box::encryption_box_get_info_api,
    );
    module.register_async_fn(
        crate::crypto::encryption_box_encrypt,
        crate::crypto::boxes::encryption_box::encryption_box_encrypt_api,
    );
    module.register_async_fn(
        crate::crypto::encryption_box_decrypt,
        crate::crypto::boxes::encryption_box::encryption_box_decrypt_api,
    );
    module.register_async_fn(
        crate::crypto::create_encryption_box,
        crate::crypto::boxes::encryption_box::create_encryption_box_api,
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
    module.register_error_code::<crate::abi::ErrorCode>();
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
    module.register_type::<crate::abi::AbiParam>();
    module.register_type::<crate::abi::AbiEvent>();
    module.register_type::<crate::abi::AbiData>();
    module.register_type::<crate::abi::AbiFunction>();
    module.register_type::<crate::abi::AbiContract>();

    module.register_async_fn(
        crate::abi::encode_message_body,
        crate::abi::encode_message::encode_message_body_api,
    );
    module.register_async_fn(
        crate::abi::attach_signature_to_message_body,
        crate::abi::encode_message::attach_signature_to_message_body_api,
    );
    module.register_async_fn(
        crate::abi::encode_message,
        crate::abi::encode_message::encode_message_api,
    );
    module.register_async_fn(
        crate::abi::encode_internal_message,
        crate::abi::encode_message::encode_internal_message_api,
    );
    module.register_async_fn(
        crate::abi::attach_signature,
        crate::abi::encode_message::attach_signature_api,
    );
    module.register_async_fn(
        crate::abi::decode_message,
        crate::abi::decode_message::decode_message_api,
    );
    module.register_async_fn(
        crate::abi::decode_message_body,
        crate::abi::decode_message::decode_message_body_api,
    );
    module.register_async_fn(
        crate::abi::encode_account,
        crate::abi::encode_account::encode_account_api,
    );
    module.register_async_fn(
        crate::abi::decode_account_data,
        crate::abi::decode_data::decode_account_data_api,
    );
    module.register_async_fn(
        crate::abi::update_initial_data,
        crate::abi::init_data::update_initial_data_api,
    );
    module.register_async_fn(
        crate::abi::encode_initial_data,
        crate::abi::init_data::encode_initial_data_api,
    );
    module.register_async_fn(
        crate::abi::decode_initial_data,
        crate::abi::init_data::decode_initial_data_api,
    );
    module.register_async_fn(
        crate::abi::decode_boc,
        crate::abi::decode_boc::decode_boc_api,
    );
    module.register_async_fn(
        crate::abi::encode_boc,
        crate::abi::encode_boc::encode_boc_api,
    );
    module.register_sync_fn(
        crate::abi::calc_function_id,
        crate::abi::function_id::calc_function_id_api,
    );
    module.register();
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "boc")]
pub(crate) struct BocModule;

fn register_boc(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<BocModule>(handlers);
    module.register_type::<crate::boc::BocCacheType>();
    module.register_error_code::<crate::boc::ErrorCode>();
    module.register_async_fn(
        crate::boc::parse_message,
        crate::boc::parse::parse_message_api,
    );
    module.register_async_fn(
        crate::boc::parse_transaction,
        crate::boc::parse::parse_transaction_api,
    );
    module.register_async_fn(
        crate::boc::parse_account,
        crate::boc::parse::parse_account_api,
    );
    module.register_async_fn(crate::boc::parse_block, crate::boc::parse::parse_block_api);
    module.register_async_fn(
        crate::boc::parse_shardstate,
        crate::boc::parse::parse_shardstate_api,
    );
    module.register_async_fn(
        crate::boc::get_blockchain_config,
        crate::boc::blockchain_config::get_blockchain_config_api,
    );
    module.register_async_fn(
        crate::boc::get_boc_hash,
        crate::boc::common::get_boc_hash_api,
    );
    module.register_async_fn(
        crate::boc::get_boc_depth,
        crate::boc::common::get_boc_depth_api,
    );
    module.register_async_fn(
        crate::boc::get_code_from_tvc,
        crate::boc::tvc::get_code_from_tvc_api,
    );
    module.register_async_fn(crate::boc::cache_get, crate::boc::cache::cache_get_api);
    module.register_async_fn(crate::boc::cache_set, crate::boc::cache::cache_set_api);
    module.register_async_fn(crate::boc::cache_unpin, crate::boc::cache::cache_unpin_api);
    module.register_type::<BuilderOp>();
    module.register_async_fn(crate::boc::encode_boc, crate::boc::encode::encode_boc_api);
    module.register_async_fn(
        crate::boc::get_code_salt,
        crate::boc::tvc::get_code_salt_api,
    );
    module.register_async_fn(
        crate::boc::set_code_salt,
        crate::boc::tvc::set_code_salt_api,
    );
    module.register_async_fn(crate::boc::decode_tvc, crate::boc::tvc::decode_tvc_api);
    module.register_async_fn(crate::boc::encode_tvc, crate::boc::tvc::encode_tvc_api);
    module.register_async_fn(
        crate::boc::encode_external_in_message,
        crate::boc::encode_external_in_message::encode_external_in_message_api,
    );
    module.register_async_fn(
        crate::boc::get_compiler_version,
        crate::boc::tvc::get_compiler_version_api,
    );
    module.register();
}

/// Network access.
#[derive(ApiModule)]
#[api_module(name = "net")]
pub(crate) struct NetModule;

fn register_net(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<NetModule>(handlers);
    module.register_error_code::<crate::net::ErrorCode>();

    module.register_type::<crate::net::OrderBy>();
    module.register_type::<crate::net::SortDirection>();
    module.register_type::<crate::net::ParamsOfQueryOperation>();
    module.register_type::<crate::net::FieldAggregation>();
    module.register_type::<crate::net::AggregationFn>();
    module.register_type::<crate::net::TransactionNode>();
    module.register_type::<crate::net::MessageNode>();

    module.register_async_fn(crate::net::query, crate::net::queries::query_api);
    module.register_async_fn(crate::net::batch_query, crate::net::batch::batch_query_api);
    module.register_async_fn(
        crate::net::query_collection,
        crate::net::queries::query_collection_api,
    );
    module.register_async_fn(
        crate::net::aggregate_collection,
        crate::net::queries::aggregate_collection_api,
    );
    module.register_async_fn(
        crate::net::wait_for_collection,
        crate::net::queries::wait_for_collection_api,
    );
    module.register_async_fn(
        crate::net::unsubscribe,
        crate::net::subscriptions::unsubscribe_api,
    );
    module.register_async_fn_with_callback(
        super::net::subscribe_collection,
        super::net::subscribe_collection_api,
    );
    module.register_async_fn_with_callback(super::net::subscribe, super::net::subscribe_api);
    module.register_async_fn_no_args(crate::net::suspend, crate::net::suspend_api);
    module.register_async_fn_no_args(crate::net::resume, crate::net::resume_api);
    module.register_async_fn(
        crate::net::find_last_shard_block,
        crate::net::find_last_shard_block_api,
    );
    module.register_async_fn_no_args(crate::net::fetch_endpoints, crate::net::fetch_endpoints_api);
    module.register_async_fn(crate::net::set_endpoints, crate::net::set_endpoints_api);
    module.register_async_fn_no_args(crate::net::get_endpoints, crate::net::get_endpoints_api);
    module.register_async_fn(
        crate::net::query_counterparties,
        crate::net::queries::query_counterparties_api,
    );
    module.register_async_fn(
        crate::net::transaction_tree::query_transaction_tree,
        crate::net::transaction_tree::query_transaction_tree_api,
    );

    module.register_async_fn(
        crate::net::iterators::block_iterator::create_block_iterator,
        crate::net::iterators::block_iterator::create_block_iterator_api,
    );
    module.register_async_fn(
        crate::net::iterators::block_iterator::resume_block_iterator,
        crate::net::iterators::block_iterator::resume_block_iterator_api,
    );
    module.register_async_fn(
        crate::net::iterators::transaction_iterator::create_transaction_iterator,
        crate::net::iterators::transaction_iterator::create_transaction_iterator_api,
    );
    module.register_async_fn(
        crate::net::iterators::transaction_iterator::resume_transaction_iterator,
        crate::net::iterators::transaction_iterator::resume_transaction_iterator_api,
    );
    module.register_async_fn(
        crate::net::iterators::iterator_next,
        crate::net::iterators::iterator_next_api,
    );
    module.register_async_fn(
        crate::net::iterators::remove_iterator,
        crate::net::iterators::remove_iterator_api,
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
    module.register_error_code::<crate::processing::ErrorCode>();

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
    module.register_error_code::<crate::tvm::ErrorCode>();

    module.register_type::<crate::tvm::types::ExecutionOptions>();
    module.register_type::<crate::tvm::AccountForExecutor>();
    module.register_type::<crate::tvm::TransactionFees>();
    module.register_async_fn(
        crate::tvm::run_executor,
        crate::tvm::run_message::run_executor_api,
    );
    module.register_async_fn(crate::tvm::run_tvm, crate::tvm::run_message::run_tvm_api);
    module.register_async_fn(crate::tvm::run_get, crate::tvm::run_get::run_get_api);
    module.register();
}

/// Misc utility Functions.
#[derive(ApiModule)]
#[api_module(name = "utils")]
pub struct UtilsModule;

fn register_utils(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<UtilsModule>(handlers);
    module.register_type::<crate::utils::AddressStringFormat>();
    module.register_type::<crate::utils::AccountAddressType>();
    module.register_sync_fn(
        crate::utils::convert_address,
        crate::utils::conversion::convert_address_api,
    );
    module.register_sync_fn(
        crate::utils::get_address_type,
        crate::utils::conversion::get_address_type_api,
    );
    module.register_async_fn(
        crate::utils::calc_storage_fee,
        crate::utils::calc_storage_fee::calc_storage_fee_api,
    );
    #[cfg(feature = "include-zstd")]
    module.register_sync_fn(super::utils::compress_zstd, super::utils::compress_zstd_api);
    #[cfg(feature = "include-zstd")]
    module.register_sync_fn(
        super::utils::decompress_zstd,
        super::utils::decompress_zstd_api,
    );
    module.register();
}

/// [UNSTABLE](UNSTABLE.md) Module for working with debot.
#[derive(ApiModule)]
#[api_module(name = "debot")]
pub struct DebotModule;

fn register_debot(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<DebotModule>(handlers);
    module.register_error_code::<crate::debot::ErrorCode>();

    module.register_type::<crate::debot::DebotHandle>();
    module.register_type::<crate::debot::DebotAction>();
    module.register_type::<crate::debot::DebotHandle>();
    module.register_type::<crate::debot::DebotInfo>();
    module.register_type::<crate::debot::DebotActivity>();
    module.register_type::<crate::debot::Spending>();
    module.register_async_fn_with_app_object(
        crate::json_interface::debot::init,
        crate::json_interface::debot::init_api,
    );
    module.register_async_fn(crate::debot::start, crate::debot::start_api);
    module.register_async_fn(crate::debot::fetch, crate::debot::fetch_api);
    module.register_async_fn(crate::debot::execute, crate::debot::execute_api);
    module.register_async_fn(crate::debot::send, crate::debot::send_api);
    module.register_sync_fn(crate::debot::remove, crate::debot::remove_api);
    module.register();
}

/// [UNSTABLE](UNSTABLE.md) Module for proving data, retrieved from TONOS API.
#[derive(ApiModule)]
#[api_module(name = "proofs")]
pub struct ProofsModule;

fn register_proofs(handlers: &mut RuntimeHandlers) {
    let mut module = ModuleReg::new::<ProofsModule>(handlers);
    module.register_error_code::<crate::proofs::ErrorCode>();

    module.register_type::<crate::proofs::ParamsOfProofBlockData>();
    module.register_type::<crate::proofs::ParamsOfProofTransactionData>();
    module.register_type::<crate::proofs::ParamsOfProofMessageData>();

    module.register_async_fn(
        crate::proofs::proof_block_data,
        crate::proofs::proof_block_data_api,
    );
    module.register_async_fn(
        crate::proofs::proof_transaction_data,
        crate::proofs::proof_transaction_data_api,
    );
    module.register_async_fn(
        crate::proofs::proof_message_data,
        crate::proofs::proof_message_data_api,
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
    register_debot(handlers);
    register_proofs(handlers);
}
