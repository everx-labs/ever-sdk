use crate::abi::DecodedMessageBody;
use crate::client::ClientContext;
use crate::error::ApiError;
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::Arc;

// TODO: move this to the `tvm` module
pub(crate) enum TvmExitCode {
    MessageExpired = 57,
    ReplayProtection = 52,
}

impl TryFrom<i32> for TvmExitCode {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            57 => Ok(Self::MessageExpired),
            62 => Ok(Self::ReplayProtection),
            _ => Err(value),
        }
    }
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, PartialEq, Clone)]
pub struct AbiDecodedOutput {
    /// Decoded bodies of the out messages.
    ///
    /// If the message can't be decoded then `None` will be stored in
    /// the appropriate position.
    pub out_messages: Vec<Option<DecodedMessageBody>>,

    /// Decoded body of the function output message.
    pub output: Option<Value>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, PartialEq, Clone)]
pub struct TransactionOutput {
    /// Parsed transaction.
    ///
    /// In addition to the regular transaction fields there is a
    /// `boc` field encoded with `base64` which contains source
    /// transaction BOC.
    pub transaction: Value,

    /// List of parsed output messages.
    ///
    /// Similar to the `transaction` each message contains the `boc`
    /// field.
    pub out_messages: Vec<Value>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub abi_decoded: Option<AbiDecodedOutput>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,

    /// Determine that callback must stay registered after operation
    /// has been finished.
    ///
    /// By default the client will automatically unregister callback
    /// after the operation that used callback has been finished.
    pub stay_registered: Option<bool>,
}

impl CallbackParams {
    pub fn with_id(id: u32) -> Self {
        Self {
            id,
            stay_registered: None,
        }
    }
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub enum ProcessingEvent {
    /// Notifies the app that the current shard block will be fetched
    /// from the network.
    ///
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},

    /// Notifies the app that the client has failed to fetch current
    /// shard block.
    ///
    /// Message processing has finished.
    FetchFirstBlockFailed { error: ApiError },

    /// Notifies the app that the message will be sent to the
    /// network.
    WillSend {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the message was sent to the network.
    DidSend {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the sending operation was failed with
    /// network error.
    ///
    /// Nevertheless the processing will be continued at the waiting
    /// phase because the message possibly has been delivered to the
    /// node.
    SendFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the next shard block will be fetched
    /// from the network.
    ///
    /// Event can occurs more than one time due to block walking
    /// procedure.
    WillFetchNextBlock {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the next block can't be fetched due to
    /// error.
    ///
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the message was expired.
    ///
    /// Event occurs for messages with the `expiration` replay
    /// protection.
    ///
    /// Processing will be continued at encoding phase after
    /// `expiration_retries_timeout`.
    MessageExpired {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the client has received the
    /// transaction.
    ///
    /// Processing has finished.
    TransactionReceived {
        /// Input message id. Encoded with `hex`.
        message_id: String,
        /// Input message. BOC encoded with `base64`.
        message: String,
        /// Results of transaction.
        result: TransactionOutput,
    },
}

impl ProcessingEvent {
    pub fn emit(self, context: &Arc<ClientContext>, callback: &CallbackParams) {
        let _ = context.send_callback_result(callback.id, self);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcessingState {
    /// The last shard block received before the message was sent or
    /// the last shard block checked for the resulting transaction
    /// after the message was sent.
    pub last_checked_block_id: String,

    /// The time when the message was sent.
    pub message_sending_time: u64,
}
