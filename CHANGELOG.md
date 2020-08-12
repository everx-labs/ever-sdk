# Release Notes
All notable changes to this project will be documented in this file.

## 0.26.0 Aug 7, 2020
### New
- `crypto` function `crypto.derive_sign_keys_from_mnemonic`

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
