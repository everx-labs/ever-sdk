/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::tests_common::PIGGY_BANK_CONTRACT_ABI;
use serde_json::Value;
use ton_vm::stack::StackItem;
use ton_vm::stack::integer::IntegerData;
use std::sync::Arc;

const CONTRACT: &str = r#"{
    "id": "0:19ef6e8e83c5287b85ad0bfebf2fb1af6b5ad0844253d764f9675d772af0a56a",
    "balance": "0xb1182e7",
    "last_paid": 1584684866,
    "code": "te6ccgECEQEAA0MAAib/APSkICLAAZL0oOGK7VNYMPShAwEBCvSkIPShAgAAAgEgBgQB/P9/IdMAAY4igQIA1xgg+QEBcO1E0PQFgED0DvKK1wv/ASH4ZQMB+RDyqN7tRNAg10nCAY4Z9ATTP9MA1AH4atM/Afhr0X/4Yfhm+GP4Yo4U9AX4YnD4Y3D4ZsjJ+Gpw+Gt/+GHi0z8B+EMhuSCfMCD4I4ED6KiCCBt3QKC53gUAMpMg+GOUgDTy8OIw0x8B+CO88rnTHwHxQAECASANBwIBIAsIAgFqCgkA/bVszQX8ILdHHXaiaBBrpOEAxwz6Ammf6YBqAPw1aZ+A/DXov/ww/DN8MfwxRwp6AvwxOHwxuHwzZGT8NTh8Nb/8MPFvfCNJeRnJuPwzcXwAaZ/qaJD8NZB8NS38IWR6AHwh54Wf/CNnhYBkfCVninwl54Wf54vk9qo4NT/8M8AB47VQlGJ8ILdHDnaiaHoCaZ/pgGoA/DVpn4D8Nei//DD8M3wx/DFvaPwl5EEIOVCUYkEIQAAAAFjnhY+Q54Wf5DnnhYD8FGeLOWegfBLnhZ/AEOegEGeakWeY3ks456AQ54vKuOegkObxEGS4/YAtkOB/wAwB4bvEGBivhBbo4c7UTQ9ATTP9MA1AH4atM/Afhr0X/4Yfhm+GP4Yt7R+ErIghBcQYGKghCAAAAAsc8LHyHPFMhzzwsB+CjPFnLPQPglzws/gCHPQCDPNSLPMbyWcc9AIc8XlXHPQSHN4iDJcfsAWyHA/4DABMjh74Qsj0APhDzws/+EbPCwDI+ErPFPhLzws/zxfJ7VTecWp/+GcCASAQDgHxuyWkS7+EFujhztRND0BNM/0wDUAfhq0z8B+GvRf/hh+Gb4Y/hi3vhFIG6SMHDecPhCgED0DvKK1wv/uvLgZPpA0fgnbxD4S7zy4GX4ACD4S3CBAIDIcc8LASLPCgBzz0AkzxYj+gKAac9Acs9AIMki+wBfBTAgwP+A8ATI4e+ELI9AD4Q88LP/hGzwsAyPhKzxT4S88LP88Xye1U3nBqf/hnAGrdcCHHAJ0i0HPXIdcLAMABkJDi4CHXDR+Q4VMRwACQ4MEDIoIQ/////byxkOAB8AH4R26Q3g==",
    "data": "te6ccgEBBgEAUQACIYAAALh7SgP1wAAAAAAAAB7gAgEAElNvbWUgZ29hbAIDzyAFAwEB3gQAA9AgAEHZ0ZOZnpDm7EEuLYDU+ayCbh0T9ip8ddv4s9oarwEfRAQ="
  }"#;

const KEYS: &str = r"f65b410f0c7e17850807749b17e11bb0754bcbdd15e399ccec49116b11326d453a327333d21cdd8825c5b01a9f35904dc3a27ec54f8ebb7f167b4355e023e880";

#[test]
fn test_local_piggy_call() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let messages = contract.local_call_tvm_json(
        "getTargetAmount".to_owned(),
        None,
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        None).expect("Error calling contract");
    println!("messages count {}", messages.len());
    assert!(messages.len() == 1);

    let answer = crate::Contract::decode_function_response_json(
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        "getTargetAmount".to_owned(),
        messages[0].body().expect("Message has no body"),
        false)
        .expect("Error decoding result");

    assert_eq!(answer, r#"{"value0":"0x7b"}"#);
}

#[test]
fn test_local_call_accept_error() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let result = contract.local_call_json(
        "getGoal".to_owned(),
        None,
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        None,
        Default::default());
    assert!(result.is_err());
}

