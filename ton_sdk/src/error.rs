/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use std::io;

#[cfg(feature = "node_interaction")]
use graphite::types::GraphiteError;

#[cfg(not(feature = "node_interaction"))]
#[derive(Debug)]
pub struct GraphiteError {}

#[cfg(not(feature = "node_interaction"))]
impl std::fmt::Display for GraphiteError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unreachable!()
    }
}

#[cfg(not(feature = "node_interaction"))]
impl std::error::Error for GraphiteError {
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
        Tvm(ton_vm::error::TvmError);
        TvmException(ton_vm::types::Exception);
        TvmExceptionCode(ton_vm::types::ExceptionCode);
        TonBlocks(ton_block::BlockError);
        Graphql(GraphiteError);
        SerdeJson(serde_json::Error);
        TryFromSliceError(std::array::TryFromSliceError);
        ParseIntError(std::num::ParseIntError);
        FromHexError(hex::FromHexError);
        Base64DecodeError(base64::DecodeError);
        AbiError(ton_abi::error::AbiError);
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
        NotInitialized {
            description("SDK is not initialized")
        }
        InitializeError {
            description("SDK initialize error")
        }
        DefaultWorkchainNotSet {
            description("Default workchain not set")
        }
    }
}
