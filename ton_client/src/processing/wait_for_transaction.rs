use crate::abi::Abi;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use crate::processing::internal::{get_message_expiration_time, get_message_id, resolve_error};
use crate::processing::{fetching, internal, Error};
use crate::processing::{ProcessingEvent, ResultOfProcessMessage};
use std::sync::Arc;
use ton_sdk::Contract;

//--------------------------------------------------------------------------- wait_for_transaction

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Optional ABI for decoding the transaction result.
    ///
    /// If it is specified, then the output messages' bodies will be
    /// decoded according to this ABI.
    ///
    /// The `abi_decoded` result field will be filled out.
    pub abi: Option<Abi>,

    /// Message BOC. Encoded with `base64`.
    pub message: String,

    /// The last generated block id of the destination account shard before the message was sent.
    ///
    /// You must provide the same value as the `send_message` has returned.
    pub shard_block_id: String,

    /// Flag that enables/disables intermediate events
    pub send_events: bool,
}

pub async fn wait_for_transaction<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync,
) -> ClientResult<ResultOfProcessMessage> {
    let net = context.get_server_link()?;

    // Prepare to wait
    let message = Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?;
    let processing_timeout = net.config().message_processing_timeout;
    let now = context.env.now_ms();
    let max_block_time = message_expiration_time.unwrap_or(now + processing_timeout as u64);
    log::debug!(
        "message_expiration_time {}",
        message_expiration_time.unwrap_or_default() / 1000
    );
    let mut shard_block_id = params.shard_block_id.clone();

    // Block walking loop
    loop {
        let fetch_block_timeout =
            (std::cmp::max(max_block_time, now) - now) as u32 + processing_timeout;

        let block = fetching::fetch_next_shard_block(
            &context,
            &params,
            &address,
            &shard_block_id,
            &message_id,
            fetch_block_timeout,
            &callback,
        )
        .await?;
        if let Some(transaction_id) =
            internal::find_transaction(&block, &message_id, &shard_block_id)?
        {
            // Transaction has been found.
            // Let's fetch other stuff.
            return Ok(fetching::fetch_transaction_result(
                &context,
                &shard_block_id,
                &message_id,
                &transaction_id,
                &params.abi,
                address,
                (max_block_time / 1000) as u32,
                block.gen_utime,
            )
            .await?);
        }
        // If we found a block with expired `gen_utime`,
        // then stop walking and return error.
        if block.gen_utime as u64 * 1000 > max_block_time {
            let waiting_expiration_time = (max_block_time / 1000) as u32;
            let error = if message_expiration_time.is_some() {
                Error::message_expired(
                    &message_id,
                    &shard_block_id,
                    waiting_expiration_time,
                    block.gen_utime,
                )
            } else {
                Error::transaction_wait_timeout(
                    &message_id,
                    &shard_block_id,
                    waiting_expiration_time,
                    processing_timeout,
                    block.gen_utime,
                )
            };

            resolve_error(
                context.clone(),
                &address,
                params.message.clone(),
                error,
                waiting_expiration_time - 1,
            ).await?;
        }

        // We have successfully walked through the block.
        // So store it as the last checked.
        shard_block_id = block.id.to_string();
    }
}
