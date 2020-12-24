use super::*;
use crate::abi::{CallSet, DeploySet, ParamsOfEncodeMessage, Signer};
use crate::error::ClientError;
use crate::net::{
    ParamsOfQueryCollection, ParamsOfSubscribeCollection, ParamsOfWaitForCollection,
    ResultOfQueryCollection, ResultOfSubscribeCollection, ResultOfSubscription,
    ResultOfWaitForCollection,
};
use crate::processing::ParamsOfProcessMessage;
use crate::tests::{TestClient, HELLO};
use tokio::sync::Mutex;

#[tokio::test(core_threads = 2)]
async fn query() {
    let client = TestClient::new();

    let info: ResultOfQuery = client
        .request_async(
            "net.query",
            ParamsOfQuery {
                query: "query{info{version}}".to_owned(),
                variables: None,
            },
        )
        .await
        .unwrap();

    let version = info.result["data"]["info"]["version"].as_str().unwrap();
    assert_eq!(version.split(".").count(), 3);
}

#[tokio::test(core_threads = 2)]
async fn block_signatures() {
    let client = TestClient::new();

    let _: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "blocks_signatures".to_owned(),
                filter: Some(json!({})),
                result: "id".to_owned(),
                limit: Some(1),
                order: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test(core_threads = 2)]
async fn all_accounts() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({})),
                result: "id balance".to_owned(),
                limit: None,
                order: None,
            },
        )
        .await
        .unwrap();

    assert!(accounts.result.len() > 0);
}

#[tokio::test(core_threads = 2)]
async fn ranges() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "messages".to_owned(),
                filter: Some(json!({
                    "created_at": { "gt": 1562342740 }
                })),
                result: "body created_at".to_owned(),
                limit: None,
                order: None,
            },
        )
        .await
        .unwrap();

    assert!(accounts.result[0]["created_at"].as_u64().unwrap() > 1562342740);
}

#[tokio::test(core_threads = 2)]
async fn wait_for() {
    let now = ton_sdk::Contract::now();
    let request = tokio::spawn(async move {
        let client = TestClient::new();
        let transactions: ResultOfWaitForCollection = client
            .request_async(
                "net.wait_for_collection",
                ParamsOfWaitForCollection {
                    collection: "transactions".to_owned(),
                    filter: Some(json!({
                        "now": { "gt": now }
                    })),
                    result: "id now".to_owned(),
                    timeout: None,
                },
            )
            .await
            .unwrap();
        assert!(transactions.result["now"].as_u64().unwrap() > now as u64);
    });

    let client = TestClient::new();

    client
        .get_grams_from_giver_async(&TestClient::get_giver_address(), None)
        .await;

    request.await.unwrap();
}

