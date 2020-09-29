use crate::abi::Abi;
use crate::boc::ParamsOfParse;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::processing::internal::get_message_id;
use crate::processing::types::{
    can_retry_network_error, resolve_network_retries_timeout, CallbackParams, ProcessingEvent,
    ProcessingOptions, ProcessingState, TransactionOutput,
};
use crate::processing::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::node_client::MAX_TIMEOUT;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::{Block, Contract, MessageId};

//--------------------------------------------------------------------------- wait_for_transaction

const MESSAGE_EXPIRED_CODE: i32 = 57;
const REPLAY_PROTECTION_CODE: i32 = 52;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Optional ABI for decoding transaction results.
    /// If it is specified then the output messages bodies will be decoded
    /// according to this ABI.
    /// The `abi_return_output` result field will be filled out.
    pub abi: Option<Abi>,
    /// Message BOC. Encoded with `base64`.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    /// Must be the same value as it specified in `expire` ABI header
    /// of the message body.
    pub message_expiration_time: Option<u32>,
    /// Processing options.
    pub processing_options: Option<ProcessingOptions>,
    /// Processing state. As it received from `send_message`
    /// or 'Incomplete` result of the previous call to the `wait_for_transaction`.
    pub processing_state: ProcessingState,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum ResultOfWaitForTransaction {
    Complete(TransactionOutput),
    Incomplete {
        processing_state: ProcessingState,
        reason: ApiError,
    },
}

#[method_info(name = "processing.wait_for_transaction")]
pub async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
) -> ApiResult<ResultOfWaitForTransaction> {
    let net = context.get_client()?;

    // Prepare to wait
    let message = Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let mut processing_state = params.processing_state.clone();
    let now = context.now_millis();
    let processing_timeout = net.config().message_processing_timeout() as u64;
    let max_block_time = match params.message_expiration_time {
        Some(time) => time as u64,
        None => now + processing_timeout,
    };
    let fetch_block_timeout = std::cmp::max(max_block_time, now) - now + processing_timeout;

    let incomplete = |processing_state: &ProcessingState, reason: ApiError| {
        Ok(ResultOfWaitForTransaction::Incomplete {
            processing_state: processing_state.clone(),
            reason,
        })
    };

    // Block walking loop
    loop {
        match fetch_next_shard_block(
            &context,
            &params,
            &address,
            &processing_state,
            &message_id,
            fetch_block_timeout,
        )
        .await
        {
            Ok(block) => {
                match find_transaction(&block, &message_id, &processing_state) {
                    Ok(Some(transaction_id)) => {
                        // Transaction has been found.
                        // Let's fetch other stuff.
                        return match fetch_transaction_result(
                            &context,
                            &processing_state,
                            &message_id,
                            &transaction_id,
                        )
                        .await
                        {
                            Ok(result) => {
                                // We have all stuff collected, so returns with it.
                                Ok(ResultOfWaitForTransaction::Complete(result))
                            }
                            Err(err) => {
                                // There was a problem while fetching some
                                // transaction related stuff from the network.
                                // Returns an incomplete state.
                                incomplete(&processing_state, err)
                            }
                        };
                    }
                    Err(err) => {
                        // There is some block corruption occurs.
                        // Returns an incomplete state.
                        return incomplete(&processing_state, err);
                    }
                    _ => (),
                }
                // If we found a block with expired `gen_utime`,
                // then stop walking and return error.
                if block.gen_utime as u64 * 1000 > max_block_time {
                    return Err(if params.message_expiration_time.is_some() {
                        Error::message_expired(&message_id, &processing_state)
                    } else {
                        Error::transaction_wait_timeout(&message_id, &processing_state)
                    });
                }

                // We have successfully walked through the block.
                // So store it as the last checked.
                processing_state.last_checked_block_id = block.id.to_string();
            }
            Err(error) => {
                // There was network problems while fetching next block.
                // Returns an incomplete state.
                return incomplete(&processing_state, error);
            }
        }
    }
}

