use crate::abi::DecodedBody;
use crate::error::ClientError;
use serde_json::Value;

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct DecodedOutput {
    /// Decoded bodies of the out messages.
    ///
    /// If the message can't be decoded then `None` will be stored in
    /// the appropriate position.
    pub out_messages: Vec<Option<DecodedBody>>,

    /// Decoded body of the function output message.
    pub output: Option<Value>,
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct ResultOfProcessMessage {
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
    pub decoded: Option<DecodedOutput>,
}

#[derive(Clone, num_derive::FromPrimitive, PartialEq, Debug)]
pub enum ProcessingResponseType {
    ProcessingEvent = 100,
}

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
#[serde(tag = "type")]
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
    FetchFirstBlockFailed { error: ClientError },

    /// Notifies the app that the message will be sent to the
    /// network.
    WillSend {
        shard_block_id: String,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the message was sent to the network.
    DidSend {
        shard_block_id: String,
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
        shard_block_id: String,
        message_id: String,
        message: String,
        error: ClientError,
    },

    /// Notifies the app that the next shard block will be fetched
    /// from the network.
    ///
    /// Event can occurs more than one time due to block walking
    /// procedure.
    WillFetchNextBlock {
        shard_block_id: String,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the next block can't be fetched due to
    /// error.
    ///
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        shard_block_id: String,
        message_id: String,
        message: String,
        error: ClientError,
    },

    /// Notifies the app that the message was expired.
    ///
    /// Event occurs for messages with the `expiration` replay
    /// protection.
    ///
    /// Processing will be continued at encoding phase after
    /// `expiration_retries_timeout`.
    MessageExpired {
        message_id: String,
        message: String,
        error: ClientError,
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
        result: ResultOfProcessMessage,
    },
}
