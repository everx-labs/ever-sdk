use crate::tests::*;
use crate::contracts::{
    EncodedMessage,
    deploy::{DeployFunctionCallSet, ParamsOfDeploy}
};
use super::*;

#[test]
fn block_signatures() {
    let client = TestClient::new();

    let _: ResultOfQueryCollection = client.request_async(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "blocks_signatures".to_owned(),
            filter: Some(json!({})),
            result: "id".to_owned(),
            limit: Some(1),
            order: None,
        }
    );
}

#[test]
fn all_accounts() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request_async(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({})),
            result: "id balance".to_owned(),
            limit: None,
            order: None,
        }
    );

    assert!(accounts.result.len() > 0);
}

#[test]
fn ranges() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request_async(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "created_at": { "gt": 1562342740 }
            })),
            result: "body created_at".to_owned(),
            limit: None,
            order: None,
        }
    );

    assert!(accounts.result[0]["created_at"].as_u64().unwrap() > 1562342740);
}

#[test]
fn wait_for() {
    let handle = std::thread::spawn(|| {
        let client = TestClient::new();
        let now = ton_sdk::Contract::now();
        let transactions: ResultOfWaitForCollection = client.request_async(
            "queries.wait_for_collection",
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

#[test]
fn subscribe_for_transactions_with_addresses() {
    let client = TestClient::new();
    let keys = client.generate_kepair();
    let deploy_params = ParamsOfDeploy{
        call_set: DeployFunctionCallSet {
            abi: HELLO_ABI.clone(),
            constructor_header: None,
            constructor_params: json!({}),
        },
        image_base64: base64::encode(HELLO_IMAGE.as_slice()),
        init_params: None,
        key_pair: keys,
        workchain_id: None,
        try_index: None
    };

    let msg: EncodedMessage = client.request_async(
        "contracts.deploy.message",
        deploy_params.clone()
    );

    let transactions = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let transactions_copy = transactions.clone();
    let address = msg.address.clone().unwrap();
    let callback_id = 123;

    let callback = move |request_id: u32, result: ApiResult<ResultOfSubscription>, flags: u32| {
        assert_eq!(flags, 0);
        assert_eq!(request_id, callback_id);
        let result = result.unwrap();
        assert_eq!(result.result["account_addr"], address);
        transactions_copy.lock().unwrap().push(result.result);
    };

    let callback_id = client.register_callback(Some(callback_id), callback);

    let handle: ResultOfSubscribeCollection = client.request_async(
            "queries.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "account_addr": { "eq": msg.address.clone().unwrap() },
                    "status": { "eq": ton_sdk::json_helper::transaction_status_to_u8(ton_block::TransactionProcessingStatus::Finalized) }
                })),
                result: "id account_addr".to_owned(),
                callback_id
            }
        );

    client.deploy_with_giver(deploy_params, None);
    
    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let transactions = transactions.lock().unwrap();
    assert_eq!(transactions.len(), 2);
    assert_ne!(transactions[0]["id"], transactions[1]["id"]);

    let _: () = client.request_async("queries.unsubscribe", handle);
    client.unregister_callback(callback_id);
}

#[test]
fn subscribe_for_messages() {
    let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let messages_copy = messages.clone();

    let callback = move |_request_id: u32, result: ApiResult<ResultOfSubscription>, flags: u32| {
        assert_eq!(flags, 0);
        let result = result.unwrap();
        messages_copy.lock().unwrap().push(result.result);
    };

    let client = TestClient::new();
    let callback_id = client.register_callback(None, callback);

    let handle: ResultOfSubscribeCollection = client.request_async(
        "queries.subscribe_collection",
        ParamsOfSubscribeCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "dst": { "eq": "1" }
            })),
            result: "id".to_owned(),
            callback_id
        }
    );

    client.get_grams_from_giver(&TestClient::get_giver_address(), None);

    assert_eq!(messages.lock().unwrap().len(), 0);

    let _: () = client.request_async("queries.unsubscribe", handle);
    client.unregister_callback(callback_id);
}
