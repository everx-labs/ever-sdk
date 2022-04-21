# Release Notes

All notable changes to this project will be documented in this file.

## [1.33.0] – 2022-04-21

### New 

- `allow_partial` flag in all `abi.decode_*` functions. This flag controls decoder behaviour whether to 
  return error or partially decoded BOC in case of incomplete BOC decoding.
- `tvm.run_responsible` function, that allows to run responsible methods (including get-methods) off-chain.

## [1.32.0] – 2022-03-22

### New 

- `network.queries_protocol` config parameter allows selecting protocol the SDK uses to communicate 
  with GraphQL endpoint:
  - `HTTP` – SDK performs single HTTP-request for each request.
  - `WS` – SDK uses single WebSocket connection to send all requests. This protocol is a preferable
    way when the application sends many GraphQL requests in parallel. 

### Fixed
- **Debot module**:
    - If DEngine received a non-zero exit_code while emulating a transaction while sending a message, 
      DEngine will call onErrorId callback of the message.
    
## [1.31.0] – 2022-03-09

### New 

**crypto module:**
- `Cryptobox` introduced: root crypto object that stores encrypted secret and acts as a factory for 
  all crypto primitives used in SDK.
  Crypto box provides signing and encryption boxes.

  Functions:
  [`create_crypto_box`](./docs/reference/types-and-methods/mod_crypto.md#create_crypto_box) - initializes cryptobox with secret
  [`remove_crypto_box`](./docs/reference/types-and-methods/mod_crypto.md#remove_crypto_box) - removes cryptobox and overwrites all secrets with zeroes
  [`get_crypto_box_seed_phrase`](./docs/reference/types-and-methods/mod_crypto.md#get_crypto_box_seed_phrase) - returns decrypted seed phrase
  [`get_crypto_box_info`](./docs/reference/types-and-methods/mod_crypto.md#get_crypto_box_info) - returns encrypted cryptobox secret for next cryptobox initializations
  [`get_signing_box_from_crypto_box`](./docs/reference/types-and-methods/mod_crypto.md#get_signing_box_from_crypto_box) - derives signing box from secret
  [`get_encryption_box_from_crypto_box`](./docs/reference/types-and-methods/mod_crypto.md#get_encryption_box_from_crypto_box) - derives encryption box from secret
  [`clear_crypto_box_secret_cache`](./docs/reference/types-and-methods/mod_crypto.md#clear_crypto_box_secret_cache) - forces secret cache (signing and encryption) clean up (overwrites all secrets with zeroes).

- Support of `initCodeHash` in tvm: `X-Evernode-Expected-Account-Boc-Version`=2 http header added to graphql requests. value=2 means that SDK requests the latest account boc version with initCodeHash support from API. This was done because new boc version is not compatible with old transaction executor, i.e. with previous SDK versions. New transaction executor is compatible with new and old boc formats. New SDK asks for new boc format if it exists,if it does not - old boc version is returned. Previous version of SDK will fetch only old version without initCodeHash support.

**Attention! Migrate your applications to the new SDK version ASAP because in some period of time we will stop supporting old boc version in API and default value of X-Evernode-Expected-Account-Boc-Version on backend side will become 2, missing value will cause an error. Now default value on API side is Null which means that API returns the old version. This is done to avoid breaking changes in existing applications and give time to migrate to the new SDK**.

### Fixed
- Documentation generator for app object interface fills documentation from
  `ParamsOfXXXAppObject` enum.
- Documentation generator for function with `obj` parameter add this parameter
  into parameters section with link to appropriate AppObject interface.

## [1.30.0] – 2022-02-04

### New
- Added `boc.encode_external_in_message` function to encode message BOC based on
  a low level message parts such as a body, state init etc.
- Added `net.subscribe` function to start a low level GraphQL subscription.
- Added support for new MYCODE TVM command in `tvm.run_tvm` and `tvm.run_get` functions.

## [1.29.0] – 2022-02-03

### New
- Added `abi.encode_boc` function to encode parameters with values to BOC, using ABI types.
- Added support of `address` type in `boc.encode_boc`.
- All fetch requests are now called with timeouts to prevent freezing in case of infinite answer.
- Support of `MYCODE` instruction in TVM

## [1.28.1] – 2022-01-25

### Fixed
- Support breaking changes in `ton-labs-block-json` v0.7.1
- Updated endpoints for `main.ton.dev` alias.

## [1.28.0] – 2021-12-24

### New
- DevNet endpoints now changed to EVER OS domain: eri01.net.everos.dev, rbx01.net.everos.dev, gra01.net.everos.dev
- **Debot module**:
    - Аdded float numbers support for Json interface
- Added guide for custom giver usage.

### Fixed
- Debot module: fixed a bug in Query.query function called with an empty `variables` field.


## [1.27.1] – 2021-12-09

### Fixed
- Empty `function_name` field in the "create run message failed" error.

## [1.27.0] – 2021-12-06

### New
- Function [`abi.encode_initial_data`](./docs/reference/types-and-methods/mod_abi.md#encode_initial_data) which
  encodes initial account data with initial values for the contract's static variables and owner's public key.
  This function is analogue of `tvm.buildDataInit` function in Solidity.

### Fixed
- Subscription for Counterparties failed with 'Unknown type "CounterpartieFilter"' error.

## [1.26.1] – 2021-12-01

### Fixed
- Fixed building and warning.

## [1.26.0] – 2021-11-25

### New
- **Debot module**:
    - Аdded `allow_no_signature` parameter to `decode_and_fix_ext_msg()` and
      `onerror_id` return value to `prepare_ext_in_message()` inner functions used in TS4.
    - Added support for async external calls.
    - `Query` interface extended with `waitForCollection` and `query` methods. `waitForCollection` allows to wait
     for completion of async external calls.
    - Added support for DeBots with ABI 2.2.
- Function [`proofs.proof_message_data`](./docs/reference/types-and-methods/mod_proofs.md#proof_message_data) which proves message data, retrieved
  from Graphql API.

## [1.25.0] – 2021-11-08

### New
- New module [`proofs`](./docs/mod_proofs.md) is introduced!
- Functions [`proofs.proof_block_data`](./docs/mod_proofs.md#proof_block_data) and [`proofs.proof_transaction_data`](./docs/mod_proofs.md#proof_transaction_data)
  which prove block data, retrieved from Graphql API.

  These are the first functions from proofs series :) Wait for others(`proof_account_data`, `proof_message_data`) in the next releases.

  Read about them more in the [documentation](./docs/mod_proofs.md#proof_block_data).

- [`abi.decode_boc`](./docs/mod_abi.md#decode_boc) function to decode custom BOC data into JSON parameters.
- `Ref(<ParamType>)` type was added to ABI.
  Solidity functions use ABI types for builder encoding. The simplest way to decode such a BOC is to use ABI decoding. ABI has it own rules for fields layout in cells so manually encoded BOC can not be described in terms of ABI rules. To solve this problem we introduce a new ABI type `Ref(<ParamType>)` which allows to store `ParamType` ABI parameter in cell reference and, thus, decode manually encoded BOCs. This type is available only in `decode_boc` function and will not be available in ABI messages encoding until it is included into some ABI revision.

## [1.24.0] – 2021-10-18

### New
- `boc.get_boc_depth` function to get depth of the provided boc.
- `boc.decode_tvc` function returns additional fields `code_hash`, `code_depth`, `data_hash`, `data_depth` and `compiler_version`

- **Debot module**:
    - added `parse` function to Json interface.

## [1.23.0] – 2021-10-05

### New
- `boc.get_code_salt` and `boc.set_code_salt` functions for contract code salt management.
- `boc.encode_tvc` and `boc.decode_tvc` functions for TVC image encoding and decoding
- `boc.get_compiler_version` function extracting compiler version from contract code
- `abi.update_initial_data` and `abi.decode_initial_data` function for pre-deployment contract data management

## [1.22.0] – 2021-09-20

### New
- ABI v2.2 with fixed message body layout supported. [See the specification](https://github.com/tonlabs/ton-labs-abi/blob/master/docs/ABI_2.2_spec.md).

  Now, for contracts with ABI version < 2.2  compact layout will still be used for compatibility, for contracts with ABI version 2.2 and more - fixed layout will be used.
**Please, make sure that you updated the ABI if you recompiled your contract with 2.2 ABI, or you may get an inconsistent contract behaviour**.
- **Debot module**:
    - added `getEncryptionBoxInfo`, `getSigningBoxInfo` functions to Sdk interface.
    - implemented Query DeBot interface in DEngine.

## [1.21.5] – 2021-09-13

### Fixed
- `abi.encode_message` and `processing.process_message` created invalid deploy message in case of
`Signer::None` was used, and contract could not be deployed.

## [1.21.4] – 2021-09-08

### New
- Support MacOS aarch64 target

## [1.21.3] – 2021-09-02

### New
- Information about used endpoint is added to subscription errors.
- Graphql response error codes 500-599 are treated as retriable network errors

## [1.21.2] – 2021-08-25

### Fixed
- Updated crypto libraries in order to fix building.

## [1.21.1] – 2021-08-24

### Fixed
- http errors were not processed as network errors and didn't lead to endpoint reconnect and request retry

## [1.21.0] – 2021-08-18

### New
- `crypto.create_encryption_box` function for creating SDK-defined encryption boxes. First supported
algorithm - AES with CBC mode.
- **Debot module**:
    - Аdded public `prepare_ext_in_message` function.

### Fixed
- `tvm.run_executor` did not work when SDK is configured to use TONOS SE, because of incomplete default
blockchain configuration. Now mainnet config from key block 10660619 (last key block at the moment of fix)
is used as default.

## [1.20.1] – 2021-07-30

### New
- Added support of contract error messages. Error messages (for example, require(...) in Solidity) are now parsed by SDK
  and returned in error message. New field `contract_error` was added to error's `data`.

### Fixed
- Fixed problem with WASM binaries (https://github.com/tonlabs/ton-labs-types/pull/42)

## [1.20.0] – 2021-07-16

### New
- ABI version `2.1` supported.
  **Attention!**
  If you work with contracts, that contain String parameters, then during migration from ABI 2.0 to 2.1 you will need to remove all String type conversions to bytes and back and pass string to your contract as is.

- Now all requests to GraphQL are limited with timeout to react on unexpected server unavailability.
Existing timeouts in waiting functions keep the same behaviour. All other requests timeout now can
be set with `net.query_timeout` config parameter. Its default value is 60000 ms
- **Debot module**:
    - added `encrypt`, `decrypt` functions to Sdk interface which accept encryption box handles.

### Fixed
- Deployment with empty signer in cases of public key set in TVC or deploy set.

## [1.19.0] – 2021-07-07

### New
- `get_address_type` function in `utils` module, which validates address and returns its type. See the documentation.
- `decode_account_data` function in `abi` module that converts account data BOC into JSON representation according to ABI 2.1. See the documentation.
- Diagnostic fields `filter` and `timestamp` added to `wait_for_collection` error
- `main.ton.dev` and `net.ton.dev` endpoints that will be deprecated on 12.07.21 are now replaced with [proper endpoints list](https://docs.ton.dev/86757ecb2/p/85c869-networks), if they were specified in network `endpoints` config

### Fixed
- Search of the first master blocks during the network start period was fixed in blocks and transactions iterators

## [1.18.0] – 2021-06-26

### New
- Iterators in `net` module: robust way to iterate blockchain items (blocks, transactions)
  in specified range. See documentation for `create_block_iterator` , `create_transaction_iterator`,
  `resume_block_iterator`, `resume_transaction_iterator`, `iterator_next`, `iterator_remove`
  functions.
- Library adds `http://` protocol to endpoints `localhost`, `127.0.0.1`, `0.0.0.0` if protocol
  isn't specified in config.
- **Debot module**:
    - added tests for Json interface.

## [1.17.0] – 2021-06-21

### New
- Added support of external encryption boxes. [See the documentation](docs/mod_crypto.md#register_encryption_box)
- **Debot module**:
    - Dengine waits for completion of all transactions in a chain initiated by debot's onchain call.

## [1.16.1] – 2021-06-16

### New
- `timeout` option to `query_transaction_tree` – timeout used to limit waiting time for the next
  message and transaction in the transaction tree.

### Improved
- Improved error messages regarding ABI and JSON interface. SDK now shows additional tips for the user in cases of
  errors.

### Fixed
- Warnings in Rust 1.52+. Little fixes in the documentation.
- `total_output` field in fees was always 0.
- `query_transaction_tree` didn't wait for messages.

## [1.16.0] – 2021-05-25

### New
- `query_transaction_tree` function that returns messages and transactions tree produced
  by the specified message was added to `net` module. [See the documentation](docs/mod_net.md#query_transaction_tree)

### Fixed
- `AbiData.key` type changed to u32.
- attempt to use `orderBy` instead of `order` in `query_collection` will raise error.

## [1.15.0] – 2021-05-18

### New
- Sync latency detection increases connection reliability. Library will change the current endpoint
  when it detects data sync latency on it.

- Configuration parameters: `latency_detection_interval`,
  `max_latency`. See client documentation for details.

- **Debot module**:
    - signing messages with signing box handles returned from debots.
    - return any sdk errors to debot in case of external calls.
    - defining signing box handle used to sign message in approve callback.

## [1.14.1] – 2021-04-29

### Fixed
- Fixed building under Rust versions older than 1.51.

## [1.14.0] – 2021-04-28

### New
- **Debot module**:
    - implementation of Network DeBot interface in DEngine.
    - implementation of `signHash` function in Sdk interface.

### Fixed
- **Debot module**:
    - fixed bug in Json interface with supporting nested structures and arrays of structures.
    - fixed bug in Json interface with keys containing hyphens.

## [1.13.0] – 2021-04-23

### New
- [`net.query_counterparties`](docs/mod_net.md#query_counterparties) - allows to query and paginate through the list of accounts that the specified account
 has interacted with, sorted by the time of the last internal message between accounts.
  Subscription to counterparties collection is available via `net.subscribe_collection` function.

- Blockchain interaction reliability improvement (broadcast): library sends external inbound messages simultaneously
  to the N randomly chosen endpoints. If all N endpoints failed to responce then library repeats
  sending to another random N endpoints (except the failed one).
  If all the available endpoints fail to respond then library throws error.
  The N parameter is taken from `config.network.sending_endpoint_count` (default is 2).

- Blockchain interaction reliability improvement (bad delivery list): library tracks endpoints
  with bad message delivery (expired messages). These endpoints have lower priority when library chooses endpoints
  to send message.

- **Debot module**:
    - Implementation of `Json` DeBot interface in DEngine.

### Fixed
- `BuilderOp::Integer.size` type has changed from `u8` to `u32`.
- **Debot Module**:
    - `Sdk` interface function `getAccountsDataByHash` didn't find accounts by `code_hash` with leading zero.

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
