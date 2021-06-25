# Common Types
## ResponseHandler
```ts
type ResponseHandler = (params: any, responseType: number) => void;
```

Handles additional function responses.

Where:
- `params`: _any_ – Response parameters. Actual type depends on API function. 
- `responseType`: _number_ – Function specific response type.

# Modules
## [client](mod_client.md) – Provides information about library.

[get_api_reference](mod_client.md#get_api_reference) – Returns Core Library API reference

[version](mod_client.md#version) – Returns Core Library version

[build_info](mod_client.md#build_info) – Returns detailed information about this build.

[resolve_app_request](mod_client.md#resolve_app_request) – Resolves application request processing result

## [crypto](mod_crypto.md) – Crypto functions.

[factorize](mod_crypto.md#factorize) – Integer factorization

[modular_power](mod_crypto.md#modular_power) – Modular exponentiation

[ton_crc16](mod_crypto.md#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate_random_bytes](mod_crypto.md#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert_public_key_to_ton_safe_format](mod_crypto.md#convert_public_key_to_ton_safe_format) – Converts public key to ton safe_format

[generate_random_sign_keys](mod_crypto.md#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](mod_crypto.md#sign) – Signs a data using the provided keys.

[verify_signature](mod_crypto.md#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](mod_crypto.md#sha256) – Calculates SHA256 hash of the specified data.

[sha512](mod_crypto.md#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](mod_crypto.md#scrypt) – Perform `scrypt` encryption

[nacl_sign_keypair_from_secret_key](mod_crypto.md#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl_sign](mod_crypto.md#nacl_sign) – Signs data using the signer's secret key.

[nacl_sign_open](mod_crypto.md#nacl_sign_open) – Verifies the signature and returns the unsigned message

[nacl_sign_detached](mod_crypto.md#nacl_sign_detached) – Signs the message using the secret key and returns a signature.

[nacl_sign_detached_verify](mod_crypto.md#nacl_sign_detached_verify) – Verifies the signature with public key and `unsigned` data.

[nacl_box_keypair](mod_crypto.md#nacl_box_keypair) – Generates a random NaCl key pair

[nacl_box_keypair_from_secret_key](mod_crypto.md#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl_box](mod_crypto.md#nacl_box) – Public key authenticated encryption

[nacl_box_open](mod_crypto.md#nacl_box_open) – Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

[nacl_secret_box](mod_crypto.md#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl_secret_box_open](mod_crypto.md#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic_words](mod_crypto.md#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic_from_random](mod_crypto.md#mnemonic_from_random) – Generates a random mnemonic

[mnemonic_from_entropy](mod_crypto.md#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic_verify](mod_crypto.md#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic_derive_sign_keys](mod_crypto.md#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey_xprv_from_mnemonic](mod_crypto.md#hdkey_xprv_from_mnemonic) – Generates an extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](mod_crypto.md#hdkey_derive_from_xprv) – Returns extended private key derived from the specified extended private key and child index

[hdkey_derive_from_xprv_path](mod_crypto.md#hdkey_derive_from_xprv_path) – Derives the extended private key from the specified key and path

[hdkey_secret_from_xprv](mod_crypto.md#hdkey_secret_from_xprv) – Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](mod_crypto.md#hdkey_public_from_xprv) – Extracts the public key from the serialized extended private key

[chacha20](mod_crypto.md#chacha20) – Performs symmetric `chacha20` encryption.

[register_signing_box](mod_crypto.md#register_signing_box) – Register an application implemented signing box.

[get_signing_box](mod_crypto.md#get_signing_box) – Creates a default signing box implementation.

[signing_box_get_public_key](mod_crypto.md#signing_box_get_public_key) – Returns public key of signing key pair.

[signing_box_sign](mod_crypto.md#signing_box_sign) – Returns signed user data.

[remove_signing_box](mod_crypto.md#remove_signing_box) – Removes signing box from SDK.

[register_encryption_box](mod_crypto.md#register_encryption_box) – Register an application implemented encryption box.

[remove_encryption_box](mod_crypto.md#remove_encryption_box) – Removes encryption box from SDK

[encryption_box_get_info](mod_crypto.md#encryption_box_get_info) – Queries info from the given encryption box

[encryption_box_encrypt](mod_crypto.md#encryption_box_encrypt) – Encrypts data using given encryption box

[encryption_box_decrypt](mod_crypto.md#encryption_box_decrypt) – Decrypts data using given encryption box

## [abi](mod_abi.md) – Provides message encoding and decoding according to the ABI specification.

[encode_message_body](mod_abi.md#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](mod_abi.md#attach_signature_to_message_body)

[encode_message](mod_abi.md#encode_message) – Encodes an ABI-compatible message

[encode_internal_message](mod_abi.md#encode_internal_message) – Encodes an internal ABI-compatible message

[attach_signature](mod_abi.md#attach_signature) – Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

[decode_message](mod_abi.md#decode_message) – Decodes message body using provided message BOC and ABI.

[decode_message_body](mod_abi.md#decode_message_body) – Decodes message body using provided body BOC and ABI.

[encode_account](mod_abi.md#encode_account) – Creates account state BOC

## [boc](mod_boc.md) – BOC manipulation module.

[parse_message](mod_boc.md#parse_message) – Parses message boc into a JSON

[parse_transaction](mod_boc.md#parse_transaction) – Parses transaction boc into a JSON

[parse_account](mod_boc.md#parse_account) – Parses account boc into a JSON

[parse_block](mod_boc.md#parse_block) – Parses block boc into a JSON

[parse_shardstate](mod_boc.md#parse_shardstate) – Parses shardstate boc into a JSON

[get_blockchain_config](mod_boc.md#get_blockchain_config) – Extract blockchain configuration from key block and also from zerostate.

[get_boc_hash](mod_boc.md#get_boc_hash) – Calculates BOC root hash

[get_code_from_tvc](mod_boc.md#get_code_from_tvc) – Extracts code from TVC contract image

[cache_get](mod_boc.md#cache_get) – Get BOC from cache

[cache_set](mod_boc.md#cache_set) – Save BOC into cache

[cache_unpin](mod_boc.md#cache_unpin) – Unpin BOCs with specified pin.

[encode_boc](mod_boc.md#encode_boc) – Encodes BOC from builder operations.

## [processing](mod_processing.md) – Message processing module.

[send_message](mod_processing.md#send_message) – Sends message to the network

[wait_for_transaction](mod_processing.md#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](mod_processing.md#process_message) – Creates message, sends it to the network and monitors its processing.

## [utils](mod_utils.md) – Misc utility Functions.

[convert_address](mod_utils.md#convert_address) – Converts address from any TON format to any TON format

[calc_storage_fee](mod_utils.md#calc_storage_fee) – Calculates storage fee for an account over a specified time period

[compress_zstd](mod_utils.md#compress_zstd) – Compresses data using Zstandard algorithm

[decompress_zstd](mod_utils.md#decompress_zstd) – Decompresses data using Zstandard algorithm

## [tvm](mod_tvm.md)

[run_executor](mod_tvm.md#run_executor) – Emulates all the phases of contract execution locally

[run_tvm](mod_tvm.md#run_tvm) – Executes get-methods of ABI-compatible contracts

[run_get](mod_tvm.md#run_get) – Executes a get-method of FIFT contract

## [net](mod_net.md) – Network access.

[query](mod_net.md#query) – Performs DAppServer GraphQL query.

[batch_query](mod_net.md#batch_query) – Performs multiple queries per single fetch.

[query_collection](mod_net.md#query_collection) – Queries collection data

[aggregate_collection](mod_net.md#aggregate_collection) – Aggregates collection data.

[wait_for_collection](mod_net.md#wait_for_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](mod_net.md#unsubscribe) – Cancels a subscription

[subscribe_collection](mod_net.md#subscribe_collection) – Creates a subscription

[suspend](mod_net.md#suspend) – Suspends network module to stop any network activity

[resume](mod_net.md#resume) – Resumes network module to enable network activity

[find_last_shard_block](mod_net.md#find_last_shard_block) – Returns ID of the last block in a specified account shard

[fetch_endpoints](mod_net.md#fetch_endpoints) – Requests the list of alternative endpoints from server

[set_endpoints](mod_net.md#set_endpoints) – Sets the list of endpoints to use on reinit

[get_endpoints](mod_net.md#get_endpoints) – Requests the list of alternative endpoints from server

[query_counterparties](mod_net.md#query_counterparties) – Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

[query_transaction_tree](mod_net.md#query_transaction_tree) – Returns transactions tree for specific message.

[create_block_iterator](mod_net.md#create_block_iterator) – Creates block iterator.

[resume_block_iterator](mod_net.md#resume_block_iterator) – Resumes block iterator.

[create_transaction_iterator](mod_net.md#create_transaction_iterator) – Creates transaction iterator.

[resume_transaction_iterator](mod_net.md#resume_transaction_iterator) – Resumes transaction iterator.

[iterator_next](mod_net.md#iterator_next) – Returns next available items.

[remove_iterator](mod_net.md#remove_iterator) – Removes an iterator

## [debot](mod_debot.md) – [UNSTABLE](UNSTABLE.md) Module for working with debot.

[init](mod_debot.md#init) – [UNSTABLE](UNSTABLE.md) Creates and instance of DeBot.

[start](mod_debot.md#start) – [UNSTABLE](UNSTABLE.md) Starts the DeBot.

[fetch](mod_debot.md#fetch) – [UNSTABLE](UNSTABLE.md) Fetches DeBot metadata from blockchain.

[execute](mod_debot.md#execute) – [UNSTABLE](UNSTABLE.md) Executes debot action.

[send](mod_debot.md#send) – [UNSTABLE](UNSTABLE.md) Sends message to Debot.

[remove](mod_debot.md#remove) – [UNSTABLE](UNSTABLE.md) Destroys debot handle.

