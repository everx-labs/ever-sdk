/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ed25519_dalek::*;
use sha2::Sha512;

use ton_types::{BuilderData, SliceData};
use ton_types::dictionary::HashmapE;
use ton_block::{MsgAddressInt, Serializable};

use json_abi::*;

const WALLET_ABI: &str = r#"{
    "ABI version": 1,
    "setTime": false,
    "functions": [
        {
            "name": "sendTransaction",
            "inputs": [
                {"name":"dest","type":"address"},
                {"name":"value","type":"uint128"},
                {"name":"bounce","type":"bool"}
            ],
            "outputs": [
            ]
        },
        {
            "name": "setSubscriptionAccount",
            "inputs": [
                {"name":"addr","type":"address"}
            ],
            "outputs": [
            ]
        },
        {
            "name": "getSubscriptionAccount",
            "inputs": [
            ],
            "outputs": [
                {"name":"value0","type":"address"}
            ]
        },
        {
            "name": "createOperationLimit",
            "inputs": [
                {"name":"value","type":"uint256"}
            ],
            "outputs": [
                {"name":"value0","type":"uint256"}
            ]
        },
        {
            "name": "createArbitraryLimit",
            "inputs": [
                {"name":"value","type":"uint256"},
                {"name":"period","type":"uint32"}
            ],
            "outputs": [
                {"name":"value0","type":"uint64"}
            ]
        },
        {
            "name": "changeLimit",
            "inputs": [
                {"name":"limitId","type":"uint64"},
                {"name":"value","type":"uint256"},
                {"name":"period","type":"uint32"}
            ],
            "outputs": [
            ]
        },
        {
            "name": "deleteLimit",
            "inputs": [
                {"name":"limitId","type":"uint64"}
            ],
            "outputs": [
            ]
        },
        {
            "name": "getLimit",
            "inputs": [
                {"name":"limitId","type":"uint64"}
            ],
            "outputs": [
                {"components":[{"name":"value","type":"uint256"},{"name":"period","type":"uint32"},{"name":"ltype","type":"uint8"},{"name":"spent","type":"uint256"},{"name":"start","type":"uint32"}],"name":"value0","type":"tuple"}
            ]
        },
        {
            "name": "getLimitCount",
            "inputs": [
            ],
            "outputs": [
                {"name":"value0","type":"uint64"}
            ]
        },
        {
            "name": "getLimits",
            "inputs": [
            ],
            "outputs": [
                {"name":"value0","type":"uint64[]"}
            ]
        },
        {
            "name": "constructor",
            "inputs": [
            ],
            "outputs": [
            ]
        }
    ],
    "events": [{
        "name": "event",
        "inputs": [
            {"name":"param","type":"uint8"}
        ]
    }
    ],
    "data": [
        {"key":101,"name":"subscription","type":"address"},
        {"key":100,"name":"owner","type":"uint256"}
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
        "value": 12,
        "period": 30
    }"#;

    let expected_params = r#"{"value":"0xc","period":"0x1e"}"#;

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "createArbitraryLimit".to_owned(),
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
    assert_eq!(response.function_name, "createArbitraryLimit");

    let mut vec = vec![0x3C, 0x0B, 0xB9, 0xBC];
    vec.resize(vec.len() + 31, 0);
    vec.extend_from_slice(&[0x0C, 0x00, 0x00, 0x00, 0x1E, 0x80]);

    let expected_tree = BuilderData::with_bitstring(vec).unwrap();

    test_tree.checked_drain_reference().unwrap();
    assert_eq!(test_tree, SliceData::from(expected_tree));


    let expected_response = r#"{"value0":"0x0"}"#;

    let response_tree = SliceData::from(
        BuilderData::with_bitstring(
            vec![0xBC, 0x0B, 0xB9, 0xBC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])
        .unwrap());

    let response = decode_function_response(
        WALLET_ABI.to_owned(),
        "createArbitraryLimit".to_owned(),
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
    assert_eq!(response.function_name, "createArbitraryLimit");
}

