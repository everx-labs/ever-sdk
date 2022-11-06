use tokio::sync::Mutex;

use crate::abi::{CallSet, DeploySet, ParamsOfEncodeMessage, Signer};
use crate::error::ClientError;
use crate::processing::ParamsOfProcessMessage;
use crate::tests::{TestClient, HELLO};

use super::*;
use crate::client::NetworkMock;
use crate::net::subscriptions::ParamsOfSubscribe;
use crate::net::ton_gql::GraphQLQuery;
use crate::ClientConfig;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use std::vec;


#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn not_authorized_response_code() {
    // Query failed: Can not send http request: Server responded with code 401
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": ["mainnet.evercloud.dev"]
        }
    }));

    let result = client
        .request_async::<_, ResultOfQueryCollection>(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({})),
                result: "id balance".to_owned(),
                limit: None,
                order: None,
            },
        )
        .await;

    if let Err(err) = result {
        assert_eq!(err.code, super::ErrorCode::Unauthorized as u32);
    } else {
        panic!("Error expected");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn not_authorized_response_text() {
    // Query failed: Can not send http request: Server responded with code 401
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": ["mainnet.evercloud.dev"]
        }
    }));
    let context = client.context().clone();
    let link = context.net.server_link.as_ref().unwrap();
    let result = link
        .query_http(
            &GraphQLQuery {
                query: "query { info { version } }".to_string(),
                timeout: None,
                variables: None,
                is_batch: false,
            },
            None,
        )
        .await;
    if let Err(err) = result {
        println!("{}", err.message);
        assert_ne!(err.message, "Unauthorized");
        assert_ne!(err.message, "Query failed: Can not send http request: Server responded with code 401");
    } else {
        panic!("Error expected");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn not_authorized() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": [TestClient::with_project("mainnet.evercloud.dev")]
        }
    }));
    let context = client.context().clone();
    let link = context.net.server_link.as_ref().unwrap();
    let result = link
        .query_http(
            &GraphQLQuery {
                query: "query { info { version } }".to_string(),
                timeout: None,
                variables: None,
                is_batch: false,
            },
            None,
        )
        .await
        .unwrap();
    println!("{:?}", result)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_header() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "access_key": "secret"
        }
    }));
    assert_eq!(
        Some((
            "Authorization".to_string(),
            "Basic OnNlY3JldA==".to_string()
        )),
        client.context().config.network.get_auth_header()
    );
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let client = TestClient::new_with_config(json!({
        "network": {
            "access_key": jwt
        }
    }));
    assert_eq!(
        Some(("Authorization".to_string(), format!("Bearer {}", jwt))),
        client.context().config.network.get_auth_header()
    );
    let client = TestClient::new();
    assert_eq!(None, client.context().config.network.get_auth_header());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn endpoints_with_graphql_suffix() {
    let url = TestClient::endpoints()[0].clone();
    let url = if let Some(url) = url.strip_suffix("/graphql") {
        url.to_string()
    } else {
        url
    };

    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": vec![format!("{}/graphql", url)]
        }
    }));
    let endpoint = client
        .context()
        .get_server_link()
        .unwrap()
        .get_query_endpoint()
        .await
        .unwrap();

    assert_eq!(endpoint.query_url, format!("{}/graphql", url));

    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": vec![url.clone()]
        }
    }));
    let endpoint = client
        .context()
        .get_server_link()
        .unwrap()
        .get_query_endpoint()
        .await
        .unwrap();
    assert_eq!(endpoint.query_url, format!("{}/graphql", url));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn batch_query() {
    let client = TestClient::new();

    let batch: ResultOfBatchQuery = client
        .request_async(
            "net.batch_query",
            ParamsOfBatchQuery {
                operations: vec![
                    ParamsOfQueryOperation::QueryCollection(ParamsOfQueryCollection {
                        collection: "blocks_signatures".to_owned(),
                        filter: None,
                        result: "id".to_owned(),
                        limit: Some(1),
                        order: None,
                    }),
                    ParamsOfQueryOperation::AggregateCollection(ParamsOfAggregateCollection {
                        collection: "accounts".to_owned(),
                        filter: None,
                        fields: Some(vec![FieldAggregation {
                            field: "".into(),
                            aggregation_fn: AggregationFn::COUNT,
                        }]),
                    }),
                    ParamsOfQueryOperation::WaitForCollection(ParamsOfWaitForCollection {
                        collection: "transactions".to_owned(),
                        filter: Some(json!({
                            "now": { "gt": 20 }
                        })),
                        result: "id now".to_owned(),
                        timeout: None,
                    }),
                ],
            },
        )
        .await
        .unwrap();

    assert_eq!(batch.results.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregates() {
    let client = TestClient::new();

    let result: ResultOfAggregateCollection = client
        .request_async(
            "net.aggregate_collection",
            ParamsOfAggregateCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({})),
                fields: Some(vec![FieldAggregation {
                    field: "".into(),
                    aggregation_fn: AggregationFn::COUNT,
                }]),
            },
        )
        .await
        .unwrap();

    let count = u32::from_str_radix(result.values[0].as_str().unwrap(), 10).unwrap();
    assert!(count > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let client = TestClient::new();

    client
        .get_tokens_from_giver_async(&client.giver_address().await, None)
        .await;

    request.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn message_sending_addresses() {
    let client = ClientContext::new(ClientConfig {
        network: NetworkConfig {
            endpoints: Some(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into(),
                "g".into(),
                "h".into(),
            ]),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let link = client.get_server_link().unwrap();
    link.update_stat(
        &vec!["a".to_string(), "e".to_string()],
        EndpointStat::MessageUndelivered,
    )
    .await;
    let bad: HashSet<_> = vec!["a".to_string(), "e".to_string()]
        .iter()
        .cloned()
        .collect();
    for _ in 0..100 {
        let addresses = link.get_addresses_for_sending().await;
        let tail: HashSet<_> = addresses[addresses.len() - 2..].iter().cloned().collect();
        assert_eq!(tail, bad);
    }
    link.update_stat(
        &vec!["a".to_string(), "e".to_string()],
        EndpointStat::MessageDelivered,
    )
    .await;
    let mut a_good = false;
    let mut e_good = false;
    for _ in 0..100 {
        let addresses = link.get_addresses_for_sending().await;
        let tail: HashSet<_> = addresses[addresses.len() - 2..].iter().cloned().collect();
        if !tail.contains("a") {
            a_good = true;
        }
        if !tail.contains("e") {
            e_good = true;
        }
    }
    assert!(a_good && e_good)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_for_transactions_with_addresses() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": TestClient::endpoints(),
        }
    }));
    let subscription_client = TestClient::new();
    let keys = subscription_client.generate_sign_keys();
    let deploy_params = ParamsOfEncodeMessage {
        abi: TestClient::abi(HELLO, None),
        deploy_set: Some(DeploySet {
            tvc: TestClient::tvc(HELLO, None),
            ..Default::default()
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };
    let msg = subscription_client
        .encode_message(deploy_params.clone())
        .await
        .unwrap();
    let transactions = std::sync::Arc::new(Mutex::new(vec![]));
    let transactions_copy1 = transactions.clone();
    let transactions_copy2 = transactions.clone();
    let notifications = std::sync::Arc::new(Mutex::new(vec![]));
    let notifications_copy1 = notifications.clone();
    let notifications_copy2 = notifications.clone();
    let address1 = msg.address.clone();
    let address2 = msg.address.clone();
    println!("Account address {}", address1);

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
        let notifications_copy = notifications_copy1.clone();
        async move {
            match result {
                Ok(result) => {
                    println!(
                        "Transaction 1 {}, {}",
                        result.result["id"], result.result["status"]
                    );
                    assert_eq!(result.result["account_addr"], address1);
                    transactions_copy.lock().await.push(result.result);
                }
                Err(err) => {
                    println!(">>> {}", err);
                    notifications_copy.lock().await.push(err);
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
                result: "id account_addr status".to_owned(),
            },
            callback1
        ).await.unwrap();

    // send grams to create first transaction
    client.get_tokens_from_giver_async(&msg.address, None).await;

    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(2000));

    {
        // check that transaction is received
        let transactions = transactions.lock().await;
        assert_eq!(transactions.len(), 1);
        // and no error notifications
        let notifications = notifications.lock().await;
        assert_eq!(notifications.len(), 0);
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

    // create second subscription while network is suspended
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
        let notifications_copy = notifications_copy2.clone();
        let address2 = address2.clone();
        async move {
            match result {
                Ok(result) => {
                    println!(
                        "Transaction 2 {}, {}",
                        result.result["id"], result.result["status"]
                    );
                    assert_eq!(result.result["account_addr"], address2);
                    transactions_copy.lock().await.push(result.result);
                }
                Err(err) => {
                    println!(">>> {}", err);
                    notifications_copy.lock().await.push(err);
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
                result: "id account_addr status".to_owned(),
            },
            callback2
        ).await.unwrap();

    // give some time for subscription to receive all data
    std::thread::sleep(std::time::Duration::from_millis(500));
    {
        // check that second transaction is not received when subscription suspended
        let transactions = transactions.lock().await;
        assert_eq!(transactions.len(), 1);
        // and both subscriptions received notification about suspend
        let notifications = notifications.lock().await;
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0], Error::network_module_suspended());
        assert_eq!(notifications[1], Error::network_module_suspended());
    }

    // resume subscription
    let _: () = subscription_client
        .request_async("net.resume", ())
        .await
        .unwrap();

    // run contract function to create third transaction
    subscription_client
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
    let transactions = transactions.lock().await.clone();
    println!(
        "{:?}",
        transactions
            .iter()
            .map(|x| x["id"].to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(transactions.len(), 3);
    assert_ne!(transactions[0]["id"], transactions[2]["id"]);
    // and both subscriptions received notification about resume
    let notifications = notifications.lock().await;
    assert_eq!(notifications.len(), 4);
    assert_eq!(notifications[2].code, Error::network_module_resumed().code);
    assert!(!notifications[2].data["query_url"].is_null());
    assert_eq!(notifications[3].code, Error::network_module_resumed().code);
    assert!(!notifications[3].data["query_url"].is_null());

    let _: () = subscription_client
        .request_async("net.unsubscribe", handle1)
        .await
        .unwrap();
    let _: () = subscription_client
        .request_async("net.unsubscribe", handle2)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
        .get_tokens_from_giver_async(&client.giver_address().await, None)
        .await;

    assert_eq!(messages.lock().await.len(), 0);

    let _: () = client
        .request_async("net.unsubscribe", handle)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn find_last_shard_block() {
    let client = TestClient::new();

    let block: ResultOfFindLastShardBlock = client
        .request_async(
            "net.find_last_shard_block",
            ParamsOfFindLastShardBlock {
                address: client.giver_address().await,
            },
        )
        .await
        .unwrap();

    println!("{}", block.block_id);
}

// #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
// async fn test_endpoints() {
//     let client = TestClient::new_with_config(json!({
//         "network": {
//             "endpoints": ["cinet.tonlabs.io", "cinet2.tonlabs.io/"],
//         }
//     }));

//     let endpoints: EndpointsSet = client
//         .request_async("net.fetch_endpoints", ())
//         .await
//         .unwrap();

//     let _: () = client
//         .request_async("net.set_endpoints", endpoints)
//         .await
//         .unwrap();
// }

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wait_resume() {
    let client = std::sync::Arc::new(TestClient::new());
    let client_copy = client.clone();

    let _: () = client.request_async("net.suspend", ()).await.unwrap();

    let start = std::time::Instant::now();

    let duration = tokio::spawn(async move {
        client_copy
            .fetch_account(&client_copy.giver_address().await)
            .await;

        start.elapsed().as_millis()
    });

    let timeout = 5000;
    tokio::time::sleep(tokio::time::Duration::from_millis(timeout)).await;

    let _: () = client.request_async("net.resume", ()).await.unwrap();

    assert!(duration.await.unwrap() > timeout as u128);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_counterparties() {
    if TestClient::node_se() {
        return;
    }

    let client = TestClient::new();

    let account = client.giver_address().await;

    let counterparties1: ResultOfQueryCollection = client
        .request_async(
            "net.query_counterparties",
            ParamsOfQueryCounterparties {
                account: account.clone(),
                first: Some(5),
                after: None,
                result: "counterparty last_message_id cursor".to_owned(),
            },
        )
        .await
        .unwrap();

    assert!(counterparties1.result.len() <= 5);

    if counterparties1.result.len() == 5 {
        let counterparties2: ResultOfQueryCollection = client
            .request_async(
                "net.query_counterparties",
                ParamsOfQueryCounterparties {
                    account: account.clone(),
                    first: Some(5),
                    after: Some(
                        counterparties1.result[4]["cursor"]
                            .as_str()
                            .unwrap()
                            .to_owned(),
                    ),
                    result: "counterparty last_message_id cursor".to_owned(),
                },
            )
            .await
            .unwrap();

        assert_ne!(counterparties1.result, counterparties2.result);
    }
}

async fn query_block_id(client: &Arc<ClientContext>) -> String {
    crate::net::query_collection(
        client.clone(),
        ParamsOfQueryCollection {
            collection: "blocks".to_string(),
            result: "id".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .result[0]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn get_query_url(client: &Arc<ClientContext>) -> String {
    let mut url = client
        .get_server_link()
        .unwrap()
        .get_query_endpoint()
        .await
        .unwrap()
        .query_url
        .clone();
    if let Some(stripped) = url.strip_prefix("http://") {
        url = stripped.to_string();
    }
    if let Some(stripped) = url.strip_prefix("https://") {
        url = stripped.to_string();
    }
    if let Some(stripped) = url.strip_suffix("/graphql") {
        url = stripped.to_string();
    }
    url
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retry_query_on_network_errors() {
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into()]),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );

    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .info(now, 1000)
        .network_err()
        .blocks("1")
        .repeat(5)
        .network_err()
        .blocks("2")
        .status(502, "")
        .blocks("3")
        .ok(&json!({
          "errors": [
            {
              "message": "Service Unavailable",
              "locations": [
                {
                  "line": 2,
                  "column": 3
                }
              ],
              "path": [
                "counterparties"
              ],
              "extensions": {
                "code": "INTERNAL_SERVER_ERROR",
                "exception": {
                  "source": "graphql",
                  "code": 503
                }
              }
            }
          ],
          "data": {
            "blocks": null
          }
        })
        .to_string())
        .blocks("4")
        .reset_client(&client)
        .await;
    assert_eq!(query_block_id(&client).await, "1");
    assert_eq!(query_block_id(&client).await, "2");
    assert_eq!(query_block_id(&client).await, "3");
    assert_eq!(query_block_id(&client).await, "4");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn querying_endpoint_selection() {
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into(), "b".into()]),
                network_retries_count: 3,
                max_latency: 1000,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );

    // Check for the fastest
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(200)
        .info(now, 500) // looser
        .url("b")
        .delay(100)
        .info(now, 500) // winner
        .reset_client(&client)
        .await;
    assert_eq!(get_query_url(&client).await, "b");

    println!("\nSkip endpoint with bad latency\n");
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(100)
        .info(now, 1500) // looser
        .url("b")
        .delay(200)
        .info(now, 500) // winner
        .reset_client(&client)
        .await;
    assert_eq!(get_query_url(&client).await, "b");

    println!("\nSelect when all have bad latency\n");
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(200)
        .info(now, 1500) // winner (slower but better latency)
        .url("b")
        .delay(100)
        .info(now, 2000) // looser (faster but worse latency)
        .reset_client(&client)
        .await;
    assert_eq!(get_query_url(&client).await, "a");

    println!("\nFailed Network\n");
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into()]),
                network_retries_count: 2,
                max_latency: 1000,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );
    NetworkMock::build()
        .url("a")
        .repeat(3)
        .network_err()
        .reset_client(&client)
        .await;
    let result = crate::net::query_collection(
        client.clone(),
        ParamsOfQueryCollection {
            collection: "blocks".to_string(),
            result: "id id".to_string(),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        match &result {
            Ok(_) => "ok",
            Err(e) => &e.message,
        },
        "Query failed: Can not send http request: Network error"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn latency_detection_with_queries() {
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into(), "b".into()]),
                network_retries_count: 3,
                max_latency: 600,
                latency_detection_interval: 100,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );

    // Check for the fastest
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(10)
        .info(now, 500) // winner
        .url("b")
        .delay(20)
        .info(now, 0) // looser
        .url("a")
        .delay(200)
        .blocks("1") // query
        .ok(&json!({
            "data": {
                "q1": [{
                    "id": "2",
                }],
                "q2": {
                    "version": "0.39.0",
                    "time": 1000,
                    "latency": 1000,
                    "rempEnabled": false,
                },
            }
        })
        .to_string()) // query with latency checking, returns bad latency
        .url("a")
        .delay(20)
        .info(now, 0) // looser
        .url("b")
        .delay(10)
        .info(now, 500) // winner
        .url("b")
        .blocks("2") // retry query
        .reset_client(&client)
        .await;

    assert_eq!(get_query_url(&client).await, "a");
    assert_eq!(query_block_id(&client).await, "1");
    assert_eq!(query_block_id(&client).await, "2");
    assert_eq!(get_query_url(&client).await, "b");
    NetworkMock::assert_is_empty(&client).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn latency_detection_with_websockets() {
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into(), "b".into()]),
                network_retries_count: 3,
                max_latency: 600,
                latency_detection_interval: 100,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );

    // Check for the fastest
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(10)
        .info(now, 500) // winner
        .url("b")
        .delay(20)
        .info(now, 0) // looser
        .url("a")
        .delay(100)
        .ws_ack()
        .delay(200)
        .ws_ka()
        .url("b")
        .delay(10)
        .ws_ack()
        .url("a")
        .delay(20)
        .info(now, 700) // check latency, bad
        .delay(20)
        .info(now, 0) // looser
        .url("b")
        .delay(10)
        .info(now, 500) // winner
        .reset_client(&client)
        .await;

    assert_eq!(get_query_url(&client).await, "a");
    let subscription = subscribe_collection(
        client.clone(),
        ParamsOfSubscribeCollection {
            collection: "blocks".to_string(),
            filter: None,
            result: "id".to_string(),
        },
        |_| async {},
    )
    .await
    .unwrap();
    let _ = client.env.set_timer(2000).await;
    unsubscribe(client.clone(), subscription).await.unwrap();
    assert_eq!(NetworkMock::get_len(&client).await, 0);
    assert_eq!(get_query_url(&client).await, "b");
    client.get_server_link().unwrap().suspend().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_endpoints() {
    let client = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                endpoints: Some(vec!["a".into(), "b".into()]),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );

    // Check for the fastest
    let now = client.env.now_ms();
    NetworkMock::build()
        .url("a")
        .delay(10)
        .info(now, 500) // winner
        .url("b")
        .delay(20)
        .info(now, 0) // looser
        .reset_client(&client)
        .await;

    let result = crate::net::get_endpoints(client.clone()).await.unwrap();
    assert_eq!(NetworkMock::get_len(&client).await, 0);
    assert_eq!(result.query, "https://a/graphql");
    assert_eq!(result.endpoints, vec!["a".to_string(), "b".to_string()]);
}

