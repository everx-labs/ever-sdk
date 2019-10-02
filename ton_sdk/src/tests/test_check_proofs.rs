use crate::*;
use super::*;

const COMPLETE_JSON: &str = r#"
    {
        "string1": "value1",
        "number1": 123,
        "string2": "value2",
        "number3": 12345,
        "array1": [1, 2, 3],
        "array2": [
            {
                "string1": "value1",
                "number1": 123
            },
            {
                "string2": "value2",
                "number2": 12123
            }
        ],
        "object1": {
            "string1": "value1",
            "number1": 123,
            "string2": "value2",
            "number3": 12345
        }
    }"#;



#[test]
fn test_check_correct_incomplete_json1() {
    let complete_json = serde_json::from_str(COMPLETE_JSON).unwrap();

    let correct_incomplete_json = serde_json::from_str(r#"
    {
        "string1": "value1",
        "number1": 123,
        "string2": null,
        "array1": [1, 2, 3],
        "array2": [
            {
                "string1": "value1"
            },
            {
                "string2": "value2",
                "number2": null
            }
        ],
        "object1": {
            "string1": "value1",
            "string2": "value2",
            "number3": null
        },
        "boc": "flkajhfdklahsfjlkahfkasdf",
        "proof": "12189hd98y2819467129y4091284"
    }"#).unwrap();

    check_incomplete_json(&correct_incomplete_json, &complete_json).unwrap();
}

#[test]
fn test_check_correct_incomplete_json2() {
    let complete_json = serde_json::from_str(COMPLETE_JSON).unwrap();

    let correct_incomplete_json = serde_json::from_str(r#"
    {
        "number1": 123,
        "string2": null,
        "array1": null,
        "boc": "flkajhfdklahsfjlkahfkasdf",
        "proof": "12189hd98y2819467129y4091284"
    }"#).unwrap();

    check_incomplete_json(&correct_incomplete_json, &complete_json).unwrap();
}

#[test]
fn test_check_wrong_incomplete_json1() {
    let complete_json = serde_json::from_str(COMPLETE_JSON).unwrap();

    let correct_incomplete_json = serde_json::from_str(r#"
    {
        "string1": "value1",
        "number1": 123,
        "string2": "value2",
        "number3": 12345,
        "array1": [1, 2, 3],
        "array2": [
            {
                "string1": "value1",
                "number1": 123
            },
            {
                "string2": "value2",
                "number2": 12123
            }
        ],
        "object1": {
            "string1": "value1_1"
        }
    }"#).unwrap();

    match check_incomplete_json(&correct_incomplete_json, &complete_json) {
        SdkResult::Err(SdkError(SdkErrorKind::WrongJson, _)) => (),
        _ => panic!(),
    }
}

#[test]
fn test_check_wrong_incomplete_json2() {
    let complete_json = serde_json::from_str(COMPLETE_JSON).unwrap();

    let correct_incomplete_json = serde_json::from_str(r#"
    {
        "string1": "value1",
        "number1": 123,
        "string2": "value2",
        "number3": 12345,
        "array1": [1, 2, 3],
        "array2": [
            {
                "string1": "value1"
            },
            {
                "string2": "value2"
            },
            {
                "string3": "value3"
            }
        ],
        "boc": "flkajhfdklahsfjlkahfkasdf",
        "proof": "12189hd98y2819467129y4091284"
    }"#).unwrap();

    match check_incomplete_json(&correct_incomplete_json, &complete_json) {
        SdkResult::Err(SdkError(SdkErrorKind::WrongJson, _)) => (),
        _ => panic!(),
    }
}

#[test]
fn test_check_wrong_incomplete_json3() {
    let complete_json = serde_json::from_str(COMPLETE_JSON).unwrap();

    let correct_incomplete_json = serde_json::from_str(r#"
    {
        "string1": "value1",
        "number1": 123,
        "number4": 4
    }"#).unwrap();

    match check_incomplete_json(&correct_incomplete_json, &complete_json) {
        SdkResult::Err(SdkError(SdkErrorKind::WrongJson, _)) => (),
        _ => panic!(),
    }
}