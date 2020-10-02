use super::blocks_walking::wait_next_block;
use crate::abi::{decode_message, Abi, DecodedMessageType, ParamsOfDecodeMessage};
use crate::boc::{ParamsOfParse, ResultOfParse};
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::net::MAX_TIMEOUT;
use crate::processing::internal::{
    can_retry_network_error, get_message_id, resolve_network_retries_timeout,
};
use crate::processing::types::{AbiDecodedOutput, TvmExitCode};
use crate::processing::Error;
use crate::processing::{CallbackParams, ProcessingEvent, ProcessingState, TransactionOutput};
use serde_json::Value;
use std::convert::TryInto;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::{Block, Contract, MessageId};

//--------------------------------------------------------------------------- wait_for_transaction

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Optional ABI for decoding transaction results.
    ///
    /// If it is specified then the output messages bodies will be
    /// decoded according to this ABI.
    ///
    /// The `abi_decoded` result field will be filled out.
    pub abi: Option<Abi>,

    /// Message BOC. Encoded with `base64`.
    pub message: String,

    /// Processing state. As it received from `send_message` or
    /// 'Incomplete` result of the previous call to the
    /// `wait_for_transaction`.
    pub processing_state: ProcessingState,

    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum ResultOfWaitForTransaction {
    /// The transaction has been found.
    ///
    /// All transaction related output provided.
    Complete(TransactionOutput),

    /// The transaction hasn't been found yet.
    ///
    /// Waiting was aborted due to some unexpected reason
    /// (e.g. network error).
    ///
    /// Application can resume waiting calling `wait_for_transaction`
    /// again with provided `processing_state`.
    ///
    /// The reason of the abortion is provided in `reason` field.
    Incomplete {
        processing_state: ProcessingState,
        reason: ApiError,
    },
}

