use crate::client::ClientContext;
use crate::error::ApiError;
use crate::processing::defaults::{
    can_retry_more, DEFAULT_EXPIRATION_RETRIES_LIMIT, DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    DEFAULT_NETWORK_RETRIES_LIMIT, DEFAULT_NETWORK_RETRIES_TIMEOUT,
};
use serde_json::Value;
use std::sync::Arc;
use ton_sdk::{NetworkConfig, ReceivedTransaction};

#[derive(Serialize, Deserialize, TypeInfo, Debug, PartialEq, Clone)]
pub struct TransactionResult {
    /// Transaction BOC. Encoded with `base64`.
    pub transaction: Value,
    /// List of all output messages (BOCs). Encoded with `base64`.
    pub out_messages: Vec<Value>,
    /// Parsed body of the out message that contains the ABI function return value.
    pub abi_return_value: Option<Value>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,
    /// Determine that callback must stay registered after operation has been finished.
    /// By default the client will automatically unregister callback.
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
    /// Notifies the app that the current shard block will be fetched from the network.
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},

    /// Notifies the app that the client has failed to fetch current shard block.
    /// Message processing has finished.
    FetchFirstBlockFailed { error: ApiError },

    /// Notifies the app that the message will be sent to the network.
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

    /// Notifies the app that the sending operation was failed with network error.
    /// Processing will be continued at waiting phase because
    /// the message possibly has been delivered to the node.
    SendFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the next shard block will be fetched from the network.
    /// Event can occurs more than one time due to block walking procedure.
    WillFetchNextBlock {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the next block can't be fetched due to error.
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the message was expired.
    /// Event occurs for messages with the `expiration` replay protection.
    /// Processing will be continued at encoding phase after
    /// `expiration_retries_timeout`.
    MessageExpired {
        processing_state: ProcessingState,
        message_id: String,
        message: String,
        error: ApiError,
    },

    /// Notifies the app that the client has received the transaction.
    /// Processing has finished.
    TransactionReceived {
        /// Input message id. Encoded with `hex`.
        message_id: String,
        /// Input message. BOC encoded with `base64`.
        message: String,
        /// Results of transaction.
        result: TransactionResult,
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

impl Option<ProcessingOptions> {
    pub fn can_retry_network_error(&self, context: &Arc<ClientContext>, retries: &mut i8) -> bool {
        can_retry_more(
            retries,
            self.resolve(
                context.config.network.as_ref(),
                |x| x.network_retries_limit,
                |_| None,
                DEFAULT_NETWORK_RETRIES_LIMIT,
            ),
        )
    }

    pub fn resolve_network_retries_timeout(&self, context: &Arc<ClientContext>) -> u32 {
        self.resolve(
            context.config.network.as_ref(),
            |x| x.network_retries_timeout,
            |_| None,
            DEFAULT_NETWORK_RETRIES_TIMEOUT,
        )
    }

    pub fn can_retry_expired_message(&self, context: &Arc<ClientContext>, retries: i8) -> bool {
        can_retry_more(
            retries,
            self.resolve(
                context.config.network.as_ref(),
                |x| x.expiration_retries_limit,
                |x| Some(x.message_retries_count() as i8),
                DEFAULT_EXPIRATION_RETRIES_LIMIT,
            ),
        )
    }

    pub fn resolve_expiration_retries_timeout(&self, context: &Arc<ClientContext>) -> u32 {
        self.resolve(
            context.config.network.as_ref(),
            |x| x.expiration_retries_timeout,
            |x| Some(x.message_processing_timeout()),
            DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
        )
    }

    fn resolve<C, R>(
        &self,
        config: Option<&C>,
        resolve_opt: fn(opt: Self) -> Option<R>,
        resolve_cfg: fn(cfg: &C) -> Option<R>,
        def: R,
    ) -> R {
        let opt = self.map_or(None, |x| resolve_opt(x));
        let cfg = config.map_or(None, |x| resolve_cfg(x));
        opt.or(cfg).unwrap_or(def)
    }
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
