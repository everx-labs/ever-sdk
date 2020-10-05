use crate::tests::*;
use crate::contracts::{
    EncodedMessage,
    deploy::{DeployFunctionCallSet, ParamsOfDeploy}
};
use crate::error::ApiError;
use super::*;

#[tokio::test(core_threads = 2)]
async fn block_signatures() {
    let client = TestClient::new();

    let _: ResultOfQueryCollection = client.request_async(
        "net.query_collection",
        ParamsOfQueryCollection {
            collection: "blocks_signatures".to_owned(),
            filter: Some(json!({})),
            result: "id".to_owned(),
            limit: Some(1),
            order: None,
        }
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn all_accounts() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request_async(
        "net.query_collection",
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({})),
            result: "id balance".to_owned(),
            limit: None,
            order: None,
        }
    ).await;

    assert!(accounts.result.len() > 0);
}

#[tokio::test(core_threads = 2)]
async fn ranges() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request_async(
        "net.query_collection",
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "created_at": { "gt": 1562342740 }
            })),
            result: "body created_at".to_owned(),
            limit: None,
            order: None,
        }
    ).await;

    assert!(accounts.result[0]["created_at"].as_u64().unwrap() > 1562342740);
}

#[test]
fn wait_for() {
    let handle = std::thread::spawn(move|| {
        let client = TestClient::new();
        let now = ton_sdk::Contract::now();
        let transactions: ResultOfWaitForCollection = client.request(
            "net.wait_for_collection",
            ParamsOfWaitForCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "now": { "gt": now }
                })),
                result: "id now".to_owned(),
                timeout: None
            }
        );

        assert!(transactions.result["now"].as_u64().unwrap() > now as u64);
    });

    let client = TestClient::new();

    client.get_grams_from_giver(&TestClient::get_giver_address(), None);

    handle.join().unwrap();
}

#[tokio::test(core_threads = 2)]
async fn subscribe_for_transactions_with_addresses() {
    let client = TestClient::new();
    let keys = client.generate_sign_keys();
    let deploy_params = ParamsOfDeploy{
        call_set: DeployFunctionCallSet {
            abi: TestClient::abi(HELLO, None),
            constructor_header: None,
            constructor_params: json!({}),
        },
        image_base64: TestClient::tvc(HELLO, None),
        init_params: None,
        key_pair: keys,
        workchain_id: None,
        try_index: None
    };

    let msg: EncodedMessage = client.request_async(
        "contracts.deploy.message",
        deploy_params.clone()
    ).await;

    let transactions = std::sync::Arc::new(Mutex::new(vec![]));
    let transactions_copy = transactions.clone();
    let address = msg.address.clone().unwrap();
    let callback = move |result: serde_json::Value, response_type: SubscriptionResponseType| {
        let result = match response_type {
            SubscriptionResponseType::Ok => Ok(serde_json::from_value::<ResultOfSubscription>(result).unwrap()),
            SubscriptionResponseType::Error => Err(serde_json::from_value::<ApiError>(result).unwrap())
        }.unwrap();
        assert_eq!(result.result["account_addr"], address);
        let transactions_copy = transactions_copy.clone();
        async move {
            transactions_copy.lock().await.push(result.result);
        }
    };

    let handle: ResultOfSubscribeCollection = client.request_async_callback(
            "net.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "account_addr": { "eq": msg.address.clone().unwrap() },
                    "status": { "eq": ton_sdk::json_helper::transaction_status_to_u8(ton_block::TransactionProcessingStatus::Finalized) }
                })),
                result: "id account_addr".to_owned(),
            },
            callback
        ).await;

    client.deploy_with_giver_async(deploy_params, None).await;

    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let transactions = transactions.lock().await;
    assert_eq!(transactions.len(), 2);
    assert_ne!(transactions[0]["id"], transactions[1]["id"]);

    let _: () = client.request_async("net.unsubscribe", handle).await;
}

#[tokio::test(core_threads = 2)]
async fn subscribe_for_messages() {
    let messages = std::sync::Arc::new(Mutex::new(Vec::new()));
    let messages_copy = messages.clone();

    let callback = move |result: serde_json::Value, response_type: SubscriptionResponseType| {
        let result = match response_type {
            SubscriptionResponseType::Ok => Ok(serde_json::from_value::<ResultOfSubscription>(result).unwrap()),
            SubscriptionResponseType::Error => Err(serde_json::from_value::<ApiError>(result).unwrap())
        }.unwrap();
        let messages_copy = messages_copy.clone();
        async move {
            messages_copy.lock().await.push(result.result);
        }
    };

    let client = TestClient::new();

    let handle: ResultOfSubscribeCollection = client.request_async_callback(
        "net.subscribe_collection",
        ParamsOfSubscribeCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "dst": { "eq": "1" }
            })),
            result: "id".to_owned(),
        },
        callback
    ).await;

    client.get_grams_from_giver_async(&TestClient::get_giver_address(), None).await;

    assert_eq!(messages.lock().await.len(), 0);

    let _: () = client.request_async("net.unsubscribe", handle).await;
}
