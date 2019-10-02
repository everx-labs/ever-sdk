#![recursion_limit="128"] // needed for error_chain

#[macro_use]
extern crate tvm;
extern crate ton_abi_json;
extern crate ton_abi_core;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate hex;
extern crate ed25519_dalek;
extern crate rand;
extern crate sha2;

#[macro_use]
extern crate error_chain;
#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate serde_json;
#[cfg(feature = "node_interaction")]
extern crate futures;
#[cfg(feature = "node_interaction")]
extern crate graphite;

pub use ton_abi_json::json_abi;
pub use ton_abi_json::Contract as AbiContract;
pub use ton_abi_json::Function as AbiFunction;

#[allow(deprecated)]
#[macro_use]
mod error;
pub use error::*;

mod contract;
pub use contract::*;

mod message;
pub use message::*;

mod local_tvm;

#[cfg(feature = "node_interaction")]
mod transaction;
#[cfg(feature = "node_interaction")]
pub use transaction::*;
/*
#[cfg(feature = "node_interaction")]
mod block;
#[cfg(feature = "node_interaction")]
pub use block::*;
*/
#[cfg(feature = "node_interaction")]
mod types;
#[cfg(feature = "node_interaction")]
pub use types::*;

#[cfg(feature = "node_interaction")]
pub mod queries_helper;
#[cfg(feature = "node_interaction")]
mod requests_helper;

#[cfg(feature = "node_interaction")]
mod check_proofs;

/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init(default_workchain: Option<i32>, config: NodeClientConfig) -> SdkResult<()> {
    Contract::set_default_workchain(default_workchain);
    requests_helper::init(config.requests_config);
    queries_helper::init(config.queries_config);
    Ok(())
}

/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init_json(default_workchain: Option<i32>, config: &str) -> SdkResult<()> {
    init(default_workchain, serde_json::from_str(config)
        .map_err(|err| SdkErrorKind::InvalidArg(format!("{}", err)))?)
}

/// Uninit SKD. Should be called before process
#[cfg(feature = "node_interaction")]
pub fn uninit() {
    requests_helper::uninit();
    queries_helper::uninit();
}

#[cfg(test)]
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;
