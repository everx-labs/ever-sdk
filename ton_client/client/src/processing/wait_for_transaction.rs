use crate::abi::{Abi};
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::processing::internal::{get_message_expiration_time, get_message_id};
use crate::processing::{fetching, internal, Error};
use crate::processing::{CallbackParams, ProcessingState, TransactionOutput};
use std::sync::Arc;
use ton_sdk::Contract;

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

/// Performs monitoring of the network for a results of the external
/// inbound message processing.
///
/// Note that presence of the `abi` parameter is critical for ABI
/// compliant contracts. Message processing uses drastically
/// different strategy for processing message with an ABI expiration
/// replay protection.
///
/// When the ABI header `expire` is present, the processing uses
/// `message expiration` strategy:
/// - The maximum block gen time is set to
///   `message_expiration_time + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `MessageExpired` error.
///
/// When the ABI header `expire` isn't present or `abi` parameter
/// isn't specified, the processing uses `transaction waiting`
/// strategy:
/// - The maximum block gen time is set to
///   `now() + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `Incomplete` result.
#[function_info(name = "processing.wait_for_transaction")]
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
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?;
    let max_block_time = message_expiration_time.unwrap_or(now + processing_timeout as u64);
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
        match fetching::fetch_next_shard_block(
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
                match internal::find_transaction(&block, &message_id, &processing_state) {
                    Ok(Some(transaction_id)) => {
                        // Transaction has been found.
                        // Let's fetch other stuff.
                        return match fetching::fetch_transaction_result(
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
