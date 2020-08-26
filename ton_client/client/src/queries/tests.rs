use crate::tests::TestClient;
use super::*;

#[test]
fn block_signatures() {
    let client = TestClient::new();

    let _: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "blocks_signatures".to_owned(),
            filter: Some(json!({})),
            result: "id".to_owned(),
            limit: Some(1),
            order: None,
            timeout: None
        }
    ).unwrap();
}

#[test]
fn all_accounts() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({})),
            result: "id balance".to_owned(),
            limit: None,
            order: None,
            timeout: None
        }
    ).unwrap();

    assert!(accounts.result.len() > 0);
}

#[test]
fn ranges() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "created_at": { "gt": 1562342740 }
            })),
            result: "body created_at".to_owned(),
            limit: None,
            order: None,
            timeout: None
        }
    ).unwrap();

    assert!(accounts.result[0]["created_at"].as_u64().unwrap() > 1562342740);
}

#[test]
fn wait_for() {
    let handle = std::thread::spawn(|| {
        let client = TestClient::new();
        let now = ton_sdk::Contract::now();
        let transactions: ResultOfQueryCollection = client.request(
            "queries.query_collection",
            ParamsOfQueryCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "now": { "gt": now }
                })),
                result: "id now".to_owned(),
                limit: None,
                order: None,
                timeout: Some(ton_sdk::types::DEFAULT_WAIT_TIMEOUT)
            }
        ).unwrap();

        assert!(transactions.result[0]["now"].as_u64().unwrap() > now as u64);
    });

    let client = TestClient::new();

    client.get_grams_from_giver(&TestClient::get_giver_address(), None);

    handle.join().unwrap();
}