/// When the ABI header `expire` is present, the processing uses
/// `message expiration` strategy:
/// - The maximum block gen time is set to
///   `message_expiration_time + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `MessageExpired` error.
///
/// When the ABI header `expire` isn't present, the processing uses
/// `transaction waiting` strategy:
/// - The maximum block gen time is set to
///   `now() + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `Incomplete` result.
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
    let now = context.env.now_ms();
    let processing_timeout = net.config().message_processing_timeout();
    let abi_header = match params.abi.as_ref() {
        Some(abi) => crate::abi::decode_message(
            context.clone(),
            ParamsOfDecodeMessage {
                abi: abi.clone(),
                message: params.message.clone(),
            },
        )
        .map(|x| x.header)?,
        None => None,
    };
    let message_expiration_time = abi_header
        .as_ref()
        .map_or(None, |x| x.expire)
        .map(|x| x as u64 * 1000);
    let max_block_time = match message_expiration_time {
        Some(time) => time,
        None => now + processing_timeout as u64,
    };
    let fetch_block_timeout =
        (std::cmp::max(max_block_time, now) - now) as u32 + processing_timeout;

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
                            &params,
                            &processing_state,
                            &message_id,
                            &transaction_id,
                            &params.abi,
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
                    // TODO: here we must execute contract and collect execution result
                    // TODO: to get more diagnostic data for application
                    return if message_expiration_time.is_some() {
                        Err(Error::message_expired(&message_id, &processing_state))
                    } else {
                        incomplete(
                            &processing_state,
                            Error::transaction_wait_timeout(&message_id, &processing_state),
                        )
                    };
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
    timeout: u32,
) -> ApiResult<Block> {
    let mut retries: i8 = 0;
    let current_block_id = processing_state.last_checked_block_id.clone().into();
    let network_retries_timeout = resolve_network_retries_timeout(context);

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
        match wait_next_block(context, &current_block_id, &address, Some(timeout)).await {
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
                if !can_retry_network_error(context, &mut retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context.env.set_timer(network_retries_timeout as u64).await;
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

async fn fetch_transaction_result(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    processing_state: &ProcessingState,
    message_id: &str,
    transaction_id: &str,
    abi: &Option<Abi>,
) -> ApiResult<TransactionOutput> {
    let transaction_boc =
        fetch_transaction_boc(context, processing_state, message_id, &transaction_id).await?;
    let (transaction, out_messages) =
        parse_transaction_boc(context, &transaction_boc)?;
    let abi_decoded = if let Some(abi) = abi {
        Some(decode_abi_output(context, abi, &out_messages)?)
    } else {
        None
    };
    let exit_code = get_exit_code(&transaction, processing_state, message_id)?;

    match exit_code.try_into() {
        Ok(TvmExitCode::MessageExpired) | Ok(TvmExitCode::ReplayProtection) => {
            Err(Error::message_expired(&message_id, &processing_state))
        }
        _ => {
            let result = TransactionOutput {
                transaction,
                out_messages,
                abi_decoded,
            };
            if let Some(cb) = &params.callback {
                ProcessingEvent::TransactionReceived {
                    message_id: message_id.to_string(),
                    message: params.message.clone(),
                    result: result.clone(),
                }
                .emit(&context, cb);
            }
            Ok(result)
        }
    }
}

async fn fetch_transaction_boc(
    context: &Arc<ClientContext>,
    processing_state: &ProcessingState,
    message_id: &str,
    transaction_id: &&str,
) -> ApiResult<TransactionBoc> {
    let transaction_boc = serde_json::from_value::<TransactionBoc>(
        context
            .get_client()?
            .wait_for(
                TRANSACTIONS_TABLE_NAME,
                &json!({
                    "id": { "eq": transaction_id.to_string() }
                }),
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
    Ok(transaction_boc)
}

fn parse_transaction_boc(
    context: &Arc<ClientContext>,
    transaction_boc: &TransactionBoc,
) -> ApiResult<(Value, Vec<Value>)> {
    let mut parsed_out_messages = Vec::<Value>::new();
    for out_message in &transaction_boc.out_messages {
        parsed_out_messages.push(parse_boc(
            context,
            &out_message.boc,
            crate::boc::parse_message,
        )?);
    }
    Ok((
        parse_boc(context, &transaction_boc.boc, crate::boc::parse_transaction)?,
        parsed_out_messages,
    ))
}

#[derive(Deserialize)]
struct ComputePhase {
    exit_code: i32,
}

#[derive(Deserialize)]
struct Transaction {
    compute: ComputePhase,
}

fn get_exit_code(
    parsed_transaction: &Value,
    processing_state: &ProcessingState,
    message_id: &str,
) -> ApiResult<i32> {
    Ok(
        serde_json::from_value::<Transaction>(parsed_transaction.clone())
            .map_err(|err| {
                Error::fetch_transaction_result_failed(
                    format!("Transaction can't be parsed: {}", err),
                    message_id,
                    processing_state,
                )
            })?
            .compute
            .exit_code,
    )
}

fn parse_boc(
    context: &Arc<ClientContext>,
    boc: &str,
    parser: fn(Arc<ClientContext>, ParamsOfParse) -> ApiResult<ResultOfParse>,
) -> ApiResult<Value> {
    let mut parsed = parser(
        context.clone(),
        ParamsOfParse {
            boc: boc.to_string(),
        },
    )?
    .parsed;
    parsed["boc"] = Value::String(boc.to_string());
    Ok(parsed)
}

fn decode_abi_output(
    context: &Arc<ClientContext>,
    abi: &Abi,
    message_bocs: &Vec<Value>,
) -> ApiResult<AbiDecodedOutput> {
    let mut out_messages = Vec::new();
    let mut output = None;
    for message_boc in message_bocs {
        out_messages.push(
            match decode_message(
                context.clone(),
                ParamsOfDecodeMessage {
                    message: message_boc.to_string(),
                    abi: abi.clone(),
                },
            ) {
                Ok(decoded) => {
                    if decoded.message_type == DecodedMessageType::FunctionOutput {
                        output = Some(decoded.value.clone());
                    }
                    Some(decoded)
                }
                _ => None,
            },
        );
    }
    Ok(AbiDecodedOutput {
        out_messages,
        output,
    })
}
