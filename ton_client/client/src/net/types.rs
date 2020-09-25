use crate::client::ClientContext;
use crate::error::ApiError;
use crate::net::defaults::{
    DEFAULT_EXPIRATION_RETRIES_LIMIT, DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    DEFAULT_NETWORK_RETRIES_LIMIT, DEFAULT_NETWORK_RETRIES_TIMEOUT,
};
use std::sync::Arc;
use ton_sdk::{NetworkConfig, ReceivedTransaction};

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,
    /// Determine that callback must stay registered after operation has been finished.
    /// By default the client will automatically unregister callback.
    pub stay_registered: Option<bool>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub enum MessageProcessingEvent {
    /// Notifies the app that the current shard block will be fetched from the network.
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},

    /// Notifies the app that the client has failed to fetch current shard block.
    /// Message processing has finished.
    FetchFirstBlockFailed { error: ApiError },

    /// Notifies the app that the message will be sent to the network.
    WillSend {
        message_id: String,
        waiting_state: TransactionWaitingState,
    },

    /// Notifies the app that the sending operation was failed with network error.
    /// Processing will be continued at waiting phase because
    /// the message possibly has been delivered to the node.
    SendFailed {
        waiting_state: TransactionWaitingState,
        error: ApiError,
    },

    /// Notifies the app that the next shard block will be fetched from the network.
    /// Event can occurs more than one time due to block walking procedure.
    WillFetchNextBlock {
        waiting_state: TransactionWaitingState,
    },

    /// Notifies the app that the next block can't be fetched due to error.
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        state: TransactionWaitingState,
        error: ApiError,
    },

    /// Notifies the app that the message was expired.
    /// Event occurs for messages with the `expiration` replay protection.
    /// Processing will be continued at encoding phase after
    /// `expiration_retries_timeout`.
    MessageExpired {
        state: TransactionWaitingState,
        error: ApiError,
    },

    /// Notifies the app that the client has received the transaction.
    /// Processing has finished.
    TransactionReceived { transaction: ReceivedTransaction },
}

pub struct TransactionWaitingOptions {
    /// Limit the retries count for failed network requests.
    /// Negative value means infinite.
    /// Default is -1.
    pub network_retries_limit: Option<isize>,
    /// Timeout between retries of failed network operations.
    /// Default is 40000.
    pub network_retries_timeout: Option<isize>,
    /// Limit the retries count for expired messages.
    /// Negative value means infinite.
    /// Default is 8.
    pub expiration_retries_limit: Option<isize>,
    /// Limit the retries count for expired messages.
    /// Default is 40000.
    pub expiration_retries_timeout: Option<isize>,
}

impl TransactionWaitingOptions {
    pub fn resolve(
        options: &Option<TransactionWaitingOptions>,
        context: &Arc<ClientContext>,
    ) -> (isize, isize, isize, isize) {
        let resolve = |opt: fn(&TransactionWaitingOptions) -> Option<isize>,
                       ctx: fn(&NetworkConfig) -> Option<isize>,
                       def: isize| {
            options
                .as_ref()
                .map_or(None, |x| opt(x))
                .or(context.config.network.as_ref().map_or(None, |x| ctx(x)))
                .unwrap_or(def)
        };
        (
            resolve(
                |x| x.network_retries_limit,
                |_| None,
                DEFAULT_NETWORK_RETRIES_LIMIT,
            ),
            resolve(
                |x| x.network_retries_timeout,
                |_| None,
                DEFAULT_NETWORK_RETRIES_TIMEOUT,
            ),
            resolve(
                |x| x.expiration_retries_limit,
                |x| Some(x.message_retries_count() as isize),
                DEFAULT_EXPIRATION_RETRIES_LIMIT,
            ),
            resolve(
                |x| x.expiration_retries_timeout,
                |x| Some(x.message_processing_timeout() as isize),
                DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
            ),
        )
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TransactionWaitingState {
    /// The last shard block received before the message was sent
    /// or the last shard block checked for the resulting transaction
    /// after the message was sent.
    pub last_checked_block_id: String,
    /// The time when the message was sent.
    pub message_sending_time: i64,
}
