/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

#[cfg(feature = "fee_calculation")]
#[test]
fn test_local_call_accept_error() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let result = contract.local_call_json(
        "getGoal".to_owned(),
        None,
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        None);
    assert!(result.is_err());
}

#[cfg(feature = "fee_calculation")]
#[test]
fn test_executor_call() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let keypair = ed25519_dalek::Keypair::from_bytes(&hex::decode(KEYS).unwrap()).unwrap();

    let result = contract.local_call_json(
        "transfer".to_owned(),
        None,
        "{\"to\": \"0:e6392da8a96f648098f818501f0211f27c89675e5f196445d211947b48e7c85b\"}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        Some(&keypair)).expect("Error calling contract");
    assert!(result.messages.len() == 1);

    //println!("{:?}", result.fees);

    assert_eq!(result.fees.in_msg_fwd_fee, 2008000);
    assert_eq!(result.fees.gas_fee, 7256000);
    assert_eq!(result.fees.out_msgs_fwd_fee, 1000000);
    assert!(result.fees.total_account_fees > 10264122);
    assert!(result.fees.storage_fee > 122);
}

#[test]
fn test_stack_from_json() {
    fn test_json(json: &str, expected: Vec<StackItem>) {
        let array: Vec<Value> = serde_json::from_str(json).unwrap();
        let items = crate::contract::StackItemJSON::items_from_json_array(array.iter()).unwrap();
        assert_eq!(items, expected);
    }
    fn i64(v: i64) -> StackItem {
        StackItem::Integer(Arc::new(IntegerData::from_i64(v)))
    }
    test_json("[]", vec![]);
    test_json("[null]", vec![StackItem::None]);
    test_json("[false]", vec![StackItem::Integer(Arc::new(IntegerData::zero()))]);
    test_json("[true]", vec![StackItem::Integer(Arc::new(IntegerData::one()))]);
    test_json(r#"["NaN"]"#, vec![StackItem::Integer(Arc::new(IntegerData::nan()))]);
    test_json(r#"[11]"#, vec![i64(11)]);
    test_json(r#"["12"]"#, vec![i64(12)]);

    test_json(
        r#"["0x13"]"#,
        vec![
            i64(0x13)
        ]);

    test_json(
        r#"["0X14"]"#,
        vec![
            i64(0x14),
        ]);

    test_json(
        r#"[-15]"#,
        vec![
            i64(-15),
        ]);

    test_json(
        r#"["-16"]"#,
        vec![
            i64(-16),
        ]);

    test_json(r#"["-0x17"]"#, vec![i64(-0x17)]);

    test_json(r#"["-0X18"]"#, vec![i64(-0x18)]);

    test_json(r#"["0x123456789abcDEF"]"#, vec![i64(0x123456789abcdef)]);

    fn test_stack(stack: Vec<StackItem>, expected: &str) {
        let json = crate::contract::StackItemJSON::json_array_from_items(stack.iter()).unwrap();
        assert_eq!(json.to_string(), expected);
    }
    test_stack(vec![], "[]");
}

