# Release Notes
All notable changes to this project will be documented in this file.

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
