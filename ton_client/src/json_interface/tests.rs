use regex::{Regex, Match};
use serde_json::Value;

use crate::tests::TestClient;

lazy_static! {
    static ref TIP_RE: Regex = Regex::new("\nTip: ([^\n]+)").unwrap();
}

#[test]
fn test_invalid_params_errors() {
    check_client_error_msg(
        json!({}),
        &[
            r#"Field "abi" value is expected, but not provided."#,
            r#"Field "signer" value is expected, but not provided."#,
        ],
    );

    check_client_error_msg(
        json!({
            "abi": "abc",
            "signer": "edf",
        }),
        &[
            "Field \"abi\" is expected to be an enumeration of the class `Abi` represented by the \
                JSON object with a field \"type\" identifying its type (one of \
                \"Contract\", \"Json\", \"Handle\", \"Serialized\") and additional fields with \
                data from the object of a corresponding class.",
            "Field \"signer\" is expected to be an enumeration of the class `Signer` represented \
                by the JSON object with a field \"type\" identifying its type (one of \
                \"None\", \"External\", \"Keys\", \"SigningBox\") and additional fields with data \
                from the object of a corresponding class.",
        ],
    );

    check_client_error_msg(
        json!({
                "abi": {
                    "type": "Json",
                    "Json": {}
                },
                "signer": {
                    "type": "Keys",
                    "public": "123",
                    "secret": "123",
                }
            }),
        &[
            r#"Field "value" value is expected, but not provided."#,
            r#"Field "keys" value is expected, but not provided."#
        ],
    );

    check_client_error_msg(
        json!({
                "abi": {
                    "type": "Json",
                    "value": "{}"
                },
                "signer": {
                    "type": "Keys",
                    "keys": {
                        "public": "123",
                        "secret": "123",
                    }
                }
            }),
        &[],
    );
}

fn check_client_error_msg(params: Value, expected_tips: &[&str]) {
    let error = TestClient::new().request_json("abi.encode_message", params).unwrap_err();
    let tips: Vec<Match> = TIP_RE.find_iter(&error.message)
        .map(|tip| TIP_RE.captures(tip.as_str()).unwrap().get(1).unwrap())
        .collect();
    assert_eq!(tips.len(), expected_tips.len(), "Tips count mismatch");
    for i in 0..expected_tips.len() {
        assert_eq!(
            tips[i].as_str(),
            expected_tips[i],
            r#"Expected tip "{}" at position {}, but actual message is: "{}"."#,
            expected_tips[i],
            i,
            error.message,
        )
    }
}