#[test]
fn test_executor_call() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let keypair = ed25519_dalek::Keypair::from_bytes(&hex::decode(KEYS).unwrap()).unwrap();

    let result = contract.local_call_json(
        "transfer".to_owned(),
        None,
        "{\"to\": \"0:e6392da8a96f648098f818501f0211f27c89675e5f196445d211947b48e7c85b\"}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        Some(&keypair),
        Default::default()).expect("Error calling contract");
    assert!(result.transaction.out_messages.len() == 1);
    assert!(!result.transaction.aborted);

    let fees = result.transaction.calc_fees();

    //println!("{:?}", fees);

    assert_eq!(fees.in_msg_fwd_fee, 2008000);
    assert_eq!(fees.gas_fee, 7256000);
    assert_eq!(fees.out_msgs_fwd_fee, 1000000);
    assert!(fees.total_account_fees > 10264122);
    assert!(fees.storage_fee > 122);
}

#[test]
fn test_stack_from_json() {
    fn test_json(json: &str, expected: Vec<StackItem>) {
        let array: Vec<Value> = serde_json::from_str(json).unwrap();
        let items = crate::contract::StackItemJSON::items_from_json_array(array.iter()).unwrap();
        assert_eq!(items, expected);
    }
    fn int_item(data: IntegerData) -> StackItem {
        StackItem::Integer(Arc::new(data))
    }
    fn i64_item(v: i64) -> StackItem {
        int_item(IntegerData::from_i64(v))
    }
    test_json("[]", vec![]);
    test_json("[null]", vec![StackItem::None]);
    test_json("[false]", vec![int_item(IntegerData::zero())]);
    test_json("[true]", vec![int_item(IntegerData::one())]);
    test_json(r#"["NaN"]"#, vec![int_item(IntegerData::nan())]);
    test_json(r#"[11]"#, vec![i64_item(11)]);
    test_json(r#"["12"]"#, vec![i64_item(12)]);
    test_json(r#"["0x13"]"#, vec![i64_item(0x13)]);
    test_json(r#"["0X14"]"#, vec![i64_item(0x14)]);
    test_json(r#"[-15]"#, vec![i64_item(-15)]);
    test_json(r#"["-16"]"#, vec![i64_item(-16)]);
    test_json(r#"["-0x17"]"#, vec![i64_item(-0x17)]);
    test_json(r#"["-0X18"]"#, vec![i64_item(-0x18)]);
    test_json(r#"["0x123456789abcDEF"]"#, vec![i64_item(0x123456789abcdef)]);
    test_json(r#"[1, [2, 3, 4]]"#, vec![
        i64_item(1),
        StackItem::Tuple(vec![
            i64_item(2),
            i64_item(3),
            i64_item(4),
        ])
    ]);

    fn test_stack(expected_json: &str, stack: Vec<StackItem>) {
        let json = crate::contract::StackItemJSON::json_array_from_items(stack.iter()).unwrap();
        assert_eq!(json.to_string(), expected_json);
    }
    test_stack("[]", vec![]);
    test_stack("[null]", vec![StackItem::None]);
    test_stack(r#"["0x0"]"#, vec![int_item(IntegerData::zero())]);
    test_stack(r#"["0x1"]"#, vec![int_item(IntegerData::one())]);
    test_stack(r#"["NaN"]"#, vec![int_item(IntegerData::nan())]);
    test_stack(r#"["0xb"]"#, vec![i64_item(11)]);
    test_stack(r#"["0xc"]"#, vec![i64_item(12)]);
    test_stack(r#"["0x13"]"#, vec![i64_item(0x13)]);
    test_stack(r#"["0x14"]"#, vec![i64_item(0x14)]);
    test_stack(r#"["-0xf"]"#, vec![i64_item(-15)]);
    test_stack(r#"["-0x10"]"#, vec![i64_item(-16)]);
    test_stack(r#"["-0x17"]"#, vec![i64_item(-0x17)]);
    test_stack(r#"["-0x18"]"#, vec![i64_item(-0x18)]);
    test_stack(r#"["0x123456789abcdef"]"#, vec![i64_item(0x123456789abcdef)]);
    test_stack(r#"["0x1",["0x2","0x3","0x4"]]"#, vec![
        int_item(IntegerData::from_i32(1)),
        StackItem::Tuple(vec![
            int_item(IntegerData::from_i32(2)),
            int_item(IntegerData::from_u64(3)),
            int_item(IntegerData::from_i128(4)),
        ])
    ]);
}

