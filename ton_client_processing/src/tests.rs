use crate::message_monitor::{
    MessageMonitor, MessageMonitoringParams, MessageMonitoringResult, MessageMonitoringStatus,
    MessageMonitoringTransaction, MonitorFetchWaitMode,
};
use crate::sdk_services::MockSdkServices;
use crate::MonitoredMessage;
use std::mem;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use ton_block::MsgAddrStd;
use ton_types::{AccountId, UInt256};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::NoWait)
        .await
        .unwrap();
    assert_eq!(results, vec![]);
    api.add_recent_ext_in_messages(vec![
        msg_res(1, MessageMonitoringStatus::Finalized),
        msg_res(2, MessageMonitoringStatus::Finalized),
    ]);
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::All)
        .await
        .unwrap();
    assert_eq!(
        sorted(results, |x| &x.hash),
        vec![
            msg_res(1, MessageMonitoringStatus::Finalized),
            msg_res(2, MessageMonitoringStatus::Finalized)
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch_at_least_one() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::NoWait)
        .await
        .unwrap();
    assert_eq!(results, vec![]);
    api.add_recent_ext_in_messages(vec![
        msg_res(1, MessageMonitoringStatus::Finalized),
        msg_res(2, MessageMonitoringStatus::Finalized),
    ]);
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::All)
        .await
        .unwrap();
    assert_eq!(
        sorted(results, |x| &x.hash),
        vec![
            msg_res(1, MessageMonitoringStatus::Finalized),
            msg_res(2, MessageMonitoringStatus::Finalized)
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch_wait_all() {
    let api = sdk_services();
    let mon = Arc::new(MessageMonitor::new(api.clone()));

    // Start monitoring for [1, 2] messages
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();

    // Wait for fetching all queued messages on a spawned thread
    let fetched = Arc::new(RwLock::new(Vec::<MessageMonitoringResult>::new()));
    let results_from_spawned = fetched.clone();
    let spawned_mon = mon.clone();
    tokio::spawn(async move {
        let results = spawned_mon
            .fetch_next_monitor_results("1", MonitorFetchWaitMode::All)
            .await
            .unwrap();
        *results_from_spawned.write().unwrap() = results;
    });

    // Resolved queue must be empty yet
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::NoWait)
        .await
        .unwrap();
    assert_eq!(results, vec![]);

    // Resolve [1, 2] messages
    api.add_recent_ext_in_messages(vec![
        msg_res(1, MessageMonitoringStatus::Finalized),
        msg_res(2, MessageMonitoringStatus::Finalized),
    ]);

    // Sleep for 1 second – get a chance for spawned thread to fetch the results
    sleep(Duration::from_millis(1000)).await;

    // Check that spawned thread has received all monitoring messages
    let results = mem::replace(&mut *fetched.write().unwrap(), Vec::new());
    assert_eq!(
        sorted(results, |x| &x.hash),
        vec![
            msg_res(1, MessageMonitoringStatus::Finalized),
            msg_res(2, MessageMonitoringStatus::Finalized)
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mon_info() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 0);
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 2);
    api.add_recent_ext_in_messages(vec![msg_res(1, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 1);
    assert_eq!(info.unresolved, 1);
}

fn hash(n: usize) -> String {
    UInt256::from_be_bytes(&n.to_be_bytes()).as_hex_string()
}

fn addr(a: usize) -> String {
    MsgAddrStd::with_address(
        None,
        0,
        AccountId::from(UInt256::from_be_bytes(&a.to_be_bytes())),
    )
    .to_string()
}

fn msg(h: usize, w: u32) -> MessageMonitoringParams {
    MessageMonitoringParams {
        message: MonitoredMessage::HashAddress {
            hash: hash(h),
            address: addr(h),
        },
        wait_until: w,
        user_data: None,
    }
}

fn msg_res(h: usize, s: MessageMonitoringStatus) -> MessageMonitoringResult {
    MessageMonitoringResult {
        hash: hash(h),
        status: s,
        transaction: Some(MessageMonitoringTransaction {
            hash: Some(hash(h)),
            aborted: false,
            compute: None,
        }),
        error: None,
        user_data: None,
    }
}

fn sdk_services() -> MockSdkServices {
    MockSdkServices::new()
}

fn sorted<T, K, F>(source: Vec<T>, mut f: F) -> Vec<T>
where
    F: FnMut(&T) -> &K,
    K: Ord,
{
    let mut source = source;
    source.sort_by(|a, b| f(a).cmp(f(b)));
    source
}
