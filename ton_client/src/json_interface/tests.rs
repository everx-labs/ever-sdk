use serde_json::Value;

use crate::tests::TestClient;

#[test]
fn test_invalid_params_errors() {
    check_client_error_msg(
        json!({}),
        &[
            r#"Field "abi" value is expected, but not provided."#,
            r#"Field "signer" value is expected, but not provided."#,
        ],
        &[],
    );

    check_client_error_msg(
        json!({
            "abi": "abc",
            "signer": "edf",
        }),
        &[
            "Field \"abi\" must be a structure:\n\
            {\n    \
                \"type\": one of \"Contract\", \"Json\", \"Handle\", \"Serialized\",\n    \
                ... fields of a corresponding structure, or \"value\" in a case of scalar\n\
            }.",
            "Field \"signer\" must be a structure:\n\
            {\n    \
                \"type\": one of \"None\", \"External\", \"Keys\", \"SigningBox\",\n    \
                ... fields of a corresponding structure, or \"value\" in a case of scalar\n\
            }.",
        ],
        &["Abi", "Signer"],
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
            r#"Field "keys" value is expected, but not provided."#,
        ],
        &[],
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
        &[],
    );
}

const TIP_START_MARKER: &str = "\nTip: ";

struct TipsIterator<'msg> {
    message: &'msg str,
    current_index: Option<usize>,
}

impl<'msg> TipsIterator<'msg> {
    pub fn new(message: &'msg str) -> Self {
        Self {
            message,
            current_index: message.find(TIP_START_MARKER),
        }
    }
}

impl<'msg> Iterator for TipsIterator<'msg> {
    type Item = &'msg str;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let tip_start = index + TIP_START_MARKER.len();
            self.current_index = self.message[tip_start..]
                .find(TIP_START_MARKER)
                .map(|index| index + tip_start);
            &self.message[tip_start..self.current_index.unwrap_or(self.message.len())]
        })
    }
}

fn check_client_error_msg(
    params: Value,
    expected_tips: &[&str],
    expected_helpers_suggestions: &[&str],
) {
    let error = TestClient::new()
        .request_json("abi.encode_message", params)
        .unwrap_err();
    let tips: Vec<&str> = TipsIterator::new(&error.message).collect();
    assert_eq!(
        tips.len(),
        expected_tips.len(),
        "Tips count mismatch. Actual message: {}",
        error.message
    );
    for i in 0..expected_tips.len() {
        assert_eq!(
            tips[i], expected_tips[i],
            r#"Expected tip "{}" at position {}, but actual message is: "{}"."#,
            expected_tips[i], i, error.message,
        )
    }
    if expected_helpers_suggestions.len() > 0 {
        let actual_classes = error.data["suggest_use_helper_for"].as_array().unwrap();
        assert_eq!(
            actual_classes.len(),
            expected_helpers_suggestions.len(),
            "Helpers suggestions mismatch. Expected: {:?}, actual: {:?}.",
            expected_helpers_suggestions,
            actual_classes,
        );
        for i in 0..expected_helpers_suggestions.len() {
            assert_eq!(
                actual_classes[i], expected_helpers_suggestions[i],
                r#"Helpers' suggestions mismatch. Expected: {:?}, actual: {:?}."#,
                expected_helpers_suggestions, actual_classes,
            )
        }
    }
}
