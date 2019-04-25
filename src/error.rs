use std::io;
use tvm::types::Exception;
use abi_lib_dynamic::ABIError;
//use rdkafka::error::RDKafkaError;

error_chain! {

    types {
        SdkError, SdkErrorKind, SdkResultExt, SdkResult;
    }

    foreign_links {
        Io(io::Error);
        Tvm(Exception);
        DB(reql::errors::Error);
        Kafka(kafka::error::Error);
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
    }
}