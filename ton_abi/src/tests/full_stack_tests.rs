use ed25519_dalek::*;
use sha2::Sha512;

use tvm::stack::{BuilderData, SliceData};

use json_abi::*;

const WALLET_ABI: &str = r#"{
    "ABI version" : 1,
    "setTime": false,
    "functions" :    [{
            "inputs": [
                {"name": "recipient", "type": "fixedbytes32"},
                {"name": "value", "type": "gram"}
            ],
            "name": "sendTransaction",
            "outputs": [
                {"name": "transaction", "type": "uint64"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "type", "type": "uint8"},
                {"name": "value", "type": "gram"},
                {"name": "meta", "type": "bytes"}
            ],
            "name": "createLimit",
            "outputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "value", "type": "gram"},
                {"name": "meta", "type": "bytes"}
            ],
            "name": "changeLimitById",
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "removeLimit",
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "getLimitById",
            "outputs": [
                {
                    "name": "limitInfo",
                    "type": "tuple",
                    "components": [
                        {"name": "value", "type": "gram"},
                        {"name": "type", "type": "uint8"},
                        {"name": "meta", "type": "bytes"}
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
            "inputs": [{"name": "address", "type": "fixedbytes32" }],
            "name": "setSubscriptionAccount",
            "outputs": []
        }, {
            "inputs": [],
            "name": "getSubscriptionAccount",
            "outputs": [{"name": "address", "type": "fixedbytes32" }]
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
        false,
        None,
    ).unwrap();

    let mut expected_tree = BuilderData::with_bitstring(vec![0x54, 0xc1, 0xf4, 0x0f, 0x80]).unwrap();
    expected_tree.prepend_reference(BuilderData::new());

    let test_tree = SliceData::from(test_tree);
    let expected_tree = SliceData::from(expected_tree);
    assert_eq!(test_tree, expected_tree);

    let response = decode_unknown_function_call(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
        false
    ).unwrap();

    assert_eq!(response.params, params);
    assert_eq!(response.function_name, "constructor");


    let test_tree = SliceData::from_raw(vec![0xd4, 0xc1, 0xf4, 0x0f, 0x80], 32);

    let response = decode_unknown_function_response(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
        false
    )
    .unwrap();

    assert_eq!(response.params, params);
    assert_eq!(response.function_name, "constructor");


    let response = decode_function_response(
        WALLET_ABI.to_owned(),
        "constructor".to_owned(),
        test_tree,
        false
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

    let expected_params = r#"{"type":"0x1","value":"0xc","meta":""}"#;

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "createLimit".to_owned(),
        params.to_owned(),
        false,
        Some(&pair),
    )
    .unwrap();

    let mut test_tree = SliceData::from(test_tree);

    let response = decode_unknown_function_call(
        WALLET_ABI.to_owned(),
        test_tree.clone(),
        false
    )
    .unwrap();

    assert_eq!(response.params, expected_params);
    assert_eq!(response.function_name, "createLimit");

    let mut expected_tree = BuilderData::with_bitstring(vec![
        0x79, 0x63, 0xc9, 0x74, 0x01, 0x10, 0xc8
    ]).unwrap();
    expected_tree.append_reference(BuilderData::new());

    test_tree.checked_drain_reference().unwrap();
    assert_eq!(test_tree, SliceData::from(expected_tree));


    let expected_response = r#"{"limitId":"0x0","error":"-0x1"}"#;

    let response_tree = SliceData::from(
        BuilderData::with_bitstring(
            vec![0xf9, 0x63, 0xc9, 0x74, 0x00, 0xFF, 0x80])
        .unwrap());

    let response = decode_function_response(
        WALLET_ABI.to_owned(),
        "createLimit".to_owned(),
        response_tree.clone(),
        false
    )
    .unwrap();

    assert_eq!(response, expected_response);


    let response = decode_unknown_function_response(
        WALLET_ABI.to_owned(),
        response_tree,
        false
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
        false,
        None,
    )
    .unwrap();

    let mut expected_tree = BuilderData::with_bitstring(vec![0x0F, 0xEF, 0x4E, 0x34, 0x02, 0x80]).unwrap();
    expected_tree.prepend_reference(BuilderData::new());

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_add_signature_full() {
    let params = r#"{"limitId":"0x2"}"#;

    let (msg, data_to_sign) = prepare_function_call_for_sign(
        WALLET_ABI.to_owned(),
        "getLimitById".to_owned(),
        params.to_owned()
    )
    .unwrap();

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());
    let signature = pair.sign::<Sha512>(&data_to_sign).to_bytes().to_vec();

    let msg = add_sign_to_function_call(&signature, &pair.public.to_bytes(), msg.into()).unwrap();

    let decoded = decode_unknown_function_call(WALLET_ABI.to_owned(), msg.into(), false).unwrap();

    assert_eq!(decoded.params, params);
}
