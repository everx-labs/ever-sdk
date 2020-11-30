# Modules
## [client](mod_client.md) – Provides information about library.

[get_api_reference](mod_client.md#get_api_reference) – Returns Core Library API reference

[version](mod_client.md#version) – Returns Core Library version

[build_info](mod_client.md#build_info) – Returns detailed information about this build.

[resolve_app_request](mod_client.md#resolve_app_request) – Resolves application request processing result

## [crypto](mod_crypto.md) – Crypto functions.

[factorize](mod_crypto.md#factorize) – Performs prime factorization – decomposition of a composite number into a product of smaller prime integers (factors). See [https://en.wikipedia.org/wiki/Integer_factorization]

[modular_power](mod_crypto.md#modular_power) – Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`). See [https://en.wikipedia.org/wiki/Modular_exponentiation]

[ton_crc16](mod_crypto.md#ton_crc16) – Calculates CRC16 using TON algorithm.

[generate_random_bytes](mod_crypto.md#generate_random_bytes) – Generates random byte array of the specified length and returns it in `base64` format

[convert_public_key_to_ton_safe_format](mod_crypto.md#convert_public_key_to_ton_safe_format) – Converts public key to ton safe_format

[generate_random_sign_keys](mod_crypto.md#generate_random_sign_keys) – Generates random ed25519 key pair.

[sign](mod_crypto.md#sign) – Signs a data using the provided keys.

[verify_signature](mod_crypto.md#verify_signature) – Verifies signed data using the provided public key. Raises error if verification is failed.

[sha256](mod_crypto.md#sha256) – Calculates SHA256 hash of the specified data.

[sha512](mod_crypto.md#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](mod_crypto.md#scrypt) – Derives key from `password` and `key` using `scrypt` algorithm. See [https://en.wikipedia.org/wiki/Scrypt].

[nacl_sign_keypair_from_secret_key](mod_crypto.md#nacl_sign_keypair_from_secret_key) – Generates a key pair for signing from the secret key

[nacl_sign](mod_crypto.md#nacl_sign) – Signs data using the signer's secret key.

[nacl_sign_open](mod_crypto.md#nacl_sign_open)

[nacl_sign_detached](mod_crypto.md#nacl_sign_detached)

[nacl_box_keypair](mod_crypto.md#nacl_box_keypair)

[nacl_box_keypair_from_secret_key](mod_crypto.md#nacl_box_keypair_from_secret_key) – Generates key pair from a secret key

[nacl_box](mod_crypto.md#nacl_box) – Public key authenticated encryption

[nacl_box_open](mod_crypto.md#nacl_box_open) – Decrypt and verify the cipher text using the recievers secret key, the senders public key, and the nonce.

[nacl_secret_box](mod_crypto.md#nacl_secret_box) – Encrypt and authenticate message using nonce and secret key.

[nacl_secret_box_open](mod_crypto.md#nacl_secret_box_open) – Decrypts and verifies cipher text using `nonce` and secret `key`.

[mnemonic_words](mod_crypto.md#mnemonic_words) – Prints the list of words from the specified dictionary

[mnemonic_from_random](mod_crypto.md#mnemonic_from_random) – Generates a random mnemonic from the specified dictionary and word count

[mnemonic_from_entropy](mod_crypto.md#mnemonic_from_entropy) – Generates mnemonic from pre-generated entropy

[mnemonic_verify](mod_crypto.md#mnemonic_verify) – The phrase supplied will be checked for word length and validated according to the checksum specified in BIP0039.

[mnemonic_derive_sign_keys](mod_crypto.md#mnemonic_derive_sign_keys) – Validates the seed phrase, generates master key and then derives the key pair from the master key and the specified path

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

## [abi](mod_abi.md) – Provides message encoding and decoding according to the ABI specification.

[encode_message_body](mod_abi.md#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](mod_abi.md#attach_signature_to_message_body)

[encode_message](mod_abi.md#encode_message) – Encodes an ABI-compatible message

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

[get_blockchain_config](mod_boc.md#get_blockchain_config)

[get_boc_hash](mod_boc.md#get_boc_hash) – Calculates BOC root hash

## [processing](mod_processing.md) – Message processing module.

[send_message](mod_processing.md#send_message) – Sends message to the network

[wait_for_transaction](mod_processing.md#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](mod_processing.md#process_message) – Creates message, sends it to the network and monitors its processing.

## [utils](mod_utils.md) – Misc utility Functions.

[convert_address](mod_utils.md#convert_address) – Converts address from any TON format to any TON format

## [tvm](mod_tvm.md)

[run_executor](mod_tvm.md#run_executor)

[run_tvm](mod_tvm.md#run_tvm)

[run_get](mod_tvm.md#run_get) – Executes getmethod and returns data from TVM stack

## [net](mod_net.md) – Network access.

[query_collection](mod_net.md#query_collection) – Queries collection data

[wait_for_collection](mod_net.md#wait_for_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](mod_net.md#unsubscribe) – Cancels a subscription

[subscribe_collection](mod_net.md#subscribe_collection) – Creates a subscription

## [debot](mod_debot.md) – [UNSTABLE](UNSTABLE.md) Module for working with debot.

[start](mod_debot.md#start) – [UNSTABLE](UNSTABLE.md) Starts an instance of debot.

[fetch](mod_debot.md#fetch) – [UNSTABLE](UNSTABLE.md) Fetches debot from blockchain.

[execute](mod_debot.md#execute) – [UNSTABLE](UNSTABLE.md) Executes debot action.

[remove](mod_debot.md#remove) – [UNSTABLE](UNSTABLE.md) Destroys debot handle.