async fn fetch_next_shard_block(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    address: &MsgAddressInt,
    processing_state: &ProcessingState,
    message_id: &str,
    timeout: u64,
) -> ApiResult<Block> {
    let mut retries: i8 = 0;
    let current_block_id = processing_state.last_checked_block_id.clone().into();
    let network_retries_timeout =
        resolve_network_retries_timeout(&params.processing_options, context);

    // Network retries loop
    loop {
        // Notify app about fetching next block
        if let Some(cb) = &params.callback {
            ProcessingEvent::WillFetchNextBlock {
                processing_state: processing_state.clone(),
                message_id: message_id.to_string(),
                message: params.message.clone(),
            }
            .emit(&context, cb);
        }

        // Fetch next block
        match Block::wait_next_block(
            context.get_client()?,
            &current_block_id,
            &address,
            Some((timeout / 1000) as u32),
        )
        .await
        {
            Ok(block) => return Ok(block),
            Err(err) => {
                let error = Error::fetch_block_failed(err, &message_id, &processing_state);

                // Notify app about error
                if let Some(cb) = &params.callback {
                    ProcessingEvent::FetchNextBlockFailed {
                        processing_state: processing_state.clone(),
                        message_id: message_id.to_string(),
                        message: params.message.clone(),
                        error: error.clone(),
                    }
                    .emit(&context, cb)
                }

                // If network retries limit has reached, return error
                if !can_retry_network_error(&params.processing_options, context, &mut retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context.delay_millis(network_retries_timeout as u64).await;
    }
}

fn find_transaction(
    block: &Block,
    message_id: &str,
    processing_state: &ProcessingState,
) -> ApiResult<Option<String>> {
    let msg_id: MessageId = message_id.into();
    for msg_descr in &block.in_msg_descr {
        if Some(&msg_id) == msg_descr.msg_id.as_ref() {
            return Ok(Some(
                msg_descr
                    .transaction_id
                    .as_ref()
                    .ok_or(Error::invalid_block_received(
                        "No field `transaction_id` in block's `in_msg_descr`.",
                        message_id,
                        &processing_state,
                    ))?
                    .to_string(),
            ));
        }
    }
    Ok(None)
}

#[derive(Deserialize)]
struct MessageBoc {
    boc: String,
}

#[derive(Deserialize)]
struct TransactionBoc {
    boc: String,
    out_messages: Vec<MessageBoc>,
}

#[derive(Deserialize)]
struct ComputePhase {
    exit_code: i32,
}

#[derive(Deserialize)]
struct Transaction {
    compute: ComputePhase,
}

async fn fetch_transaction_result(
    context: &Arc<ClientContext>,
    processing_state: &ProcessingState,
    message_id: &str,
    transaction_id: &str,
    abi: &Abi,
) -> ApiResult<TransactionOutput> {
    let transaction = serde_json::from_value::<TransactionBoc>(
        context
            .get_client()?
            .wait_for(
                TRANSACTIONS_TABLE_NAME,
                &json!({
                    "id": { "eq": transaction_id.to_string() }
                })
                .to_string(),
                "boc out_messages { boc }",
                Some(MAX_TIMEOUT),
            )
            .await
            .map_err(|err| {
                Error::fetch_transaction_result_failed(
                    format!("Transaction can't be fetched: {}", err),
                    message_id,
                    processing_state,
                )
            })?,
    )
    .map_err(|err| {
        Error::fetch_transaction_result_failed(
            format!("Transaction can't be parsed: {}", err),
            message_id,
            processing_state,
        )
    })?;
    let parsed = crate::boc::parse_transaction(
        context.clone(),
        ParamsOfParse {
            boc: transaction.boc,
        },
    )?
    .parsed;

    let transaction = serde_json::from_value::<Transaction>(parsed.clone()).map_err(|err| {
        Error::fetch_transaction_result_failed(
            format!("Transaction can't be parsed: {}", err),
            message_id,
            processing_state,
        )
    })?;
    match transaction.compute.exit_code {
        Value::Some(MESSAGE_EXPIRED_CODE) | Some(REPLAY_PROTECTION_CODE) => {
            Err(Error::message_expired(&message_id, &processing_state))
        }
        _ => {

            Ok(TransactionOutput {
                transaction,
                out_messages: vec![],
                abi_return_output: None,
            })
        },
    }
}

