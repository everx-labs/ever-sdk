use super::*;
use crate::queries::{ParamsOfQueryCollection, ResultOfQueryCollection};

#[test]
fn test_parallel_requests() {
    let client1 = TestClient::new();
    let client2 = TestClient::new();
    let client3 = client1.clone();

    let start = std::time::Instant::now();
    let timeout: u32 = 5000;
    let long_wait = std::thread::spawn(move || {
        client3.request_json("queries.wait_for_collection",
            json!({
                    "collection": "accounts".to_owned(),
                    "filter": json!({
                        "id": { "eq": "123" }
                    }),
                    "result": "id",
                    "timeout": timeout
                }),
        ).unwrap_err();
        client3
    });

    std::thread::sleep(std::time::Duration::from_millis(500));

    let query = |client: &TestClient| {
        let _: ResultOfQueryCollection = client.request(
            "queries.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({})),
                result: "id".to_owned(),
                limit: Some(1),
                order: None,
            }
        );
    };

    // check that request with another context doesn't wait
    query(&client2);
    assert!(start.elapsed().as_millis() < timeout as u128);

    // check that request with same context doesn't wait too
    query(&client1);
    assert!(start.elapsed().as_millis() < timeout as u128);

    long_wait.join().unwrap();
    assert!(start.elapsed().as_millis() > timeout as u128);
}

#[test]
fn test_deferred_init() {
    let client = TestClient::new_with_config(
        json!({
            "network": {
                "server_address": "123"
            }
        })
    );

    // local functions should work
    client.generate_sign_keys();

    // deferred network init should fail due to wrong server address
    let result = client.request_json(
        "queries.query_collection",
        json!({
            "collection": "accounts",
            "result": "id".to_owned(),
        })
    ).unwrap_err();
    //println!("{:#?}", result);


    assert_eq!(result.code, crate::client::ErrorCode::HttpRequestSendError as isize);
}
