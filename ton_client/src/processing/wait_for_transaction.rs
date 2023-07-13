use super::remp::{RempStatus, RempStatusData};
use crate::abi::Abi;
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::{EndpointStat, ResultOfSubscription};
use crate::processing::internal::{get_message_expiration_time, resolve_error};
use crate::processing::{fetching, internal, Error};
use crate::processing::{ProcessingEvent, ResultOfProcessMessage};
use futures::{FutureExt, StreamExt};
use std::convert::TryInto;
use std::sync::Arc;
use tokio::sync::mpsc;
use ton_block::{Message, MsgAddressInt};

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

    let callback = Arc::new(callback);

    if net.state().get_query_endpoint().await?.remp_enabled() {
        wait_by_remp(context, params, callback).await
    } else {
        wait_by_block_walking(context, &params, callback).await
    }
}

async fn wait_by_remp<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
    callback: Arc<impl Fn(ProcessingEvent) -> F + Send + Sync>,
) -> ClientResult<ResultOfProcessMessage> {
    // fallback to block walking in case of any error
    let notify = tokio::sync::Notify::new();
    let fallback_fut = async {
        notify.notified().await;
        wait_by_block_walking(context.clone(), &params, callback.clone()).await
    }
    .fuse();
    futures::pin_mut!(fallback_fut);

    // Prepare to wait
    let message = deserialize_object_from_boc::<Message>(&context, &params.message, "message")?;
    let message_id = message.cell.repr_hash().as_hex_string();
    let message_dst = message
        .object
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let (sender, receiver) = mpsc::channel(10);
    let mut receiver = tokio_stream::wrappers::ReceiverStream::new(receiver).fuse();

    let subscription_callback = move |event: ClientResult<ResultOfSubscription>| {
        let sender = sender.clone();
        async move {
            let _ = sender
                .send(event.map(|mut result| result.result["rempReceipts"].take()))
                .await;
        }
    };

    let subscription_result = crate::net::subscribe(
        context.clone(),
        crate::net::subscriptions::ParamsOfSubscribe {
            subscription: format!(
                r#"
                subscription {{
                    rempReceipts(messageId: "{}") {{
                        messageId kind timestamp json
                    }}
                }}
                "#,
                message_id
            ),
            variables: None,
        },
        subscription_callback,
    )
    .await;

    let subscription = match subscription_result {
        Ok(result) => Some(result),
        Err(error) => {
            if params.send_events {
                callback(ProcessingEvent::RempError {
                    error,
                    message_id: message_id.clone(),
                    message_dst: message_dst.to_string(),
                })
                .await;
            }
            notify.notify_one();
            None
        }
    };

    // wait for REMP statuses and process them
    // if no statuses received during timeout or any error occurred then activate fallback
    let mut timeout = context.config.network.first_remp_status_timeout;
    let mut fallback_activated = false;
    loop {
        let timer = context.env.set_timer(timeout as u64).fuse();
        futures::pin_mut!(timer);
        let result = futures::select! {
            _ = timer => {
                if !fallback_activated {
                    if params.send_events {
                        callback(ProcessingEvent::RempError {
                            error: Error::next_remp_status_timeout(),
                            message_id: message_id.clone(),
                            message_dst: message_dst.to_string(),
                        }).await;
                    }
                    timeout = std::u32::MAX;
                    notify.notify_one();
                    fallback_activated = true;
                }
                None
            },
            fallback = fallback_fut => Some(fallback),
            remp_message = receiver.select_next_some() => {
                timeout = context.config.network.next_remp_status_timeout;
                match process_remp_message(
                    context.clone(),
                    callback.clone(),
                    &params,
                    &message_id,
                    &message_dst,
                    remp_message
                ).await {
                    Err(error) => {
                        if params.send_events {
                            callback(ProcessingEvent::RempError {
                                error,
                                message_id: message_id.clone(),
                                message_dst: message_dst.to_string(),
                            }).await;
                        }
                        notify.notify_one();
                        None
                    },
                    Ok(result) => result,
                }
            }
        };

        if let Some(result) = result {
            if let Some(subscription) = subscription {
                let _ = crate::net::unsubscribe(context.clone(), subscription).await;
            }

            return result;
        }
    }
}

