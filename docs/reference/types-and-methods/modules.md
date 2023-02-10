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
## [client](mod\_client.md) – Provides information about library.

[get_api_reference](mod\_client.md#get_api_reference) – Returns Core Library API reference

[version](mod\_client.md#version) – Returns Core Library version

[config](mod\_client.md#config) – Returns Core Library API reference

[build_info](mod\_client.md#build_info) – Returns detailed information about this build.

[resolve_app_request](mod\_client.md#resolve_app_request) – Resolves application request processing result

## [crypto](mod\_crypto.md) – Crypto functions.

[factorize](mod\_crypto.md#factorize) – Integer factorization

[modular_power](mod\_crypto.md#modular_power) – Modular exponentiation

[ton_crc16](mod\_crypto.md#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate_random_bytes](mod\_crypto.md#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert_public_key_to_ton_safe_format](mod\_crypto.md#convert_public_key_to_ton_safe_format) – Converts public key to ton safe_format

[generate_random_sign_keys](mod\_crypto.md#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](mod\_crypto.md#sign) – Signs a data using the provided keys.

[verify_signature](mod\_crypto.md#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](mod\_crypto.md#sha256) – Calculates SHA256 hash of the specified data.

[sha512](mod\_crypto.md#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](mod\_crypto.md#scrypt) – Perform `scrypt` encryption

[nacl_sign_keypair_from_secret_key](mod\_crypto.md#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl_sign](mod\_crypto.md#nacl_sign) – Signs data using the signer's secret key.

[nacl_sign_open](mod\_crypto.md#nacl_sign_open) – Verifies the signature and returns the unsigned message

[nacl_sign_detached](mod\_crypto.md#nacl_sign_detached) – Signs the message using the secret key and returns a signature.

[nacl_sign_detached_verify](mod\_crypto.md#nacl_sign_detached_verify) – Verifies the signature with public key and `unsigned` data.

[nacl_box_keypair](mod\_crypto.md#nacl_box_keypair) – Generates a random NaCl key pair

[nacl_box_keypair_from_secret_key](mod\_crypto.md#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl_box](mod\_crypto.md#nacl_box) – Public key authenticated encryption

[nacl_box_open](mod\_crypto.md#nacl_box_open) – Decrypt and verify the cipher text using the receivers secret key, the senders public key, and the nonce.

[nacl_secret_box](mod\_crypto.md#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl_secret_box_open](mod\_crypto.md#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic_words](mod\_crypto.md#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic_from_random](mod\_crypto.md#mnemonic_from_random) – Generates a random mnemonic

[mnemonic_from_entropy](mod\_crypto.md#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic_verify](mod\_crypto.md#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic_derive_sign_keys](mod\_crypto.md#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey_xprv_from_mnemonic](mod\_crypto.md#hdkey_xprv_from_mnemonic) – Generates an extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](mod\_crypto.md#hdkey_derive_from_xprv) – Returns extended private key derived from the specified extended private key and child index

[hdkey_derive_from_xprv_path](mod\_crypto.md#hdkey_derive_from_xprv_path) – Derives the extended private key from the specified key and path

[hdkey_secret_from_xprv](mod\_crypto.md#hdkey_secret_from_xprv) – Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](mod\_crypto.md#hdkey_public_from_xprv) – Extracts the public key from the serialized extended private key

[chacha20](mod\_crypto.md#chacha20) – Performs symmetric `chacha20` encryption.

[create_crypto_box](mod\_crypto.md#create_crypto_box) – Creates a Crypto Box instance.

[remove_crypto_box](mod\_crypto.md#remove_crypto_box) – Removes Crypto Box. Clears all secret data.

[get_crypto_box_info](mod\_crypto.md#get_crypto_box_info) – Get Crypto Box Info. Used to get `encrypted_secret` that should be used for all the cryptobox initializations except the first one.

[get_crypto_box_seed_phrase](mod\_crypto.md#get_crypto_box_seed_phrase) – Get Crypto Box Seed Phrase.

[get_signing_box_from_crypto_box](mod\_crypto.md#get_signing_box_from_crypto_box) – Get handle of Signing Box derived from Crypto Box.

[get_encryption_box_from_crypto_box](mod\_crypto.md#get_encryption_box_from_crypto_box) – Gets Encryption Box from Crypto Box.

[clear_crypto_box_secret_cache](mod\_crypto.md#clear_crypto_box_secret_cache) – Removes cached secrets (overwrites with zeroes) from all signing and encryption boxes, derived from crypto box.

[register_signing_box](mod\_crypto.md#register_signing_box) – Register an application implemented signing box.

[get_signing_box](mod\_crypto.md#get_signing_box) – Creates a default signing box implementation.

[signing_box_get_public_key](mod\_crypto.md#signing_box_get_public_key) – Returns public key of signing key pair.

[signing_box_sign](mod\_crypto.md#signing_box_sign) – Returns signed user data.

[remove_signing_box](mod\_crypto.md#remove_signing_box) – Removes signing box from SDK.

[register_encryption_box](mod\_crypto.md#register_encryption_box) – Register an application implemented encryption box.

[remove_encryption_box](mod\_crypto.md#remove_encryption_box) – Removes encryption box from SDK

[encryption_box_get_info](mod\_crypto.md#encryption_box_get_info) – Queries info from the given encryption box

[encryption_box_encrypt](mod\_crypto.md#encryption_box_encrypt) – Encrypts data using given encryption box Note.

[encryption_box_decrypt](mod\_crypto.md#encryption_box_decrypt) – Decrypts data using given encryption box Note.

[create_encryption_box](mod\_crypto.md#create_encryption_box) – Creates encryption box with specified algorithm

## [abi](mod\_abi.md) – Provides message encoding and decoding according to the ABI specification.

[encode_message_body](mod\_abi.md#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](mod\_abi.md#attach_signature_to_message_body)

[encode_message](mod\_abi.md#encode_message) – Encodes an ABI-compatible message

[encode_internal_message](mod\_abi.md#encode_internal_message) – Encodes an internal ABI-compatible message

[attach_signature](mod\_abi.md#attach_signature) – Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

[decode_message](mod\_abi.md#decode_message) – Decodes message body using provided message BOC and ABI.

[decode_message_body](mod\_abi.md#decode_message_body) – Decodes message body using provided body BOC and ABI.

[encode_account](mod\_abi.md#encode_account) – Creates account state BOC

[decode_account_data](mod\_abi.md#decode_account_data) – Decodes account data using provided data BOC and ABI.

[update_initial_data](mod\_abi.md#update_initial_data) – Updates initial account data with initial values for the contract's static variables and owner's public key. This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

[encode_initial_data](mod\_abi.md#encode_initial_data) – Encodes initial account data with initial values for the contract's static variables and owner's public key into a data BOC that can be passed to `encode_tvc` function afterwards.

[decode_initial_data](mod\_abi.md#decode_initial_data) – Decodes initial values of a contract's static variables and owner's public key from account initial data This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

[decode_boc](mod\_abi.md#decode_boc) – Decodes BOC into JSON as a set of provided parameters.

[encode_boc](mod\_abi.md#encode_boc) – Encodes given parameters in JSON into a BOC using param types from ABI.

[calc_function_id](mod\_abi.md#calc_function_id) – Calculates contract function ID by contract ABI

[get_signature_data](mod\_abi.md#get_signature_data) – Extracts signature from message body and calculates hash to verify the signature

## [boc](mod\_boc.md) – BOC manipulation module.

[parse_message](mod\_boc.md#parse_message) – Parses message boc into a JSON

[parse_transaction](mod\_boc.md#parse_transaction) – Parses transaction boc into a JSON

[parse_account](mod\_boc.md#parse_account) – Parses account boc into a JSON

[parse_block](mod\_boc.md#parse_block) – Parses block boc into a JSON

[parse_shardstate](mod\_boc.md#parse_shardstate) – Parses shardstate boc into a JSON

[get_blockchain_config](mod\_boc.md#get_blockchain_config) – Extract blockchain configuration from key block and also from zerostate.

[get_boc_hash](mod\_boc.md#get_boc_hash) – Calculates BOC root hash

[get_boc_depth](mod\_boc.md#get_boc_depth) – Calculates BOC depth

[get_code_from_tvc](mod\_boc.md#get_code_from_tvc) – Extracts code from TVC contract image

[cache_get](mod\_boc.md#cache_get) – Get BOC from cache

[cache_set](mod\_boc.md#cache_set) – Save BOC into cache or increase pin counter for existing pinned BOC

[cache_unpin](mod\_boc.md#cache_unpin) – Unpin BOCs with specified pin defined in the `cache_set`. Decrease pin reference counter for BOCs with specified pin defined in the `cache_set`. BOCs which have only 1 pin and its reference counter become 0 will be removed from cache

[encode_boc](mod\_boc.md#encode_boc) – Encodes bag of cells (BOC) with builder operations. This method provides the same functionality as Solidity TvmBuilder. Resulting BOC of this method can be passed into Solidity and C++ contracts as TvmCell type.

[get_code_salt](mod\_boc.md#get_code_salt) – Returns the contract code's salt if it is present.

[set_code_salt](mod\_boc.md#set_code_salt) – Sets new salt to contract code.

[decode_tvc](mod\_boc.md#decode_tvc) – Decodes tvc into code, data, libraries and special options.

[encode_tvc](mod\_boc.md#encode_tvc) – Encodes tvc from code, data, libraries ans special options (see input params)

[encode_external_in_message](mod\_boc.md#encode_external_in_message) – Encodes a message

[get_compiler_version](mod\_boc.md#get_compiler_version) – Returns the compiler version used to compile the code.

## [processing](mod\_processing.md) – Message processing module.

[send_message](mod\_processing.md#send_message) – Sends message to the network

[wait_for_transaction](mod\_processing.md#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](mod\_processing.md#process_message) – Creates message, sends it to the network and monitors its processing.

## [utils](mod\_utils.md) – Misc utility Functions.

[convert_address](mod\_utils.md#convert_address) – Converts address from any TON format to any TON format

[get_address_type](mod\_utils.md#get_address_type) – Validates and returns the type of any TON address.

[calc_storage_fee](mod\_utils.md#calc_storage_fee) – Calculates storage fee for an account over a specified time period

[compress_zstd](mod\_utils.md#compress_zstd) – Compresses data using Zstandard algorithm

[decompress_zstd](mod\_utils.md#decompress_zstd) – Decompresses data using Zstandard algorithm

## [tvm](mod\_tvm.md)

[run_executor](mod\_tvm.md#run_executor) – Emulates all the phases of contract execution locally

[run_tvm](mod\_tvm.md#run_tvm) – Executes get-methods of ABI-compatible contracts

[run_get](mod\_tvm.md#run_get) – Executes a get-method of FIFT contract

## [net](mod\_net.md) – Network access.

[query](mod\_net.md#query) – Performs DAppServer GraphQL query.

[batch_query](mod\_net.md#batch_query) – Performs multiple queries per single fetch.

[query_collection](mod\_net.md#query_collection) – Queries collection data

[aggregate_collection](mod\_net.md#aggregate_collection) – Aggregates collection data.

[wait_for_collection](mod\_net.md#wait_for_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](mod\_net.md#unsubscribe) – Cancels a subscription

[subscribe_collection](mod\_net.md#subscribe_collection) – Creates a collection subscription

[subscribe](mod\_net.md#subscribe) – Creates a subscription

[suspend](mod\_net.md#suspend) – Suspends network module to stop any network activity

[resume](mod\_net.md#resume) – Resumes network module to enable network activity

[find_last_shard_block](mod\_net.md#find_last_shard_block) – Returns ID of the last block in a specified account shard

[fetch_endpoints](mod\_net.md#fetch_endpoints) – Requests the list of alternative endpoints from server

[set_endpoints](mod\_net.md#set_endpoints) – Sets the list of endpoints to use on reinit

[get_endpoints](mod\_net.md#get_endpoints) – Requests the list of alternative endpoints from server

[query_counterparties](mod\_net.md#query_counterparties) – Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

[query_transaction_tree](mod\_net.md#query_transaction_tree) – Returns a tree of transactions triggered by a specific message.

[create_block_iterator](mod\_net.md#create_block_iterator) – Creates block iterator.

[resume_block_iterator](mod\_net.md#resume_block_iterator) – Resumes block iterator.

[create_transaction_iterator](mod\_net.md#create_transaction_iterator) – Creates transaction iterator.

[resume_transaction_iterator](mod\_net.md#resume_transaction_iterator) – Resumes transaction iterator.

[iterator_next](mod\_net.md#iterator_next) – Returns next available items.

[remove_iterator](mod\_net.md#remove_iterator) – Removes an iterator

[get_signature_id](mod\_net.md#get_signature_id) – Returns signature ID for configured network if it should be used in messages signature

## [debot](mod\_debot.md) – [UNSTABLE](UNSTABLE.md) Module for working with debot.

[init](mod\_debot.md#init) – [UNSTABLE](UNSTABLE.md) Creates and instance of DeBot.

[start](mod\_debot.md#start) – [UNSTABLE](UNSTABLE.md) Starts the DeBot.

[fetch](mod\_debot.md#fetch) – [UNSTABLE](UNSTABLE.md) Fetches DeBot metadata from blockchain.

[execute](mod\_debot.md#execute) – [UNSTABLE](UNSTABLE.md) Executes debot action.

[send](mod\_debot.md#send) – [UNSTABLE](UNSTABLE.md) Sends message to Debot.

[remove](mod\_debot.md#remove) – [UNSTABLE](UNSTABLE.md) Destroys debot handle.

## [proofs](mod\_proofs.md) – [UNSTABLE](UNSTABLE.md) Module for proving data, retrieved from TONOS API.

[proof_block_data](mod\_proofs.md#proof_block_data) – Proves that a given block's data, which is queried from TONOS API, can be trusted.

[proof_transaction_data](mod\_proofs.md#proof_transaction_data) – Proves that a given transaction's data, which is queried from TONOS API, can be trusted.

[proof_message_data](mod\_proofs.md#proof_message_data) – Proves that a given message's data, which is queried from TONOS API, can be trusted.

