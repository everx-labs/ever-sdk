use ed25519_dalek::*;
use sha2::Sha512;

use tvm::stack::{BuilderData, SliceData};

use json_abi::*;

const WALLET_ABI: &str = r#"{
    "ABI version" : 0,
    "functions" :    [{
            "inputs": [
                {"name": "recipient", "type": "bits256"},
                {"name": "value", "type": "duint"}
            ],
            "name": "sendTransaction",
            "signed": true,
            "outputs": [
                {"name": "transaction", "type": "uint64"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "type", "type": "uint8"},
                {"name": "value", "type": "duint"},
                {"name": "meta", "type": "bitstring"}
            ],
            "name": "createLimit",
            "signed": true,
            "outputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "value", "type": "duint"},
                {"name": "meta", "type": "bitstring"}
            ],
            "name": "changeLimitById",
            "signed": true,
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "removeLimit",
            "signed": true,
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "getLimitById",
            "outputs": [
                {
                    "name": "limitInfo",
                    "type": "tuple",
                    "components": [
                        {"name": "value", "type": "duint"},
                        {"name": "type", "type": "uint8"},
                        {"name": "meta", "type": "bitstring"}
                        ]
                },
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getLimits",
            "outputs": [
                {"name": "list", "type": "uint8[]"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getVersion",
            "outputs": [
                {
                    "name": "version",
                    "type": "tuple",
                    "components": [
                        {"name": "major", "type": "uint16"},
                        {"name": "minor", "type": "uint16"}
                    ]
                },
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getBalance",
            "outputs": [{"name": "balance", "type": "uint64"}]
        }, {
            "inputs": [],
            "name": "constructor",
            "outputs": []                            
        }, {
            "inputs": [{"name": "address", "type": "bits256" }],
            "name": "setSubscriptionAccount",
                    "signed": true,
            "outputs": []                            
        }, {
            "inputs": [],
            "name": "getSubscriptionAccount",
            "outputs": [{"name": "address", "type": "bits256" }]                            
        }
    ]
}
"#;

#[test]
fn test_constructor_call() {
    let params = r#"{}"#;

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "constructor".to_owned(),
        params.to_owned(),
        None,
    )
    .unwrap();

    let mut expected_tree = BuilderData::with_bitstring(vec![0x00, 0xAC, 0x81, 0x0A, 0x6D, 0x80]).unwrap();
    expected_tree.prepend_reference(BuilderData::new());

    assert_eq!(test_tree, expected_tree);


    let mut test_tree = SliceData::from(test_tree);

    let response = decode_unknown_function_call(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
    )
    .unwrap();

    assert_eq!(response.params, params);
    assert_eq!(response.function_name, "constructor");


    test_tree.checked_drain_reference().unwrap();

    let response = decode_unknown_function_response(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
    )
    .unwrap();

    assert_eq!(response.params, params);
    assert_eq!(response.function_name, "constructor");


    let response = decode_function_response(
        WALLET_ABI.to_owned(),
        "constructor".to_owned(),
        test_tree,
    )
    .unwrap();

    assert_eq!(response, params);
}

#[test]
fn test_signed_call() {
    let params = r#"
    {
        "type": 1,
        "value": 12,
        "meta": ""
    }"#;

    let expected_params = r#"{"type":"0x1","value":"0xc","meta":"x"}"#;

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "createLimit".to_owned(),
        params.to_owned(),
        Some(&pair),
    )
    .unwrap();

    let mut test_tree = SliceData::from(test_tree);

    let response = decode_unknown_function_call(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
    )
    .unwrap();

    assert_eq!(response.params, expected_params);
    assert_eq!(response.function_name, "createLimit");

    let expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0x27, 0xEF, 0x50, 0x87, 0x01, 0x0C, 0b01000000, 0x00, 0x00, 0x00, 0b00010000
    ]).unwrap();

    test_tree.checked_drain_reference().unwrap();
    assert_eq!(test_tree, SliceData::from(expected_tree));


    let expected_response = r#"{"limitId":"0x0","error":"-0x1"}"#;

    let response_tree = SliceData::from(
        BuilderData::with_bitstring(
            vec![0x00, 0x27, 0xEF, 0x50, 0x87, 0x00, 0xFF, 0x80])
        .unwrap());

    let response = decode_function_response(
        WALLET_ABI.to_owned(),
        "createLimit".to_owned(),
        response_tree.clone(),
    )
    .unwrap();

    assert_eq!(response, expected_response);


    let response = decode_unknown_function_response(
        WALLET_ABI.to_owned(),
        response_tree,
    )
    .unwrap();

    assert_eq!(response.params, expected_response);
    assert_eq!(response.function_name, "createLimit");
}

#[test]
fn test_not_signed_call() {
    let params = r#"{
        "limitId": "0x2"
    }"#;

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "getLimitById".to_owned(),
        params.to_owned(),
        None,
    )
    .unwrap();

    let mut expected_tree = BuilderData::with_bitstring(vec![0x00, 0xDA, 0x37, 0x46, 0x4F, 0x02, 0x80]).unwrap();
    expected_tree.prepend_reference(BuilderData::new());

    assert_eq!(test_tree, expected_tree);
}
