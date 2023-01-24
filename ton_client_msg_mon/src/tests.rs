use crate::monitor::{
    MessageMonitor, MessageMonitoringParams, MessageMonitoringResult, MessageMonitoringStatus,
    MessageMonitoringTransaction, MonitorFetchWait,
};
use crate::providers::MockEverApi;
use std::mem;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use ton_block::MsgAddrStd;
use ton_types::{AccountId, UInt256};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fetch() {
    let api = providers();
    let mon = MessageMonitor::new(api.clone());
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWait::NoWait)
        .await
        .unwrap();
    assert_eq!(results, vec![]);
    api.add_recent_ext_in_messages(vec![
        msg_res(1, MessageMonitoringStatus::Finalized),
        msg_res(2, MessageMonitoringStatus::Finalized),
    ]);
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWait::AllQueued)
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
    let api = providers();
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
            .fetch_next_monitor_results("1", MonitorFetchWait::AllQueued)
            .await
            .unwrap();
        *results_from_spawned.write().unwrap() = results;
    });

    // Resolved queue must be empty yet
    let results = mon
        .fetch_next_monitor_results("1", MonitorFetchWait::NoWait)
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
        .fetch_next_monitor_results("1", MonitorFetchWait::AllQueued)
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
    let api = providers();
    let mon = MessageMonitor::new(api.clone());
    let info = mon.get_monitor_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.queued, 0);
    mon.monitor_messages("1", vec![msg(1, 1), msg(2, 2)])
        .await
        .unwrap();
    let info = mon.get_monitor_info("1").unwrap();
    assert_eq!(info.resolved, 0);
    assert_eq!(info.queued, 2);
    api.add_recent_ext_in_messages(vec![msg_res(1, MessageMonitoringStatus::Finalized)]);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let info = mon.get_monitor_info("1").unwrap();
    assert_eq!(info.resolved, 1);
    assert_eq!(info.queued, 1);
}

fn u256(n: usize) -> UInt256 {
    UInt256::from_be_bytes(&n.to_be_bytes())
}

fn addr(a: usize) -> MsgAddrStd {
    MsgAddrStd::with_address(None, 0, AccountId::from(u256(a)))
}

fn msg(h: usize, w: u32) -> MessageMonitoringParams {
    MessageMonitoringParams {
        hash: u256(h),
        address: addr(h),
        wait_until: w,
        user_data: None,
    }
}

fn msg_res(h: usize, s: MessageMonitoringStatus) -> MessageMonitoringResult {
    MessageMonitoringResult {
        hash: u256(h),
        status: s,
        transaction: Some(MessageMonitoringTransaction { hash: u256(h) }),
        user_data: None,
    }
}

fn providers() -> MockEverApi {
    MockEverApi::new()
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