#[tokio::test(core_threads = 2)]
async fn subscribe_for_transactions_with_addresses() {
    let client = TestClient::new();
    let subscription_client = TestClient::new();
    let keys = client.generate_sign_keys();
    let deploy_params = ParamsOfEncodeMessage {
        abi: TestClient::abi(HELLO, None),
        deploy_set: Some(DeploySet {
            initial_data: None,
            tvc: TestClient::tvc(HELLO, None),
            workchain_id: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };
    let msg = client.encode_message(deploy_params.clone()).await.unwrap();
    let transactions = std::sync::Arc::new(Mutex::new(vec![]));
    let transactions_copy1 = transactions.clone();
    let transactions_copy2 = transactions.clone();
    let address1 = msg.address.clone();
    let address2 = msg.address.clone();

    let callback1 = move |result: serde_json::Value, response_type: SubscriptionResponseType| {
        let result = match response_type {
            SubscriptionResponseType::Ok => {
                Ok(serde_json::from_value::<ResultOfSubscription>(result).unwrap())
            }
            SubscriptionResponseType::Error => {
                Err(serde_json::from_value::<ClientError>(result).unwrap())
            }
        };
        let address1 = address1.clone();
        let transactions_copy = transactions_copy1.clone();
        async move {
            match result {
                Ok(result) => {
                    assert_eq!(result.result["account_addr"], address1);
                    transactions_copy.lock().await.push(result.result);
                }
                Err(err) => {
                    println!(">>> {}", err);
                }
            }
        }
    };

    let handle1: ResultOfSubscribeCollection = subscription_client.request_async_callback(
            "net.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "account_addr": { "eq": msg.address.clone() },
                    "status": { "eq": ton_sdk::json_helper::transaction_status_to_u8(ton_block::TransactionProcessingStatus::Finalized) }
                })),
                result: "id account_addr".to_owned(),
            },
            callback1
        ).await.unwrap();

    let callback2 = move |result: serde_json::Value, response_type: SubscriptionResponseType| {
        let result = match response_type {
            SubscriptionResponseType::Ok => {
                Ok(serde_json::from_value::<ResultOfSubscription>(result).unwrap())
            }
            SubscriptionResponseType::Error => {
                Err(serde_json::from_value::<ClientError>(result).unwrap())
            }
        };
        let transactions_copy = transactions_copy2.clone();
        let address2 = address2.clone();
        async move {
            match result {
                Ok(result) => {
                    assert_eq!(result.result["account_addr"], address2);
                    transactions_copy.lock().await.push(result.result);
                }
                Err(err) => {
                    println!(">>> {}", err);
                }
            }
        }
    };

    let handle2: ResultOfSubscribeCollection = subscription_client.request_async_callback(
            "net.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "account_addr": { "eq": msg.address.clone() },
                    "status": { "eq": ton_sdk::json_helper::transaction_status_to_u8(ton_block::TransactionProcessingStatus::Finalized) }
                })),
                result: "id account_addr".to_owned(),
            },
            callback2
        ).await.unwrap();

    // send grams to create first transaction
    client.get_grams_from_giver_async(&msg.address, None).await;

    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // check that second transaction is not received when subscription suspended
    {
        let transactions = transactions.lock().await;
        assert_eq!(transactions.len(), 2);
    }

    // suspend subscription
    let _: () = subscription_client
        .request_async("net.suspend", ())
        .await
        .unwrap();

    // deploy to create second transaction
    client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: deploy_params,
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap();

    // check that second transaction is not received when subscription suspended
    {
        let transactions = transactions.lock().await;
        assert_eq!(transactions.len(), 2);
    }

    // resume subscription
    let _: () = subscription_client
        .request_async("net.resume", ())
        .await
        .unwrap();

    // run contract function to create third transaction
    client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    abi: TestClient::abi(HELLO, None),
                    deploy_set: None,
                    signer: Signer::Keys { keys },
                    processing_try_index: None,
                    address: Some(msg.address),
                    call_set: CallSet::some_with_function("touch"),
                },
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap();

    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(5000));

    // check that third transaction is now received after resume
    let transactions = transactions.lock().await;
    assert_eq!(transactions.len(), 4);
    assert_ne!(transactions[0]["id"], transactions[2]["id"]);

    let _: () = subscription_client
        .request_async("net.unsubscribe", handle1)
        .await
        .unwrap();
    let _: () = subscription_client
        .request_async("net.unsubscribe", handle2)
        .await
        .unwrap();
}

#[tokio::test(core_threads = 2)]
async fn subscribe_for_messages() {
    let messages = std::sync::Arc::new(Mutex::new(Vec::new()));
    let messages_copy = messages.clone();

    let callback = move |result: serde_json::Value, response_type: SubscriptionResponseType| {
        let result = match response_type {
            SubscriptionResponseType::Ok => {
                Ok(serde_json::from_value::<ResultOfSubscription>(result).unwrap())
            }
            SubscriptionResponseType::Error => {
                Err(serde_json::from_value::<ClientError>(result).unwrap())
            }
        }
        .unwrap();
        let messages_copy = messages_copy.clone();
        async move {
            messages_copy.lock().await.push(result.result);
        }
    };

    let client = TestClient::new();

    let handle: ResultOfSubscribeCollection = client
        .request_async_callback(
            "net.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "messages".to_owned(),
                filter: Some(json!({
                    "dst": { "eq": "1" }
                })),
                result: "id".to_owned(),
            },
            callback,
        )
        .await
        .unwrap();

    client
        .get_grams_from_giver_async(&TestClient::get_giver_address(), None)
        .await;

    assert_eq!(messages.lock().await.len(), 0);

    let _: () = client
        .request_async("net.unsubscribe", handle)
        .await
        .unwrap();
}

#[tokio::test(core_threads = 2)]
async fn find_last_shard_block() {
    let client = TestClient::new();

    let block: ResultOfFindLastShardBlock = client
        .request_async(
            "net.find_last_shard_block",
            ParamsOfFindLastShardBlock {
                address: TestClient::get_giver_address(),
            },
        )
        .await
        .unwrap();

    println!("{}", block.block_id);
}

#[tokio::test(core_threads = 2)]
async fn test_endpoints() {
    return;
    let client = TestClient::new();

    let endpoints: EndpointsSet = client
        .request_async("net.fetch_endpoints", ())
        .await
        .unwrap();

    let _: () = client
        .request_async("net.set_endpoints", endpoints)
        .await
        .unwrap();
}
