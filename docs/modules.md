# Modules
## [client](mod_client.md) –  BOC manipulation module.

[get_api_reference](mod_client.md#get_api_reference)

[version](mod_client.md#version)

## [crypto](mod_crypto.md) –  Crypto functions.

[factorize](mod_crypto.md#factorize) – Integer factorization

[modular_power](mod_crypto.md#modular_power) – Modular exponentiation

[ton_crc16](mod_crypto.md#ton_crc16) –  Calculates CRC16 using TON algorithm.

[generate_random_bytes](mod_crypto.md#generate_random_bytes) – Generates random byte array of the specified length in the spesified encoding

[convert_public_key_to_ton_safe_format](mod_crypto.md#convert_public_key_to_ton_safe_format) –  Converts public key to ton safe_format

[generate_random_sign_keys](mod_crypto.md#generate_random_sign_keys) –  Generates random ed25519 key pair.

[sign](mod_crypto.md#sign) –  Signs a data using the provided keys.

[verify_signature](mod_crypto.md#verify_signature) –  Verifies signed data using the provided public key.

[sha256](mod_crypto.md#sha256) –  Calculates SHA256 hash of the specified data.

[sha512](mod_crypto.md#sha512) – Calculates SHA512 hash of the specified data.

[scrypt](mod_crypto.md#scrypt) – Perform `scrypt` encryption

[nacl_sign_keypair_from_secret_key](mod_crypto.md#nacl_sign_keypair_from_secret_key) –  Generates a key pair for signing from the secret key

[nacl_sign](mod_crypto.md#nacl_sign) –  Signs data using the signer's secret key.

[nacl_sign_open](mod_crypto.md#nacl_sign_open)

[nacl_sign_detached](mod_crypto.md#nacl_sign_detached)

[nacl_box_keypair](mod_crypto.md#nacl_box_keypair)

[nacl_box_keypair_from_secret_key](mod_crypto.md#nacl_box_keypair_from_secret_key)

[nacl_box](mod_crypto.md#nacl_box)

[nacl_box_open](mod_crypto.md#nacl_box_open)

[nacl_secret_box](mod_crypto.md#nacl_secret_box)

[nacl_secret_box_open](mod_crypto.md#nacl_secret_box_open)

[mnemonic_words](mod_crypto.md#mnemonic_words) –  Prints the list of words from the specified dictionary

[mnemonic_from_random](mod_crypto.md#mnemonic_from_random) – Generates a random mnemonic

[mnemonic_from_entropy](mod_crypto.md#mnemonic_from_entropy) – Generates mnemonic from the specified entropy

[mnemonic_verify](mod_crypto.md#mnemonic_verify) – Validates a mnemonic phrase

[mnemonic_derive_sign_keys](mod_crypto.md#mnemonic_derive_sign_keys) – Derives a key pair for signing from the seed phrase

[hdkey_xprv_from_mnemonic](mod_crypto.md#hdkey_xprv_from_mnemonic) –  Generate the extended master private key that will be the root for all the derived keys

[hdkey_derive_from_xprv](mod_crypto.md#hdkey_derive_from_xprv) – Derives the next child extended private key

[hdkey_derive_from_xprv_path](mod_crypto.md#hdkey_derive_from_xprv_path) – Derives the exented private key from the specified key and path

[hdkey_secret_from_xprv](mod_crypto.md#hdkey_secret_from_xprv) –  Extracts the private key from the serialized extended private key

[hdkey_public_from_xprv](mod_crypto.md#hdkey_public_from_xprv) –  Extracts the public key from the serialized extended private key

## [abi](mod_abi.md) –  Functions for encoding and decoding messages due to ABI

[encode_message](mod_abi.md#encode_message)

[attach_signature](mod_abi.md#attach_signature)

[decode_message](mod_abi.md#decode_message)

[encode_account](mod_abi.md#encode_account) –  Encodes account state as it will be

## [boc](mod_boc.md) –  BOC manipulation module.

[parse_message](mod_boc.md#parse_message)

[parse_transaction](mod_boc.md#parse_transaction)

[parse_account](mod_boc.md#parse_account)

[parse_block](mod_boc.md#parse_block)

[get_blockchain_config](mod_boc.md#get_blockchain_config)

## [processing](mod_processing.md) –  Message processing module.

[send_message](mod_processing.md#send_message)

[wait_for_transaction](mod_processing.md#wait_for_transaction) –  Performs monitoring of the network for a results of the external

[process_message](mod_processing.md#process_message) –  Sends message to the network and monitors network for a result of

## [utils](mod_utils.md) –  Misc utility Functions.

[convert_address](mod_utils.md#convert_address) –  Sends message to the network and monitors network for a result of

## [net](mod_net.md) –  Network access.

[query_collection](mod_net.md#query_collection)

[wait_for_collection](mod_net.md#wait_for_collection)

[unsubscribe](mod_net.md#unsubscribe)

[subscribe_collection](mod_net.md#subscribe_collection)

