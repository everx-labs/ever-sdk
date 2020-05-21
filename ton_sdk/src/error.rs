/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

#[derive(Debug, failure::Fail)]
pub enum SdkError {

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

    #[fail(display = "Try from int error: {}", err)]
    TryFromIntError {
        err: std::num::TryFromIntError
    },

    #[fail(display = "Wait for operation rejected on timeout")]
    WaitForTimeout,

    #[fail(display =
        "Message expired\n\tid: {}\n\tsend time: {}\n\texpiration time: {}\n\tblock time: {}",
        msg_id, send_time, expire, block_time)]
    MessageExpired {
        msg_id: crate::MessageId,
        send_time: u32,
        expire: u32,
        block_time: u32
    },

    #[fail(display = "SDK is initialized without node address")]
    SdkNotInitialized,

    #[fail(display = "No blocks produced during timeout")]
    NetworkSilent{
        msg_id: crate::MessageId,
        send_time: u32,
        expire: u32,
        timeout: u32
    },

    #[fail(display = "Existing block transaction not found")]
    TransactionsLag{
        msg_id: crate::MessageId,
        send_time: u32,
        block_id: String,
        timeout: u32
    },

    #[fail(display = "Transaction was not produced during the specified timeout")]
    TransactionWaitTimeout{
        msg_id: crate::MessageId,
        send_time: u32,
        timeout: u32
    },
}
