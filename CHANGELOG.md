# Release Notes
All notable changes to this project will be documented in this file.

## 1.1.0 Oct 30, 2020

### New
- ChaCha20 encryption support `crypto.chacha20`. 
- `boc.parse_shardstate` function for shardstates parsing.
- `client.build_info` fully defined and documented.
- `processing.wait_for_transaction` and `processing.process_message` functions execute contract 
locally in case if transaction waiting fails in order to resolve the contract execution error

### Fixed
- TS generator fix some field names that is an invalid JS identifiers.   

### Breaking
- `Abi::Serialized` renamed to `Abi::ContractAbi` 

## 1.0.0 Oct 27, 2020

### Differences between Core SDK v0 and v1

- All api functions are defined in mod.rs of appropriate modules
- Function names are the same as API function names: `module.function_name`
- Parameters naming:
    - In snake case
    - Base64 suffix is removed from parameter names. For example, `bocBase64` is changed to `boc`
- Parsed boc replaced with unparsed boc in all function input parameters
- All functions take byte arrays  in a defined encoding:
    - `base64` - encoding used for byte arrays of variable length: text, images, etc.
    - `hex-lower-case` - encoding used to encode fixed length bit sequences: hashes, keys, salt, etc.
- `contracts` module is splitted into 5 modules:
    - `tvm` - embedded TVM execution functions
    - `boc` - raw cell and BOC manipulation functions
    - `abi` - abi-compatible messages creation and parsing functions
    - `processing` - blockchain interaction functions
    - `utils` - has only `convert_address` ATM, later will be used for some useful stuff
- `query` module is renamed to `net`
- new `client` module with functions `version`, `api_reference`
- All the environment functions (fetch, websocket, spawn, now, etc.) were abstracted behind a separate environment layer crate `ClientEnv`. The standard core env layer implementation is in `std_client_env` . Later (in 1.1 release) `web_client_env` implementation for Web will be added.
- Error codes are distributed across the modules the following way: `client` - 0..99, `crypto` - 100..199, `boc` - 200..299,  `abi`  - 300..399, `tvm` - 400..499, `processing` - 500..599, `net` - 600..699
- Error descriptions related to a module are described in error.rs file in the module's folder
- `decode_message`, `process_message`, `wait_for_transaction`, `run_tvm`, `run_executor`, etc.  (all the functions that return decoded messages) now returns int*/uint* data as a string which can be either decimal or 0x-prefixed hex string. Hex representation can be in any register and have any number of leading zeroes.

### Featured

- All the functions are asynchronous
- All the functions that can be called via JSON-api are public, so that they can be used directly without JSON-api.
- Inline documentation and api reference added.
- [breaking] **interops.rs**, **tonclient.h**. `create_context` now takes `config` parameter - context creation and setup happen at the same time. Config structure has been changed.
- [breaking] **crypto module.** default values for mnemonic-related functions have been changed:

    dictionary is 1, for word count is 12,  derivation path is 'm/44'/396'/0'/0/0

- [breaking] **crypto module.** removed `word_count` parameter from `words` function
- [breaking] **crypto module.** `compliant` parameter is removed from functions `mnemonic_derive_sign_keys`, `hdkey_xprv_derive_path`, `hdkey_xprv_derive`,
- [new] **boc module.** Functions `parse_block`, `parse_account`, `parse_message`, `parse_transaction` that parse bocs to JSONs are introduced.
- [breaking] **net module.** Functions `query` , `wait.for`, `subscribe`  are renamed to `query_collection`, `wait_for_collection`, `subscribe_collection`

    `table` parameter is renamed to `collection`. `filter` parameter is now optional and of `json` type (passed as json object instead of `string`)