fn collect(loaded_messages: &Vec<Value>, messages: &mut Vec<Value>, transactions: &mut Vec<Value>) {
    for message in loaded_messages {
        messages.push(message.clone());
        let tr = &message["dst_transaction"];
        if tr.is_object() {
            transactions.push(tr.clone());
            if let Some(out_messages) = tr["out_messages"].as_array() {
                collect(out_messages, messages, transactions);
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transaction_tree() {
    let client = TestClient::new();

    let run_result = client
        .net_process_function(
            client.giver_address().await,
            TestClient::giver_abi(),
            "sendTransaction",
            json!({
                "dest": "0:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "value": 500_000_000,
                "bounce": true,
            }),
            Signer::Keys {
                keys: TestClient::giver_keys(),
            },
        )
        .await
        .unwrap();

    let messages: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "messages".to_owned(),
                filter: Some(json!({
                    "id": { "eq": run_result.transaction["in_msg"].as_str() },
                    "msg_type": { "eq": 1 },
                })),
                result: r#"
            id dst
            dst_transaction { id aborted
              out_messages { id dst msg_type_name
                dst_transaction { id aborted
                  out_messages { id dst msg_type_name
                    dst_transaction { id aborted
                    }
                  }
                }
              }
            }
        "#
                .to_string(),
                limit: None,
                order: None,
            },
        )
        .await
        .unwrap();

    let abi_registry = vec![TestClient::giver_abi()];

    let mut has_decoded_bodies = false;
    let result: ResultOfQueryTransactionTree = client
        .request_async(
            "net.query_transaction_tree",
            ParamsOfQueryTransactionTree {
                in_msg: messages.result[0]["id"].as_str().unwrap().to_string(),
                abi_registry: Some(abi_registry.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let mut ref_messages = Vec::new();
    let mut ref_transactions = Vec::new();
    collect(&messages.result, &mut ref_messages, &mut ref_transactions);
    let ref_message_ids = ref_messages
        .iter()
        .map(|x| x["id"].as_str().unwrap().to_string())
        .collect::<HashSet<String>>();
    let ref_transaction_ids = ref_transactions
        .iter()
        .map(|x| x["id"].as_str().unwrap().to_string())
        .collect::<HashSet<String>>();
    let actual_message_ids = result
        .messages
        .iter()
        .map(|x| x.id.clone())
        .collect::<HashSet<String>>();
    assert_eq!(actual_message_ids.len(), 3);
    let actual_transaction_ids = result
        .transactions
        .iter()
        .map(|x| x.id.clone())
        .collect::<HashSet<String>>();

    assert_eq!(ref_message_ids, actual_message_ids);
    assert_eq!(ref_transaction_ids, actual_transaction_ids);
    for msg in result.messages {
        if msg.decoded_body.is_some() {
            has_decoded_bodies = true;
        }
    }
    assert!(has_decoded_bodies);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn order_by_fallback() {
    let params: ParamsOfQueryCollection = serde_json::from_str(
        r#"
        {
            "collection": "messages",
            "result": "id",
            "order": [{"path": "id", "direction": "ASC"}]
        }
        "#,
    )
    .unwrap();
    assert_eq!(params.order.unwrap()[0].path, "id");
    assert!(serde_json::from_str::<ParamsOfQueryCollection>(
        r#"
        {
            "collection": "messages",
            "result": "id",
            "orderBy": [{"path": "id", "direction": "ASC"}]
        }
        "#,
    )
    .is_err());
    assert!(serde_json::from_str::<ParamsOfQueryCollection>(
        r#"
        {
            "collection": "messages",
            "result": "id",
            "orderBy": [
                {"path": "id1", "direction": "ASC"}
            ],
            "order": [
                {"path": "id", "direction": "ASC"}
            ]
        }
        "#,
    )
    .is_err());
    let client = TestClient::new();

    let result: ClientResult<ResultOfQueryCollection> = client
        .request_async(
            "net.query_collection",
            json!({
                "collection": "messages",
                "result": "id",
                "limit": 1,
            }),
        )
        .await;
    assert!(result.is_ok());
    let result: ClientResult<ResultOfQueryCollection> = client
        .request_async(
            "net.query_collection",
            json!({
                "collection": "messages",
                "result": "id",
                "limit": 1,
                "order": [{"path":"id", "direction":"ASC"}],
            }),
        )
        .await;
    assert!(result.is_ok());
    let result: ClientResult<ResultOfQueryCollection> = client
        .request_async(
            "net.query_collection",
            json!({
                "collection": "messages",
                "result": "id",
                "limit": 1,
                "orderBy": [{"path":"id", "direction":"ASC"}],
            }),
        )
        .await;
    if let Err(err) = &result {
        println!("{:?}", err);
    }
    assert!(result.is_err());
}

#[test]
fn test_subscription_gql() {
    let query = GraphQLQuery::with_collection_subscription("counterparties", &Value::Null, "id");
    assert_eq!(query.query, "subscription counterparties($filter: CounterpartyFilter) { counterparties(filter: $filter) { id } }");

    let query = GraphQLQuery::with_collection_subscription("messages", &Value::Null, "id");
    assert_eq!(
        query.query,
        "subscription messages($filter: MessageFilter) { messages(filter: $filter) { id } }"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn low_level_subscribe() {
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
            "net.subscribe",
            ParamsOfSubscribe {
                subscription: r#"
                subscription test($dst: String) {
                    messages(filter: { dst: { eq: $dst } }) {
                        id
                    }
                }
                "#
                .to_string(),
                variables: Some(json!({
                    "dst": client.giver_address().await,
                })),
            },
            callback,
        )
        .await
        .unwrap();

    client
        .get_tokens_from_giver_async(&client.giver_address().await, None)
        .await;

    std::thread::sleep(std::time::Duration::from_millis(1000));
    assert_ne!(messages.lock().await.len(), 0);

    let _: () = client
        .request_async("net.unsubscribe", handle)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_using_ws() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": TestClient::endpoints(),
            "queries_protocol": "WS"
        }
    }));

    let accounts: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({})),
                result: "id".to_owned(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert!(accounts.result.len() > 0);
    let messages: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "messages".to_owned(),
                filter: Some(json!({})),
                result: "id".to_owned(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert!(messages.result.len() > 0);
}
