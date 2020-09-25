use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::net::defaults::DEFAULT_NETWORK_RETRIES_LIMIT;
use crate::net::internal::{emit_event, get_message_id};
use crate::net::types::{
    CallbackParams, MessageProcessingEvent, TransactionWaitingOptions, TransactionWaitingState,
};
use crate::net::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::node_client::MAX_TIMEOUT;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::{
    Block, BlockId, Contract, MessageId, MessageProcessingState, NodeClient, ReceivedTransaction,
    SdkError, SdkMessage, Transaction,
};

//--------------------------------------------------------------------------- wait_for_transaction

const MESSAGE_EXPIRED_CODE: i32 = 57;
const REPLAY_PROTECTION_CODE: i32 = 52;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Message BOC.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    pub message_expiration_time: Option<u32>,
    /// Waiting options.
    pub waiting_options: Option<TransactionWaitingOptions>,
    /// Waiting state.
    pub waiting_state: TransactionWaitingState,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum ResultOfWaitForTransaction {
    Complete(ReceivedTransaction),
    Incomplete(TransactionWaitingState),
}

pub async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
) -> ApiResult<ResultOfWaitForTransaction> {
    let client = context.get_client()?;
    let message = Contract::deserialize_message(&base64_decode(&params.message)?)?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let stop_time = match params.message_expiration_time {
        Some(time) => time as u64,
        None => context.now_ms() + client.config().message_processing_timeout(),
    };
    let add_timeout = client.config().message_processing_timeout() as u64;
    let mut transaction = Value::Null;
    let mut waiting_state = params.waiting_state;
    let mut network_retries = 0;
    let options = params.waiting_options.as_ref();
    let (
        network_retries_limit,
        network_retries_timeout,
        expiration_retries_limit,
        expiration_retries_timeout,
    ) = TransactionWaitingOptions::resolve(&params.waiting_options, &context);
    loop {
        let now = context.now_ms();
        let timeout = std::cmp::max(stop_time, now) - now + add_timeout;
        emit_event(&context, &params.callback, || {
            MessageProcessingEvent::WillFetchNextBlock {
                waiting_state: waiting_state.clone(),
            }
        });
        let result = Block::wait_next_block(
            client,
            &waiting_state.last_block_id,
            &address,
            Some((timeout / 1000) as u32),
        )
        .await;
        let block = match result {
            Err(err) => {
                emit_event(&context, &params.callback, || {
                    MessageProcessingEvent::FetchBlockFailed { error }
                });
                if let Some(&SdkError::WaitForTimeout) = err.downcast_ref::<SdkError>() {
                    if infinite_wait {
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        &hex::encode(&message_id),
                        &waiting_state,
                    ));
                } else if let Some(GraphiteError::NetworkError(_)) =
                    err.downcast_ref::<GraphiteError>()
                {
                    if infinite_wait {
                        context.delay_ms(network_retries_timeout as u64).await;
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        &message_id,
                        &waiting_state,
                        timeout,
                    ));
                } else {
                    fail!(err);
                }
            }
            Ok(block) => block,
        };

        processing_context.last_block_id = block.id;

        for block_msg in &block.in_msg_descr {
            if Some(message_id) == block_msg.msg_id.as_ref() {
                let tr_id = block_msg
                    .transaction_id
                    .clone()
                    .ok_or(SdkError::InvalidData {
                        msg: "No field `transaction_id` in block".to_owned(),
                    })?;

                transaction = client
                    .wait_for(
                        TRANSACTIONS_TABLE_NAME,
                        &json!({
                            "id": { "eq": tr_id.to_string() }
                        })
                        .to_string(),
                        TRANSACTION_FIELDS_ORDINARY,
                        Some(MAX_TIMEOUT),
                    )
                    .await?;

                break;
            }
        }
        if !transaction.is_null() {
            break;
        }

        if block.gen_utime > stop_time {
            if expire.is_some() {
                fail!(SdkError::MessageExpired {
                    msg_id: message_id.clone(),
                    sending_time: state.sending_time,
                    expire: stop_time,
                    block_time: block.gen_utime,
                    block_id: state.last_block_id
                });
            } else {
                fail!(SdkError::TransactionWaitTimeout {
                    msg_id: message_id.clone(),
                    sending_time: state.sending_time,
                    timeout,
                    state
                });
            }
        }
    }

    let parsed = serde_json::from_value::<Transaction>(transaction.clone())?;
    if parsed.compute.exit_code == Some(Self::MESSAGE_EXPIRED_CODE)
        || parsed.compute.exit_code == Some(Self::REPLAY_PROTECTION_CODE)
    {
        Err(SdkError::MessageExpired {
            msg_id: message_id.clone(),
            sending_time: processing_context.sending_time,
            expire: expire.unwrap_or(0),
            block_time: parsed.now,
            block_id: transaction["block_id"].as_str().unwrap_or("null").into(),
        }
        .into())
    } else {
        Ok(ReceivedTransaction {
            parsed,
            value: transaction,
        })
    }
}
