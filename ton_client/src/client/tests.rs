use crate::client::ResultOfGetApiReference;
use crate::crypto::default_mnemonic_word_count;
use crate::json_interface::modules::ClientModule;
use crate::tests::TestClient;
use crate::ClientConfig;
use api_info::ApiModule;

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