async fn process_remp_message<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    callback: Arc<impl Fn(ProcessingEvent) -> F + Send + Sync>,
    params: &ParamsOfWaitForTransaction,
    message_id: &str,
    message_dst: &MsgAddressInt,
    remp_message: ClientResult<serde_json::Value>,
) -> ClientResult<Option<ClientResult<ResultOfProcessMessage>>> {
    let remp_message = remp_message?;
    let status: RempStatus = serde_json::from_value(remp_message).map_err(|err| {
        Error::invalid_remp_status(format!("can not parse REMP status message: {}", err))
    })?;

    match status {
        RempStatus::RejectedByFullnode(data) => Ok(Some(
            process_rejected_status(context.clone(), params, &message_dst, data).await,
        )),
        RempStatus::Finalized(data) => Ok(Some(Ok(process_finalized_status(
            context.clone(),
            params,
            message_id,
            &message_dst,
            data,
        )
        .await?))),
        _ => {
            if params.send_events {
                callback(status.into_event(message_dst.to_string())).await;
            }
            Ok(None)
        }
    }
}

async fn process_rejected_status(
    context: Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    message_dst: &MsgAddressInt,
    data: RempStatusData,
) -> ClientResult<ResultOfProcessMessage> {
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?
            .unwrap_or_else(|| context.env.now_ms());

    let error = data.json["error"].as_str().unwrap_or("unknown error");

    let resolved = resolve_error(
        context.clone(),
        message_dst,
        params.message.clone(),
        Error::message_rejected(&data.message_id, error),
        (message_expiration_time / 1000) as u32 - 1,
        true,
    )
    .await
    .add_network_url_from_context(&context)
    .await;

    resolved.map(|_| Default::default())
}

async fn process_finalized_status(
    context: Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    message_id: &str,
    message_dst: &MsgAddressInt,
    data: RempStatusData,
) -> ClientResult<ResultOfProcessMessage> {
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?
            .unwrap_or_else(|| context.env.now_ms());

    let block_id = data.json["block_id"]
        .as_str()
        .ok_or_else(|| Error::invalid_remp_status("no `block_id` field in `Finalized` message"))?;

    // Transaction has been found.
    // Let's fetch other stuff.
    let result = fetching::fetch_transaction_result(
        &context,
        block_id,
        message_id,
        &params.message,
        None,
        &params.abi,
        message_dst.clone(),
        (message_expiration_time / 1000) as u32,
        (context.env.now_ms() / 1000) as u32,
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
    };
    result
}

async fn wait_by_block_walking<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    callback: Arc<impl Fn(ProcessingEvent) -> F + Send + Sync>,
) -> ClientResult<ResultOfProcessMessage> {
    let net = context.get_server_link()?;

    // Prepare to wait
    let message = deserialize_object_from_boc::<Message>(&context, &params.message, "message")?;

    let message_id = message.cell.repr_hash().as_hex_string();
    let address = message
        .object
        .dst_ref()
        .cloned()
        .ok_or(Error::message_has_not_destination_address())?;
    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?;
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
        let timeout = std::cmp::max(max_block_time, now) - now + processing_timeout as u64;
        let fetch_block_timeout = timeout.try_into().unwrap_or(u32::MAX);
        log::debug!("fetch_block_timeout {}", fetch_block_timeout);

        let block = fetching::fetch_next_shard_block(
            &context,
            &params,
            &address,
            &shard_block_id,
            &message_id,
            fetch_block_timeout,
            callback.as_ref(),
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
                Some(&transaction_id),
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