#[test]
fn test_not_signed_call() {
    let params = r#"{
        "limitId": "0x2"
    }"#;

    let test_tree = encode_function_call(
        WALLET_ABI.to_owned(),
        "getLimit".to_owned(),
        params.to_owned(),
        false,
        None,
    )
    .unwrap();

    let mut expected_tree = BuilderData::with_bitstring(vec![
            0x23, 0xF3, 0x3E, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x80
        ]).unwrap();
    expected_tree.prepend_reference(BuilderData::new());

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_add_signature_full() {
    let params = r#"{"limitId":"0x2"}"#;

    let (msg, data_to_sign) = prepare_function_call_for_sign(
        WALLET_ABI.to_owned(),
        "getLimit".to_owned(),
        params.to_owned()
    )
    .unwrap();

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());
    let signature = pair.sign::<Sha512>(&data_to_sign).to_bytes().to_vec();

    let msg = add_sign_to_function_call(&signature, &pair.public.to_bytes(), msg.into()).unwrap();

    let decoded = decode_unknown_function_call(WALLET_ABI.to_owned(), msg.into(), false).unwrap();

    assert_eq!(decoded.params, params);
}

#[test]
fn test_find_event() {
    let event_tree = SliceData::from(
        BuilderData::with_bitstring(
            vec![0x13, 0x47, 0xD7, 0x9D, 0xFF, 0x80])
        .unwrap());

    let decoded = decode_unknown_function_response(WALLET_ABI.to_owned(), event_tree, false).unwrap();

    assert_eq!(decoded.function_name, "event");
    assert_eq!(decoded.params, r#"{"param":"0xff"}"#);
}

#[test]
fn test_insert_pubkey() {
    let event_tree = SliceData::from(
        BuilderData::with_bitstring(
            vec![0x13, 0x47, 0xD7, 0x9D, 0xFF, 0x80])
        .unwrap());

    let decoded = decode_unknown_function_response(WALLET_ABI.to_owned(), event_tree, false).unwrap();

    assert_eq!(decoded.function_name, "event");
    assert_eq!(decoded.params, r#"{"param":"0xff"}"#);
}

#[test]
fn test_store_pubkey() {
    let mut test_map = HashmapE::with_bit_len(Contract::DATA_MAP_KEYLEN);
    let test_pubkey = vec![11u8; 32];
    test_map.set(
        0u64.write_to_new_cell().unwrap().into(),
        &BuilderData::with_raw(vec![0u8; 32], 256).unwrap().into(),
    ).unwrap();

    let data = test_map.write_to_new_cell().unwrap();

    let new_data = Contract::insert_pubkey(data.into(), &test_pubkey).unwrap();

    let new_map = HashmapE::with_data(Contract::DATA_MAP_KEYLEN, new_data.into());
    let key_slice = new_map.get(
        0u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(key_slice.get_bytestring(0), test_pubkey);
}

#[test]
fn test_update_contract_data() {
    let mut test_map = HashmapE::with_bit_len(Contract::DATA_MAP_KEYLEN);
    test_map.set(
        0u64.write_to_new_cell().unwrap().into(),
        &BuilderData::with_raw(vec![0u8; 32], 256).unwrap().into(),
    ).unwrap();

    let params = r#"{
        "subscription": "0:1111111111111111111111111111111111111111111111111111111111111111",
        "owner": "0x2222222222222222222222222222222222222222222222222222222222222222"
     }
    "#;

    let data = test_map.write_to_new_cell().unwrap();
    let new_data = update_contract_data(WALLET_ABI, params, data.into()).unwrap();
    let new_map = HashmapE::with_data(Contract::DATA_MAP_KEYLEN, new_data.into());


    let key_slice = new_map.get(
        0u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(key_slice.get_bytestring(0), vec![0u8; 32]);


    let subscription_slice = new_map.get(
        101u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        subscription_slice,
        MsgAddressInt::with_standart(None, 0, vec![0x11; 32].into()).unwrap().write_to_new_cell().unwrap().into());


    let owner_slice = new_map.get(
        100u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(owner_slice.get_bytestring(0), vec![0x22; 32]);
}