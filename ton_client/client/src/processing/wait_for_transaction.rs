use crate::abi::Abi;
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
#[method_info(name = "processing.wait_for_transaction")]
pub async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
) -> ApiResult<TransactionOutput> {
    let net = context.get_client()?;

    // Prepare to wait
    let message = Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?;
    let processing_timeout = net.config().message_processing_timeout();
    let mut processing_state = params.processing_state.clone();

    // Block walking loop
    loop {
        let now = context.env.now_ms();
        let max_block_time = message_expiration_time.unwrap_or(now + processing_timeout as u64);
        let fetch_block_timeout =
            (std::cmp::max(max_block_time, now) - now) as u32 + processing_timeout;

        let block = fetching::fetch_next_shard_block(
            &context,
            &params,
            &address,
            &processing_state,
            &message_id,
            fetch_block_timeout,
        )
        .await?;
        if let Some(transaction_id) =
            internal::find_transaction(&block, &message_id, &processing_state)?
        {
            // Transaction has been found.
            // Let's fetch other stuff.
            return Ok(fetching::fetch_transaction_result(
                &context,
                &params,
                &processing_state,
                &message_id,
                &transaction_id,
                &params.abi,
            )
            .await?);
        }
        // If we found a block with expired `gen_utime`,
        // then stop walking and return error.
        if block.gen_utime as u64 * 1000 > max_block_time {
            // TODO: here we must execute contract and collect execution result
            // TODO: to get more diagnostic data for application
            return if message_expiration_time.is_some() {
                Err(Error::message_expired(&message_id, &processing_state))
            } else {
                Err(Error::transaction_wait_timeout(
                    &message_id,
                    &processing_state,
                ))
            };
        }

        // We have successfully walked through the block.
        // So store it as the last checked.
        processing_state.last_checked_block_id = block.id.to_string();
    }
}
