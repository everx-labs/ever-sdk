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
use ever_block::MsgAddrStd;
use ever_block::{AccountId, UInt256};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 2);
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
async fn test_cancel_monitor() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 2);
    mon.cancel_monitor("1").unwrap();
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch_at_least_one() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();
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
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::NoWait)
        .await
        .unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch_wait_all() {
    let api = sdk_services();
    let mon = Arc::new(MessageMonitor::new(api.clone()));

    // Start monitoring for [1, 2] messages
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();

    sleep(Duration::from_millis(1100)).await;

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

    // Queue should be empty
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWaitMode::NoWait)
        .await
        .unwrap();
    assert_eq!(results, vec![]);

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
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();
    sleep(Duration::from_millis(1100)).await;
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.unresolved, 2);
    api.add_recent_ext_in_messages(vec![msg_res(1, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let info = mon.get_queue_info("1").unwrap();
    assert_eq!(info.resolved, 1);
    assert_eq!(info.unresolved, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_buffering() {
    let api = sdk_services();
    let mon = MessageMonitor::new(api.clone());

    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)]).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        api.active_subscription_count(),
        0,
        "first subscription should not be started before 1 second"
    );

    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_eq!(
        api.active_subscription_count(),
        1,
        "first subscription should be started after 1 second"
    );

    mon.monitor_messages("1", vec![msg(3, 3), msg(4, 4)]).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        api.active_subscription_count(),
        1,
        "second subscription should not be started before 1 second"
    );

    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_eq!(
        api.active_subscription_count(),
        2,
        "second subscription should be started after 1 second"
    );

    api.add_recent_ext_in_messages(vec![msg_res(1, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        api.active_subscription_count(),
        2,
        "both subscriptions should be active"
    );

    api.add_recent_ext_in_messages(vec![msg_res(2, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        api.active_subscription_count(),
        1,
        "first subscription should be closed after all has resolved"
    );

    api.add_recent_ext_in_messages(vec![msg_res(3, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        api.active_subscription_count(),
        1,
        "second subscription should be active"
    );

    api.add_recent_ext_in_messages(vec![msg_res(4, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        api.active_subscription_count(),
        0,
        "all subscriptions should be closed after all has resolved"
    );
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
