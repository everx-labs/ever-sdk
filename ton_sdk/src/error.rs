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

    #[fail(display = "Requested item not found: {}", 0)]
    NotFound(String),

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

    #[fail(display = "SDK is not initialized")]
    NotInitialized,

    #[fail(display = "SDK initialize error")]
    InitializeError,

    #[fail(display = "Network error: {}", msg)]
    NetworkError {
        msg: String
    },

    #[fail(display = "Contract execution error, exit code: {}", 0)]
    ContractError(i32),

    #[fail(display = "Contract has no funds for requested operation")]
    NoFundsError,

    #[fail(display = "Wait for operation rejected on timeout")]
    WaitForTimeout,

    #[fail(display =
        "Message expired\n\tid: {}\n\tsend time: {}\n\texpiration time: {}\n\tblock time: {}",
        msg_id, send_time, expire, block_time)]
    MessageExpired {
        msg_id: crate::MessageId,
        msg: Vec<u8>,
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
        msg: Vec<u8>,
        send_time: u32,
        timeout: u32
    },

    #[fail(display = "Invalid server response: {}", 0)]
    InvalidServerResponse(String),

    #[fail(display = "Clock out of sync: {}", delta_ms)]
    ClockOutOfSync {
        delta_ms: i64,
        threshold_ms: i64,
        expiration_timeout: u32
    },
}
