use crate::client::ResultOfGetApiReference;
use crate::crypto::default_mnemonic_word_count;
use crate::json_interface::modules::ClientModule;
use crate::json_interface::runtime::Runtime;
use crate::net::{subscribe_collection, unsubscribe, ParamsOfSubscribeCollection};
use crate::tests::TestClient;
use crate::{create_context, destroy_context, ClientConfig};
use api_info::ApiModule;
use serde_json::Value;
use std::time::Duration;

#[test]
fn test_config_fields() {
    let config = serde_json::from_str::<ClientConfig>(
        r#"{
        "crypto": null,
        "abi": {
            "message_expiration_timeout": null
        },
        "network": {
            "max_reconnect_timeout": 100
        }
    }
    "#,
    )
    .unwrap();
    assert_eq!(
        config.crypto.mnemonic_word_count,
        default_mnemonic_word_count()
    );
    assert_eq!(config.network.max_reconnect_timeout, 100);
}

#[test]
fn test_config() {
    let client = TestClient::new_with_config(json!({
        "abi": {
            "message_expiration_timeout": 456
        },
        "network": {
            "max_reconnect_timeout": 123
        }
    }));
    let config: ClientConfig = client
        .request_no_params(&format!(
            "{}.{}",
            ClientModule::api().name,
            crate::client::config_api().name
        ))
        .unwrap();
    assert_eq!(config.abi.message_expiration_timeout, 456);
    assert_eq!(config.network.max_reconnect_timeout, 123);
}

#[test]
fn api_reference() {
    let client = TestClient::new();
    let api: ResultOfGetApiReference = client
        .request_no_params(&format!(
            "{}.{}",
            ClientModule::api().name,
            crate::client::get_api_reference_api().name
        ))
        .unwrap();
    assert_ne!(api.api.modules.len(), 0);
    assert_eq!(api.api.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_invalid_params_error_secret_stripped() {
    let public = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let secret = "9234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let error = super::errors::Error::invalid_params(
        &format!(
            r#"{{"address":"0:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "public":"{}",
            "secret":"{}"}}"#,
            public, secret
        ),
        "error",
    );
    assert!(!error.message.contains(secret));
}

#[test]
fn test_sync_calls() {

}

#[tokio::test]
async fn test_memory_leak() {
    for _ in 0..1 {
        let config = json!({
            "network": {
                "endpoints": TestClient::endpoints(),
                "queries_protocol": "WS",
            }
        });
        let ctx = create_context(config.to_string());
        let context = serde_json::from_str::<Value>(&ctx).unwrap()["result"].as_i64().unwrap() as u32;
        {
            let context = Runtime::required_context(context).unwrap();
            let subscription = subscribe_collection(
                context.clone(),
                ParamsOfSubscribeCollection {
                    collection: "blocks".to_string(),
                    result: "id".to_string(),
                    filter: None,
                },
                |_| async {},
            )
            .await
            .unwrap();
            unsubscribe(context.clone(), subscription).await.unwrap();
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let subscription = subscribe_collection(
                context.clone(),
                ParamsOfSubscribeCollection {
                    collection: "blocks".to_string(),
                    result: "id".to_string(),
                    filter: None,
                },
                |_| async {},
            )
            .await
            .unwrap();
            unsubscribe(context.clone(), subscription).await.unwrap();
        }
        destroy_context(context);
    }
}
