
use crate::tests::{TestClient};

use super::*;
use serde_json::Value;
use std::collections::HashSet;
use crate::net::ResultOfQueryCollection;

async fn query_ids_in_range(
    client: &TestClient,
    collection: &str,
    time_field: &str,
    start_time: u32,
    end_time: u32,
) -> HashSet<String> {
    let mut ids = HashSet::new();
    let mut start_time = start_time;
    while start_time < end_time {
        let items: ResultOfQueryCollection = client
            .request_async(
                "net.query_collection",
                ParamsOfQueryCollection {
                    collection: collection.to_string(),
                    filter: Some(json!({
                        time_field: { "eq": start_time },
                    })),
                    result: format!("id {}", time_field),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        for item in items.result {
            ids.insert(item["id"].as_str().unwrap().to_string());
        }
        start_time += 1;
    }
    ids
}

async fn query_block_ids_in_range(
    client: &TestClient,
    start_time: u32,
    end_time: u32,
) -> HashSet<String> {
    query_ids_in_range(
        client,
        "blocks",
        "gen_utime",
        start_time,
        end_time,
    ).await
}

async fn query_transaction_ids_in_range(
    client: &TestClient,
    start_time: u32,
    end_time: u32,
) -> HashSet<String> {
    query_ids_in_range(
        client,
        "transactions",
        "now",
        start_time,
        end_time,
    ).await
}

async fn iterate(
    client: &TestClient,
    iterator: u32,
    ids: &mut HashSet<String>,
    extra_ids: &mut HashSet<String>,
    iterated_items_limit: usize,
) -> Option<Value> {
    let mut has_more = true;
    let mut iterated_items = 0;
    let mut resume_state = None;
    while has_more && iterated_items < iterated_items_limit {
        let next: ResultOfIteratorNext = client
            .request_async(
                "net.iterator_next",
                json!({
                    "iterator": iterator,
                    "return_resume_state": true,
                    "limit": 10,
                }),
            )
            .await
            .unwrap();
        //println!(">>> {:?}", next.items);
        iterated_items += next.items.len();
        for item in next.items {
            let id = item["id"].as_str().unwrap();
            if !ids.remove(id) {
                extra_ids.insert(id.to_string());
            }
        }
        has_more = next.has_more;
        resume_state = if has_more { next.resume_state } else { None };
    }
    resume_state
}

async fn remove_iterator(client: &TestClient, iterator: u32) {
    let _: () = client
        .request_async(
            "net.remove_iterator",
            json!({
                "handle": iterator,
            }),
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn block_iterator() {
    if TestClient::node_se() {
        return;
    }
    let context = ClientContext::new(Default::default()).unwrap();
    let start_time = (context.env.now_ms() / 1000) as u32 - 60 * 60 * 10;
    let end_time = start_time + 60;
    let client = TestClient::new();
    let mut ids = query_block_ids_in_range(&client, start_time, end_time).await;
    let mut extra_ids = HashSet::new();

    let iterator: RegisteredIterator = client
        .request_async(
            "net.create_block_iterator",
            json!({
                "start_time": start_time,
                "end_time": end_time,
                "result": "id"
            }),
        )
        .await
        .unwrap();
    let resume_state = iterate(&client, iterator.handle, &mut ids, &mut extra_ids, 20).await;
    remove_iterator(&client, iterator.handle).await;

    println!(">>> Resume");

    let iterator: RegisteredIterator = client
        .request_async(
            "net.resume_block_iterator",
            json!({
                "resume_state": resume_state.unwrap(),
            }),
        )
        .await
        .unwrap();
    let resume_state = iterate(
        &client,
        iterator.handle,
        &mut ids,
        &mut extra_ids,
        usize::MAX,
    )
    .await;
    remove_iterator(&client, iterator.handle).await;
    assert!(resume_state.is_none());
    assert_eq!(ids, HashSet::default(), "Not iterated");
    assert_eq!(extra_ids, HashSet::default(), "Extra iterated");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transaction_iterator() {
    if TestClient::node_se() {
        return;
    }
    let client = TestClient::new();
    let context = ClientContext::new(Default::default()).unwrap();
    let start_time = (context.env.now_ms() / 1000) as u32 - 60 * 60 * 10;
    let end_time = start_time + 60;
    let mut ids = query_transaction_ids_in_range(&client, start_time, end_time).await;
    let mut extra_ids = HashSet::new();

    let iterator: RegisteredIterator = client
        .request_async(
            "net.create_transaction_iterator",
            json!({
                "start_time": start_time,
                "end_time": end_time,
                "result": "id",
                "include_transfers": true,
            }),
        )
        .await
        .unwrap();
    let resume_state = iterate(&client, iterator.handle, &mut ids, &mut extra_ids, 50).await;
    remove_iterator(&client, iterator.handle).await;

    println!(">>> Resume");

    let iterator: RegisteredIterator = client
        .request_async(
            "net.resume_transaction_iterator",
            json!({
                "resume_state": resume_state.unwrap(),
            }),
        )
        .await
        .unwrap();
    let resume_state = iterate(
        &client,
        iterator.handle,
        &mut ids,
        &mut extra_ids,
        usize::MAX,
    )
    .await;
    remove_iterator(&client, iterator.handle).await;
    assert!(resume_state.is_none());
    assert_eq!(ids, HashSet::default(), "Not iterated");
    assert_eq!(extra_ids, HashSet::default(), "Extra iterated");
}

/*

const iterator = await client.net.create_block_iterator({
    start_time: start_time,
    end_time: end_time,
    result: "id",
});

let has_more = true;
while (has_more) {
    const next = await client.net.iterator_next{ iterator: iterator.handle });
    for (const block of next.items) {
        console.log(block.id);
    }
    has_more = next.has_more;
}

await client.net.remove_iterator(iterator);

 */
