use crate::error::ClientResult;
use crate::ClientContext;
use std::sync::Arc;
use ton_client_processing::{
    MessageMonitoringParams, MessageMonitoringResult, MonitorFetchWait, MonitoringQueueInfo,
};

#[derive(Deserialize, Default, ApiType)]
pub struct ParamsOfMonitorMessages {
    /// Name of the monitoring queue.
    pub queue: String,
    /// Messages to start monitoring for.
    pub messages: Vec<MessageMonitoringParams>,
}

#[api_function]
/// Starts monitoring for the processing results of the specified messages.
///
/// Message monitor performs background monitoring for a message processing results
/// for the specified set of messages.
///
/// Message monitor can serve several isolated monitoring queues.
/// Each monitor queue has a unique application defined identifier (or name) used
/// to separate several queue's.
///
/// There are two important lists inside of the monitoring queue:
///
/// - unresolved messages: contains messages requested by the application for monitoring
///   and not yet resolved;
///
/// - resolved results: contains resolved processing results for monitored messages.
///
/// Each monitoring queue tracks own unresolved and resolved lists.
/// Application can add more messages to the monitoring queue at any time.
///
/// Message monitor accumulates resolved results.
/// Application should fetch this results with `fetchNextMonitorResults` function.
///
/// When both unresolved and resolved lists becomes empty, monitor stops any background activity
/// and frees all allocated internal memory.
///
/// If monitoring queue with specified name already exists then messages will be added
/// to the unresolved list.
///
/// If monitoring queue with specified name does not exist then monitoring queue will be created
/// with specified unresolved messages.
///
pub async fn monitor_messages(
    context: Arc<ClientContext>,
    params: ParamsOfMonitorMessages,
) -> ClientResult<()> {
    let monitor = context.message_monitor.clone();
    monitor
        .monitor_messages(&params.queue, params.messages)
        .await?;
    Ok(())
}

#[derive(Deserialize, Default, ApiType)]
pub struct ParamsOfFetchNextMonitorResults {
    /// Name of the monitoring queue.
    pub queue: String,
    /// Wait mode. Default is `NO_WAIT`.
    pub wait: Option<MonitorFetchWait>,
}

#[derive(Serialize, ApiType)]
pub struct ResultOfFetchNextMonitorResults {
    /// List of the resolved results.
    results: Vec<MessageMonitoringResult>,
}

#[api_function]
/// Fetches next resolved results from the specified monitoring queue.
///
/// Results and waiting options are depends on the `wait` parameter.
/// All returned results will be removed from the queue's resolved list.
pub async fn fetch_next_monitor_results(
    context: Arc<ClientContext>,
    params: ParamsOfFetchNextMonitorResults,
) -> ClientResult<ResultOfFetchNextMonitorResults> {
    let results = context
        .message_monitor
        .fetch_next_monitor_results(
            &params.queue,
            params.wait.unwrap_or(MonitorFetchWait::NoWait),
        )
        .await?;
    Ok(ResultOfFetchNextMonitorResults { results })
}

#[derive(Deserialize, ApiType, Default)]
pub struct ParamsOfGetMonitorInfo {
    /// Name of the monitoring queue.
    pub queue: String,
}

#[api_function]
/// Returns summary information about current state of the specified monitoring queue.
pub async fn get_monitor_info(
    context: Arc<ClientContext>,
    params: ParamsOfGetMonitorInfo,
) -> ClientResult<MonitoringQueueInfo> {
    Ok(context.message_monitor.get_queue_info(&params.queue)?)
}

#[derive(Deserialize, Default, ApiType)]
pub struct ParamsOfCancelMonitor {
    /// Name of the monitoring queue.
    pub queue: String,
}

#[api_function]
/// Cancels all background activity and releases all allocated system resources
/// for the specified monitoring queue.
pub async fn cancel_monitor(
    context: Arc<ClientContext>,
    params: ParamsOfCancelMonitor,
) -> ClientResult<()> {
    context.message_monitor.cancel_monitor(&params.queue)?;
    Ok(())
}
