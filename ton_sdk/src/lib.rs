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
#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate serde_json;
#[cfg(feature = "node_interaction")]
extern crate kafka;
#[cfg(feature = "node_interaction")]
extern crate reql;
#[cfg(feature = "node_interaction")]
extern crate reql_types;
#[cfg(feature = "node_interaction")]
extern crate futures;

#[allow(deprecated)]
#[macro_use]
mod error;
pub use error::*;

mod contract;
pub use contract::*;

#[cfg(feature = "node_interaction")]
mod message;
#[cfg(feature = "node_interaction")]
pub use message::*;

#[cfg(feature = "node_interaction")]
mod transaction;
#[cfg(feature = "node_interaction")]
pub use transaction::*;

#[cfg(feature = "node_interaction")]
mod block;
#[cfg(feature = "node_interaction")]
pub use block::*;

#[cfg(feature = "node_interaction")]
mod types;
#[cfg(feature = "node_interaction")]
pub use types::*;

#[cfg(feature = "node_interaction")]
pub mod db_helper;
#[cfg(feature = "node_interaction")]
mod kafka_helper;
#[cfg(feature = "node_interaction")]
mod local_tvm;

/// Init SKD. Connects to Kafka and Rethink DB.
#[cfg(feature = "node_interaction")]
pub fn init(default_workchain: Option<i32>, config: NodeClientConfig) -> SdkResult<()> {
    Contract::set_default_workchain(default_workchain);
    kafka_helper::init(config.kafka_config)?;
    db_helper::init(config.db_config)
}

/// Init SKD. Connects to Kafka and Rethink DB.
#[cfg(feature = "node_interaction")]
pub fn init_json(default_workchain: Option<i32>, config: String) -> SdkResult<()> {
    init(default_workchain, serde_json::from_str(&config)
        .map_err(|err| SdkErrorKind::InvalidArg(format!("{}", err)))?)
}

#[cfg(test)]
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;
