use crate::abi::Abi;
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::EndpointStat;
use crate::processing::internal::{get_message_expiration_time, resolve_error};
use crate::processing::{fetching, internal, Error};
use crate::processing::{ProcessingEvent, ResultOfProcessMessage};
use std::sync::Arc;

//--------------------------------------------------------------------------- wait_for_transaction

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
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

    /// The list of endpoints to which the message was sent.
    ///
    /// Use this field to get more informative errors. 
    /// Provide the same value as the `send_message` has returned.
    /// If the message was not delivered (expired), SDK will log the endpoint URLs, used for its sending.
    pub sending_endpoints: Option<Vec<String>>,
}

pub async fn wait_for_transaction<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync,
) -> ClientResult<ResultOfProcessMessage> {
    let net = context.get_server_link()?;

    // Prepare to wait
    let message =
        deserialize_object_from_boc::<ton_block::Message>(&context, &params.message, "message")
            .await?;
    let message_id = message.cell.repr_hash().as_hex_string();
    let address = message
        .object
        .dst_ref().cloned()
        .ok_or(Error::message_has_not_destination_address())?;
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message).await?;
    let processing_timeout = net.config().message_processing_timeout;
    let max_block_time =
        message_expiration_time.unwrap_or(context.env.now_ms() + processing_timeout as u64);
    log::debug!(
        "message_expiration_time {}",
        message_expiration_time.unwrap_or_default() / 1000
    );
    let mut shard_block_id = params.shard_block_id.clone();

    // Block walking loop
    loop {
        let now = context.env.now_ms();
        let fetch_block_timeout =
            (std::cmp::max(max_block_time, now) - now) as u32 + processing_timeout;
        log::debug!("fetch_block_timeout {}", fetch_block_timeout);

        let block = fetching::fetch_next_shard_block(
            &context,
            &params,
            &address,
            &shard_block_id,
            &message_id,
            fetch_block_timeout,
            &callback,
        )
        .await
        .add_network_url_from_context(&context)
        .await?;
        let transaction_ids = internal::find_transactions(&block, &message_id, &shard_block_id)?;
        let mut last_error = None;
        for transaction_id in transaction_ids {
            // Transaction has been found.
            // Let's fetch other stuff.
            let result = fetching::fetch_transaction_result(
                &context,
                &shard_block_id,
                &message_id,
                &params.message,
                &transaction_id,
                &params.abi,
                address.clone(),
                (max_block_time / 1000) as u32,
                block.gen_utime,
            )
            .await
            .add_network_url_from_context(&context)
            .await;
            if result.is_ok() {
                if let Some(endpoints) = &params.sending_endpoints {
                    context
                        .get_server_link()?
                        .update_stat(endpoints, EndpointStat::MessageDelivered)
                        .await;
                }
                return result;
            }
            last_error = Some(result);
        }
        if let Some(result) = last_error {
            return result;
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
                    &address,
                )
            } else {
                Error::transaction_wait_timeout(
                    &message_id,
                    &shard_block_id,
                    waiting_expiration_time,
                    processing_timeout,
                    block.gen_utime,
                    &address,
                )
            };
            let resolved = resolve_error(
                context.clone(),
                &address,
                params.message.clone(),
                error,
                waiting_expiration_time - 1,
                true,
            )
            .await
            .add_network_url_from_context(&context)
            .await;
            if let (Some(endpoints), Err(err)) = (&params.sending_endpoints, &resolved) {
                if err.data["local_error"].is_null() {
                    context
                        .get_server_link()?
                        .update_stat(endpoints, EndpointStat::MessageUndelivered)
                        .await
                }
            }
            resolved?;
        }

        // We have successfully walked through the block.
        // So store it as the last checked.
        shard_block_id = block.id.to_string();
    }
}
