use std::io;
use ton_abi_json::ABIError;

#[cfg(feature = "node_interaction")]
use reql::errors::Error as DbError;
#[cfg(feature = "node_interaction")]
use kafka::error::Error as KafkaError;

#[cfg(not(feature = "node_interaction"))]
#[derive(Debug)]
pub struct DbError {}

#[cfg(not(feature = "node_interaction"))]
impl std::fmt::Display for DbError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unreachable!()
    }
}

#[cfg(not(feature = "node_interaction"))]
impl std::error::Error for DbError {
    fn description(&self) -> &str {
        unimplemented!()
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        unimplemented!()
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        unimplemented!()
    }
}


#[cfg(not(feature = "node_interaction"))]
#[derive(Debug)]
pub struct KafkaError {}

#[cfg(not(feature = "node_interaction"))]
impl std::fmt::Display for KafkaError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unreachable!()
    }
}

#[cfg(not(feature = "node_interaction"))]
impl std::error::Error for KafkaError {
    fn description(&self) -> &str {
        unimplemented!()
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        unimplemented!()
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        unimplemented!()
    }
}

error_chain! {

    types {
        SdkError, SdkErrorKind, SdkResultExt, SdkResult;
    }

    foreign_links {
        Io(io::Error);
        Tvm(tvm::error::TvmError);
        DB(DbError);
        Kafka(KafkaError);
        TonBlocks(tvm::block::BlockError);
    }

    errors {
        NotFound {
            description("Requested item not found")
        }
        NoData {
            description("Requested item not found")
        }
        InvalidOperation(msg: String) {
             description("Invalid operation"),
             display("Invalid operation: {}", msg)
        }
        InvalidData(msg: String) {
            description("Invalid data"),
            display("Invalid data: {}", msg)
        }
        InvalidArg(msg: String) {
            description("Invalid argument"),
            display("Invalid argument: {}", msg)
        }
        InternalError(msg: String) {
            description("Internal error"),
            display("Internal error: {}", msg)
        }
        Signature(inner: ed25519_dalek::SignatureError) {
            description("Signature error"),
            display("Signature error: {}", inner)
        }
        AbiError(inner: ABIError) {
            description("ABI error"),
            display("ABI error: {:?}", inner)
        }
        AbiError2(inner: ton_abi_core::abi_response::Exception) {
            description("ABI error"),
            display("ABI error: {:?}", inner)
        }
        NotInitialized {
            description("SDK is not initialized")
        }
        DefaultWorkchainNotSet {
            description("Default workchain not set")
        }
    }
}
