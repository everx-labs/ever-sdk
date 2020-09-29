use crate::abi::ResultOfDecodeMessage;
use crate::client::ClientContext;
use crate::error::ApiError;
use crate::processing::defaults::{
    can_retry_more, DEFAULT_EXPIRATION_RETRIES_LIMIT, DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    DEFAULT_NETWORK_RETRIES_LIMIT, DEFAULT_NETWORK_RETRIES_TIMEOUT,
};
use serde_json::Value;
use std::sync::Arc;


#[derive(Serialize, Deserialize, TypeInfo, Debug, PartialEq, Clone)]
pub struct AbiDecodedOutput {
    /// Decoded bodies of the out messages.
    /// If some message can't
    pub out_messages: Vec<Option<ResultOfDecodeMessage>>,

    /// Decoded body of the out message that
    /// represents the function return value.
    pub return_message: Option<Value>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, PartialEq, Clone)]
pub struct TransactionOutput {
    /// Parsed transaction.
    /// In addition to the regular transaction fields
    /// there is a `boc` field encoded with `base64`
    /// which contains source transaction BOC.
    pub transaction: Value,
    /// List of parsed output messages.
    /// Similar to the `transaction` field each message
    /// contains the `boc` field.
    /// If the `abi` parameter was specified then
    /// each message contains an optional `decoded_abi_body`
    /// field, or `null` if message body hasn't been
    /// successfully decoded.
    pub out_messages: Vec<Value>,

    pub abi_decoded: Option<AbiDecodedOutput>,

}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,
    /// Determine that callback must stay registered after
    /// operation has been finished.
    /// By default the client will automatically unregister
    /// callback.
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
    /// Notifies the app that the current shard block
    /// will be fetched from the network.
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},

    /// Notifies the app that the client has failed
    /// to fetch current shard block.
    /// Message processing has finished.
    FetchFirstBlockFailed { error: ApiError },

    /// Notifies the app that the message will be sent
    /// to the network.
    WillSend {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the message was sent
    /// to the network.
    DidSend {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the sending operation was failed
    /// with network error.
    /// Nevertheless the processing will be continued
    /// at the waiting phase because the message possibly
    /// has been delivered to the node.
    SendFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the next shard block will be
    /// fetched from the network.
    /// Event can occurs more than one time due to
    /// block walking procedure.
    WillFetchNextBlock {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the next block can't be fetched
    /// due to error.
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the message was expired.
    /// Event occurs for messages with the `expiration`
    /// replay protection.
    /// Processing will be continued at encoding phase after
    /// `expiration_retries_timeout`.
    MessageExpired {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the client has received
    /// the transaction.
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

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct ProcessingOptions {
    /// Limit the retries count for failed network requests.
    /// Negative value means infinite.
    /// Default is -1.
    pub network_retries_limit: Option<i8>,
    /// Timeout between retries of failed network operations.
    /// Default is 40000.
    pub network_retries_timeout: Option<u32>,
    /// Limit the retries count for expired messages.
    /// Negative value means infinite.
    /// Default is 8.
    pub expiration_retries_limit: Option<i8>,
    /// Limit the retries count for expired messages.
    /// Default is 40000.
    pub expiration_retries_timeout: Option<u32>,
}

pub fn can_retry_network_error(
    options: &Option<ProcessingOptions>,
    context: &Arc<ClientContext>,
    retries: &mut i8,
) -> bool {
    can_retry_more(
        retries,
        resolve(
            options,
            context.config.network.as_ref(),
            |x| x.network_retries_limit,
            |_| None,
            DEFAULT_NETWORK_RETRIES_LIMIT,
        ),
    )
}

pub fn resolve_network_retries_timeout(
    options: &Option<ProcessingOptions>,
    context: &Arc<ClientContext>,
) -> u32 {
    resolve(
        options,
        context.config.network.as_ref(),
        |x| x.network_retries_timeout,
        |_| None,
        DEFAULT_NETWORK_RETRIES_TIMEOUT,
    )
}

pub fn can_retry_expired_message(
    options: &Option<ProcessingOptions>,
    context: &Arc<ClientContext>,
    retries: &mut i8,
) -> bool {
    can_retry_more(
        retries,
        resolve(
            options,
            context.config.network.as_ref(),
            |x| x.expiration_retries_limit,
            |x| Some(x.message_retries_count() as i8),
            DEFAULT_EXPIRATION_RETRIES_LIMIT,
        ),
    )
}

pub fn resolve_expiration_retries_timeout(
    options: &Option<ProcessingOptions>,
    context: &Arc<ClientContext>,
) -> u32 {
    resolve(
        options,
        context.config.network.as_ref(),
        |x| x.expiration_retries_timeout,
        |x| Some(x.message_processing_timeout()),
        DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    )
}

fn resolve<C, R>(
    options: &Option<ProcessingOptions>,
    config: Option<&C>,
    resolve_opt: fn(opt: &ProcessingOptions) -> Option<R>,
    resolve_cfg: fn(cfg: &C) -> Option<R>,
    def: R,
) -> R {
    let opt = options.as_ref().map_or(None, |x| resolve_opt(x));
    let cfg = config.map_or(None, |x| resolve_cfg(x));
    opt.or(cfg).unwrap_or(def)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcessingState {
    /// The last shard block received before the message was sent
    /// or the last shard block checked for the resulting transaction
    /// after the message was sent.
    pub last_checked_block_id: String,
    /// The time when the message was sent.
    pub message_sending_time: u64,
}
