#![recursion_limit="128"] // needs for error_chain

extern crate graphite;
extern crate futures;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate tvm;
extern crate ton_block;
extern crate ton_abi_core;
extern crate hex;
extern crate ed25519_dalek;
extern crate kafka;
//extern crate rdkafka;
extern crate tokio;
#[macro_use]
extern crate lazy_static;
extern crate ton_abi_json;
extern crate rand;
extern crate sha2;

#[allow(deprecated)]
#[macro_use]
mod error;
use error::*;

mod contract;
pub use contract::*;

mod message;
pub use message::*;

mod transaction;
pub use transaction::*;

mod block;
pub use block::*;

mod types;
pub use types::*;

pub mod db_helper;

mod kafka_helper;

mod utils;

/// Init SKD. Connects to Kafka and Rethink DB.
pub fn init(config: NodeClientConfig) -> SdkResult<()> {
    kafka_helper::init(config.kafka_config)?;
    db_helper::init(config.graphql_config)
}
pub fn init_json(config: String) -> SdkResult<()> {
    init(serde_json::from_str(&config)
        .map_err(|err| SdkErrorKind::InvalidArg(format!("{}", err)))?)
}

#[cfg(test)]
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;