- [breaking] **net module**. Function `get.next` is removed.
- [breaking] **net module.**`subscribe_collection` now uses callback to return data.
- [breaking] **abi module**. `decode_message` introduced instead of `decode_unknown_run`, `decode_run_output`
- [breaking] **abi module**. `encode_message` introduced instead of `encode_unsigned_deploy_message`, `encode_unsigned_run_message`, `run.encode_message`, `deploy.encode_message`
- [breaking] **abi module**. `signer: Signer` parameter used instead of `key_pair: KeyPair` , which can be of `None` (unsigned message will be produced), `External` (data to be signed +unsigned message), `Keys` (signed message will be produced), `SigningBox` (message will be signed using a provided interface - will be supported in coming releases)
- [breaking] **processing module.** `process_message`  introduced instead of `deploy` and `run`**.** Parameter set was drastically changed.
- [breaking] **processing module.** `process_message`   - now, if the contract was already deployed, deploy fails with an exception of double constructor call.
- [new] **processing module.** `process_message` - any function can be called at deploy, not only constructor, also there can be no function call.
- [new] **processing module.** `process_message` now can optionally use callback to monitor message processing (creation, sending, shard block fetching, transaction receiving).
- [fixed] **processing module.** `process_message` - deploy can be performed without a key pair
- [breaking] **tvm module.** `run_local` is divided into 2 functions `run_tvm` and `run_executor`.
- [new] **tvm module.** `run_tvm` function - performs contract code execution on tvm (part of compute phase). Helps to run contract methods without ACCEPT.  Returns account state with updated data, list of external messages and (optional, for ABI contracts only) list of messages decoded data.
- [new] **tvm module.** `run_executor` function - performs full contract code execution on Transaction Executor (part of collator protocol that performs all phases and checks and - as a successful result - forms a transaction) Returns updated account state,  parsed transaction, list of parsed messages with optional decoded message bodies.
- [breaking] **tvm module**. `run_get` does not download account boc from the network anymore, but takes account boc as a parameter.

## 0.26.0 Aug 15, 2020
### New
- `config.get_api_reference` api function (pre release).
- `ton_sdk_cli` cli tool (pre release).
- full local run functions use `LocalRunContext` to exactly reproduce all transaction parameters and
produce the same result as node

## 0.25.4 Aug 5, 2020
### Fixed
- `waitForTransaction` didn't use prev_alt_ref for block walking
 
## 0.25.3 Jul 30, 2020
### New
- All methods that require contract's code/data can use field `boc` 
  in account document to extract code and data 
  (instead of `code` and `data` fields).
- Optional `bocBase64` parameter of method `tvm.get` that can be used 
  instead of `codeBase64` and `dataBase64`.   

## 0.25.2 Jul 29, 2020
### New
- `error.data` object extended with fields `address`, `function_name`, `account_balance`, 
`account_address`, `query_url`, `config_server` for appropriate errors

## 0.25.0 Jul 8, 2020
### New
- supports for core context in all platforms
- local run functions return updated contract state when running with `full_run = true`
- time sync check while initializing
- parallel requests on different contexts don't block each other. Requests on the same context
remain sequential
- new transaction wait mechanism. All account's shard blocks are checked for transaction to 
guarantee message expiration
- `contracts.wait.transaction` function for awaiting previously sent message processing
- `contracts.send.message` returns message processing state for `contracts.wait.transaction` function
- `contracts.find.shard` function for account shard matching
- added logging on warning messages

## May 28, 2020
### New
- error resolving by local message processing
- `contracts.resolve.error` function for manual error resolving call
- `contracts.process.transaction` function processing transaction to check errors and get output
- `contracts.run.local` and `contracts.run.local` functions now have `fullRun` flag to emulate
node transaction processing and calculate fees
- `tonsdk` command line tool.
- `ton_client` function `get_method_names`.

## May 22, 2020
#### Fix
- TON mnemonic functions didn't check validity of the seed phrase.

## May 19, 2020
### ton-client-web 0.23.1
#### New
- Platform builder generates ready to use `index.js` for web clients (instead of install script of `ton-client-web-js` binding)

## May 17, 2020
### New
- `tvm.get` now can fetch account data if it is not provided

## May 14, 2020
### New
- Message processing functions added
- Run get methods function added
- `ed25519-dalek` version updated to `1.0.0-pre.3`
- SDK is fully opensourced since open repo `ton-labs-executor` used

### Fixed
- Panic in fee calculation under WASM
- `reqwest` crate version synced in all projects
- Memory leaking in Node JS
