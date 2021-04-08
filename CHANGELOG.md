# Release Notes

All notable changes to this project will be documented in this file.

## [1.13.0] – 2021-04-01

### New
- `net.query_counterparties` funtion for quering account counterparties and last messages info. Subscrition to counterparties collection is available via `net.subscribe_collection` function.

## [1.12.0] – 2021-04-01

### New
- [`utils.compress_zstd`](docs/mod_utils.md#compress_zstd) compresses data using Facebook's Zstandard algorithm.
- [`utils.decompress_zstd`](docs/mod_utils.md#decompress_zstd) decompresses data using Facebook's Zstandard algorithm.
- **Debot module**:
    - `init` function that creates an instance of DeBot and returns DeBot metadata.
    - Dengine fetches metadata form DeBot by calling 2 mandatory functions: `getRequiredInterfaces` and `getDebotInfo`. This data is returned by `fetch` and `init` functions.
    - `approve` DeBot Browser callback which is called by DEngine to request permission for DeBot activities.

### Changed
- **Debot Module**:
    - [breaking] `fetch` function does't create an instance of debot. It returns DeBot metadata (`DebotInfo`).
    - [breaking] `start` function does't create an instance of debot. It accepts DeBot handle created in `init` function.

## [1.11.2] – 2021-03-19

### Refactor
- Some internal refactor due to `ton-block` changes

## [1.11.1] – 2021-03-15

### New
- Giver address in tests is calculated from secret key. Default values are provided for TON OS SE giver

## [1.11.0] – 2021-03-05

### New
- [`utils.calc_storage_fee`](docs/mod_utils.md#calc_storage_fee) function to calculate account storage fee over a some time period.
- **Debot Module**:
    - Added unstable functions to `Sdk` interface: `getAccountsDataByHash`

## [1.10.0] – 2021-03-04

### New
- Add optional field `src_address` to [`ParamsOfEncodeInternalMessage`](docs/mod_abi.md#encode_internal_message).
- Field `abi` in [`ParamsOfEncodeInternalMessage`](docs/mod_abi.md#encode_internal_message) is optional and can be `None` if `call_set` and `deploy_set` are  `None`.
- [`boc.encode_boc`](docs/mod_boc.md#encode_boc) function provides ability to build and serialize any custom tree of cells.
  Application can use several base Builder serialization primitives like integers, bitstrings
  and nested cells.
- [`boc.get_blockchain_config`](docs/mod_boc.md#get_blockchain_config) function can extract blockchain configuration from key block and also
from zerostate.
- [`tvm` module](docs/mod_tvm.md) functions download current blockchain configuration if `net` is initialized with
DApp Server endpoints. Otherwise [default configuration](https://github.com/tonlabs/ton-executor/blob/11f46c416ebf1f145eacfb996587891a0a3cb940/src/blockchain_config.rs#L214) is used.
- **Debot Module**:
    - Support for debot invoking in Debot Engine. `send` browser callback is used not only for interface calls but to invoke debots.
    - `start` and `fetch` functions returns debot ABI.
    - Added new built-in interface `Hex` which implements hexadecimal encoding and decoding.
    - Added unstable functions to `Sdk` interface: naclBox, naclBoxOpen, naclKeypairFromSecret, getAccountCodeHash.

### Changed
- Both `call_set` and `deploy_set` in [`ParamsOfEncodeInternalMessage`](docs/mod_abi.md#encode_internal_message) can be omitted. In this case `encode_internal_message` generates internal message with empty body.
- **Debot Module**:
    - `send` function accepts one argument - serialized internal message as string encoded into base64.
### Documentation
- [Debot browser app object](docs/mod_debot.md#AppDebotBrowser) and [signing box app object](docs/mod_crypto.md#appsigningbox) descriptions added
- functions-helpers for enum type variable creation for [Signer](docs/mod_abi.md#signer), [Abi](docs/mod_abi.md#abi), [ParamsOfAppDebotBrowser](mod_debot.md#paramsofappdebotbrowser)

### Fixed

-  doc generator: app object interface description, constructor functions-helpers for enum type variable creation, added new line in the end if api.json
- library libsecp256k1 upgraded to fix https://rustsec.org/advisories/RUSTSEC-2019-0027

## 1.9.0 Feb 19, 2021

### New

- `tuple_list_as_array` parameter in `tvm.run_get` function which controls lists representation.
Default is stack-like based on nested tuples. If set to `true` then returned lists are encoded as plain arrays.  Use this option if you receive this error on Web: "Runtime error. Unreachable code should not be executed..."
This reduces stack size requirements for long lists.
- `function_name` field of `CallSet` structure can be the name or **id (as string in hex starting with 0x)** of the called function.
- Fields `config_servers`, `query_url`, `account_address`, `gas_used` added into specific errors' `ClientError.data` object.

### Fixed

- Binaries download links are now under https protocol
- If you receive this error on Web: "Runtime error. Unreachable code should not be executed..." in `run_get`, use the new parameter `tuple_list_as_array = true`. [See the documentation](docs/mod_tvm.md#run_get). This may happen, for example, when elector contract contains too many participants

## 1.8.0 Feb 11, 2021

### New

- **Debot Module**:
    - Added new built-in interface `Msg` which allows to send external message to blockchain and sign it with supplied keypair.

### Fixed

- `crypto.hdkey_public_from_xprv` used compressed 33-byte form instead of normal 32-byte.

## 1.7.0 Feb 9, 2021

### New

- BOC cache management functions were introduced:
  - `boc.cache_set`,
  - `boc.cache_get`
  - `boc.cache_unpin`
- Now functions that take boc as a parameter can also take a reference to boc cash instead so that it deсreases the number of boc serialization
and deserializations which drastically improves performance of `run_tvm` and `run_executor` expecially in case of numerous calls on the same data.
- `boc_cache` parameter in `tvm.run_tvm` and `tvm.run_executor` functions to save resulting messages and account BOCs into cache.
- `return_updated_account` flag parameter introduced in `tvm.run_tvm` and `tvm.run_executor` functions to return updated account state. Important: by default this flag is `false` and account data is not returned.
- `abi.encode_internal_message` function to encode an internal ABI-compatible message.
- **Debot Module**:
    - Support for get-methods and external calls in debots.
    Debots can send external inbound messages to destination contracts (signed - for external calls and unsigned - for get-methods) using native language syntax without actions.
    - Built-in debot interfaces (interfaces implemented by DEngine).
    Added two built-in interfaces: base64 and Sdk.
    - Added `DebotInterfaceExecutor` to automatically route messages to destination interfaces.
    - Debot's `fetch` function is optional now. New debots can implement only `start` function.

## 1.6.3 Feb 4, 2021
### Fixed
- Expired message wasn't retried if local execution succeeded.

## 1.6.2 Feb 3, 2021
### Added
- `ResponseHandler` type description into `modules.md`.

### Fixed
- `net.batch_query` parameters serialization did't match to docs.
- Module description in docs generator contains `null` instead of summary.
- Function result section header hadn't the line separator before.

## 1.6.0 Jan 29, 2021
### New
- `nacl_sign_detached_verify` function to verify detached signature.
- `aggregate_collection` function as a wrapper for GraphQL aggregation queries.
- `batch_query` function performs multiple queries per single fetch.
- Active endpoint invalidation in case of network error occurring.
- `network.network_retries_count` config parameter is deprecated. `network.max_reconnect_timeout` is introduced that allows to specify maximum network resolving timeout. Default value is 2 min.
- `initial_pubkey` field in `DeploySet` to specify public key instead of one from TVC file or provided by signer.
- Support for debot interfaces:
  - `send` Browser Callback to send messages with interface calls to Browser.
  - new variant `ParamsOfAppDebotBrowser::Send`.
  - `send` API function to send messages from Browser to Debot.
  - `run_output.rs` - internal structure RunOutput to filter messages generated by debot to 4 categories: interface calls, external calls, get-method calls and invoke calls.

### Fixed
- Device time synchronization is checked only in `send_message`. Data querying does not require proper time now

## 1.5.2 Dec 30, 2020

### Fixed
- `net` module functions waits for `net.resume` call instead of returning error if called while the module is suspended

### Documentation
- How to work with `Application Objects` [specification](docs/app_objects.md) added

## 1.5.1 Dec 28, 2020

### Fixed
- Updated the dependence on `ton-labs-abi`

## 1.5.0 Dec 25, 2020

### New
- `reconnect_timeout` parameter in `NetworkConfig`.
- `endpoints` parameter in `NetworkConfig`. It contains the list of available server addresses to connect.
SDK will use one them with the least connect time. `server_address` parameter is still supported but
`endpoints` is prevailing.
- `net.fetch_endpoints` function to receive available endpoints from server.
- `net.set_endpoints` function to set endpoints list for using on next reconnect.
- `ErrorCode` type in each module spec in `api.json`.

### Fixed
- send `GQL_TERMINATE_CONNECTION` and close websocket on leaving ws loop.

## 1.4.0 Dec 18, 2020

### New
- GraphQL optimization: use single web socket to serve all subscriptions.
- Support for the `keep-alive` messages from the GraphQL server.
- `tonclient-core-version` http header.
- `net.find_last_shard_block` function returning account shard last block ID.
- `boc.get_code_from_tvc` function extracting contract code from TVC image.
- **Debot Module:**
  - Add new variant `ParamsOfAppDebotBrowser::SwitchCompleted` to notify browser when all context actions are shown.
  - Added new 3 engine routines for crypto operations and 1 routine for querying account state (balance, state type, code, data) that can be used in debots.

### Fixed

- **Debot Module:**
  - Invoked debot terminated correctly after error occurred during
execution of one of its actions. Initial prev_state of invoked debot
changed to STATE_EXIT.
  - Fixed double jumping to current context in invoker debot after
returning control to it from invoked debot.
  - Fixed conversation of exception codes thrown by debots to their user-friendly description.

## 1.3.0 Dec 8, 2020

### Featured
- `net.query` method . Performs custom graphql query that can be copied directly from the playground.
- `net.suspend` and `net.resume` methods for disabling and enabling network activity. One of the possible use-cases is to manage subscriptions when a mobile application is brought to the background and into the foreground again.
- Smart summary and description doc separation.
- ts-generator includes doc comments in JSDoc format.

## 1.2.0 Nov 26, 2020

### Featured
- **UNSTABLE API. This API is experimental. It can be changed in the next releases**.
`debot` module was added with debot engine functions, such as : `start`, `fetch`, `execute`, `remove`. See the `debot` module documentation for more info.
Check our tests for code examples.

- External signing was supported for message encoding: `SigningBox` type for `Signer` enum was supported.
  Now it is possible to sign messages with externally implemented signing box interface without private key disclosure to the library. Can be used in case of signing via HSM API or via cold wallet - when there is no access to the private key.

  It is also possible to create a Signing Box instance inside SDK - from a key pair passed into the library with `get_signing_box` method. It can be used for some test cases. Also it increases security - you need to pass your keys one time only.

  Check the `crypto` module documentation for `SigningBoxHandle` type and  `register_signing_box`, `get_signing_box`, `signing_box_get_public_key`, `signing_box_sign`.
  Check our tests for code examples.

### Fixed
- panic after `tc_destroy_context` call. Now all contexts use global async runtime
- field `mnemonic_hdkey_compliant` was removed from `CryptoConfig` (unused by the library)
- original and resolved errors are swapped in result. Now `error.code` contains original error code

## 1.1.2 Nov 15, 2020
### Fixed
- `wasm` feature has been fixed
- `crypto.factorize` doesn't panic on invalid challenge
- `client.get_api_reference` returns proper version
- ABI JSON with explicit function ID is parsed properly

## 1.1.1 Nov 11, 2020
### Fixed
- Compatible with older rust version change api type derivation with `vec![]`
instead of prev `[].into()`

## 1.1.0 Nov 3, 2020

### New
- ChaCha20 encryption support `crypto.chacha20`.
- `boc.parse_shardstate` function for shardstates parsing.
- `boc.get_boc_hash` function for calculating BOC root hash
- `client.build_info` fully defined and documented.
- `processing.wait_for_transaction` and `processing.process_message` functions execute contract
locally in case if transaction waiting fails in order to resolve the contract execution error
- `run_executor`, `run_tvm` now return `exit_arg` in case of TVM errors.
- Create the `build_info.json` on the build stage.
- `Abi::Contract` variant as an alias to deprecated `Abi::Serialized`
- `Abi::Json` variant to specify an ABI as a raw JSON string.
- `api.json` now contains details about numeric types: Number and BigInt are now
have new fields `number_type` and `number_size`.
- `api.json` ref type names are fully qualified now in form of `module.type`,
for example `abi.Signer`.

### Fixed
- TS generator fix some field names that is an invalid JS identifiers.
- Use `install_name_tool` to fix loading library paths at `libton_client.dylib`.
- `api.json` is reduced, so it can't contains tuple types, only structs.
All types are exactly match to JSON.
- `out_of_sync_threshold` config parameter is `u32`

### Unstable
- `tc_request_ptr` function to use pointers `void*` instead of request_id `u32`.
This feature is **UNSTABLE** yet.

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
- [breaking] `interops.rs`, `tonclient.h`. `create_context` now takes `config` parameter - context creation and setup happen at the same time. Config structure has been changed.
- [breaking] `crypto module.` default values for mnemonic-related functions have been changed:

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
- SDK is fully open sourced since open repo `ton-labs-executor` used

### Fixed
- Panic in fee calculation under WASM
- `reqwest` crate version synced in all projects
- Memory leaking in Node JS
