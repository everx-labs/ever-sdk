/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use failure::{Context, Fail, Backtrace};
use std::fmt::{Formatter, Result, Display};

#[derive(Debug)]
pub struct SdkError {
    inner: Context<SdkErrorKind>,
}

pub type SdkResult<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum SdkErrorKind {

    #[fail(display = "Block error: {}", error)]
    BlockError {
        error: ton_block::BlockError
    },

    #[fail(display = "Requested item not found")]
    NotFound,

    #[fail(display = "No data")]
    NoData,

    #[fail(display = "Invalid operation: {}", msg)]
    InvalidOperation {
        msg: String
    },

    #[fail(display = "Invalid data: {}", msg)]
    InvalidData {
        msg: String
    },

    #[fail(display = "Invalid argument: {}", msg)]
    InvalidArg {
        msg: String
    },

    #[fail(display = "Internal error: {}", msg)]
    InternalError {
        msg: String
    },

    #[fail(display = "Signature error: {}", err)]
    Signature {
        err: ed25519_dalek::SignatureError
    },

    #[fail(display = "SDK is not initialized")]
    NotInitialized,

    #[fail(display = "SDK initialize error")]
    InitializeError,

    #[fail(display = "Network error: {}", msg)]
    NetworkError {
        msg: String
    },

    #[fail(display = "Local contract call error: {}", msg)]
    LocalCallError {
        msg: String
    },

    // External errors

    #[fail(display = "IO error: {}", err)]
    Io { 
        err: std::io::Error
    },

    #[fail(display = "VM exception: {}", ex)]
    TvmException {
        ex: ton_vm::types::Exception,
    },

    #[fail(display = "VM exception, code: {}", code)]
    TvmExceptionCode {
        code: ton_types::types::ExceptionCode,
    },

    #[cfg(feature = "node_interaction")]
    #[fail(display = "Graphite error: {}", err)]
    Graphql {
        err: graphite::types::GraphiteError
    },

    #[fail(display = "Serde json error: {}", err)]
    SerdeError {
        err: serde_json::Error
    },

    #[fail(display = "Try from slice error: {}", err)]
    TryFromSliceError {
        err: std::array::TryFromSliceError
    },

    #[fail(display = "Parse int error: {}", err)]
    ParseIntError {
        err: std::num::ParseIntError
    },

    #[fail(display = "From hex error: {}", err)]
    FromHexError {
        err: hex::FromHexError
    },

    #[fail(display = "Base64 decode error: {}", err)]
    Base64DecodeError {
        err: base64::DecodeError
    },

    #[fail(display = "ABI error: {}", err)]
    AbiError {
        err: ton_abi::error::AbiError
    },

    #[fail(display = "Try from int error: {}", err)]
    TryFromIntError {
        err: std::num::TryFromIntError
    },

    #[cfg(feature = "fee_calculation")]
    #[fail(display = "Transaction executor error: {}", err)]
    ExecutorError {
        err: ton_executor::ExecutorError
    },
}

impl SdkError {
    pub fn kind(&self) -> &SdkErrorKind {
        self.inner.get_context()
    }
}

impl Fail for SdkError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for SdkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<SdkErrorKind> for SdkError {
    fn from(kind: SdkErrorKind) -> SdkError {
        SdkError { inner: Context::new(kind) }
    }
}

impl From<std::io::Error> for SdkError {
    fn from(err: std::io::Error) -> SdkError {
        SdkError::from(SdkErrorKind::Io { err })
    }
}

impl From<ton_types::types::ExceptionCode> for SdkError {
    fn from(code: ton_types::types::ExceptionCode) -> SdkError {
        SdkError::from(SdkErrorKind::TvmExceptionCode { code })
    }
}

impl From<ton_vm::types::Exception> for SdkError {
    fn from(ex: ton_vm::types::Exception) -> SdkError {
        SdkError::from(SdkErrorKind::TvmException { ex })
    }
}

#[cfg(feature = "node_interaction")]
impl From<graphite::types::GraphiteError> for SdkError {
    fn from(err: graphite::types::GraphiteError) -> SdkError {
        SdkError::from(SdkErrorKind::Graphql { err })
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(err: serde_json::Error) -> SdkError {
        SdkError::from(SdkErrorKind::SerdeError { err })
    }
}

impl From<std::array::TryFromSliceError> for SdkError {
    fn from(err: std::array::TryFromSliceError) -> SdkError {
        SdkError::from(SdkErrorKind::TryFromSliceError { err })
    }
}

impl From<std::num::ParseIntError> for SdkError {
    fn from(err: std::num::ParseIntError) -> SdkError {
        SdkError::from(SdkErrorKind::ParseIntError { err })
    }
}

impl From<hex::FromHexError> for SdkError {
    fn from(err: hex::FromHexError) -> SdkError {
        SdkError::from(SdkErrorKind::FromHexError { err })
    }
}

impl From<base64::DecodeError> for SdkError {
    fn from(err: base64::DecodeError) -> SdkError {
        SdkError::from(SdkErrorKind::Base64DecodeError { err })
    }
}

impl From<ton_abi::error::AbiError> for SdkError {
    fn from(err: ton_abi::error::AbiError) -> SdkError {
        SdkError::from(SdkErrorKind::AbiError { err })
    }
}


impl From<std::num::TryFromIntError> for SdkError {
    fn from(err: std::num::TryFromIntError) -> SdkError {
        SdkError::from(SdkErrorKind::TryFromIntError { err })
    }
}

#[cfg(feature = "fee_calculation")]
impl From<ton_executor::ExecutorError> for SdkError {
    fn from(err: ton_executor::ExecutorError) -> SdkError {
        SdkError::from(SdkErrorKind::ExecutorError { err })
    }
}
