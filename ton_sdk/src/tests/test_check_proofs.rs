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

const CORRECT_TRANSACTIONS: [&str; 7] = [
    r#"
    {
        "id" : "4007b5902ae24022e294a41a1d6656521bbb3d997ba2697f8a6521e1564e3214",
        "end_status" : 1,
        "aborted" : true,
        "account_addr" : "0:34517c7bdf5187c55af4f8b61fdc321588c7ab768dee24b006df29106458d7cf",
        "total_fees" : "010",
        "lt" : "b103b2f60fc3",
        "storage" : {
          "status_change" : 0,
          "storage_fees_collected" : "010"
        },
        "prev_trans_lt" : "b103b2f60fc1",
        "prev_trans_hash" : "edea66f0fc61f8241fd3a965332bafc9d233d3f99963a9dde269a06c0d71d097",
        "block_id" : "e18c63776bf81e3ebb6d143cd3ba42b8ddabeeba7072db89a3c7ab0c17cda3ec",
        "compute" : {
          "mode" : 0,
          "vm_init_state_hash" : "0000000000000000000000000000000000000000000000000000000000000000",
          "gas_limit" : 10000000,
          "vm_final_state_hash" : "0000000000000000000000000000000000000000000000000000000000000000",
          "success" : true,
          "vm_steps" : 48,
          "gas_used" : 4909,
          "exit_code" : 0,
          "msg_state_used" : false,
          "compute_type" : 1,
          "account_activated" : false,
          "gas_fees" : "010"
        },
        "destroyed" : false,
        "old_hash" : "376b56a0174f52c35aaed2bac352c646d88cdf84b5a0f9fbce8225e83e29b71e",
        "orig_status" : 1,
        "tr_type" : 3,
        "boc" : "te6ccgECBgEAATIAA69zRRfHvfUYfFWvT4th/cMhWIx6t2je4ksAbfKRBkWNfPAAABA7L2D8Pt6mbw/GH4JB/TqWUzK6/J0jPT+Zljqd3iaaBsDXHQlwAAAQOy9g/BXfdILQABQIBQQBAgUwMDQDAgBpYAAAAJYAAAAEAAYAAAAAAAUZroTxe4+LIgJql1/1Xxqxn95KdodE0heN+mO7Uz4QekCQJrwAnkJlrmJaAAAAAAAAAAAAMAAAAAAAAAAA...",
        "now" : 1576486957,
        "out_msgs" : [ ],
        "action" : {
          "tot_msg_size_bits" : 1239,
          "spec_actions" : 0,
          "status_change" : 0,
          "skipped_actions" : 0,
          "action_list_hash" : "8cd74278bdc7c59101354baffaaf8d58cfef253b43a2690bc6fd31dda99f083d",
          "tot_msg_size_cells" : 2,
          "valid" : true,
          "msgs_created" : 2,
          "no_funds" : true,
          "success" : false,
          "result_code" : 37,
          "result_arg" : 2,
          "tot_actions" : 3
        },
        "proof" : "te6ccgECJQEABmIACUYD4Yxjd2v4Hj67bRQ807pCuN2r7rpwctuJo8erDBfNo+wAEwEkEBHvVar///8RIyIhAiSJSjP2/Xzg/XgH5gmmnltiSVUTAOugjvH3nvqag+iXZdfsUwItwpBAOC1uaapbyFmgmhn0eLEi+W8n190i1KIYVJfvuqTAIB8EAyhIAQFGm8vuuh1q4NeKJ/rZ0pdM0SnnFfs95FrnawWsWhfrJgAFIQGCBSIDQEAKBiKXv5VV...",
        "outmsg_cnt" : 0,
        "status" : 3,
        "new_hash" : "e18a6010ee7986ec14dd0075419aef03520a70ea4e390bf1139921cb70a1e597"
      },
    "#,

];

#[test]
fn test_check_transaction() {

    for tr in CORRECT_TRANSACTIONS.iter() {
        let tr_val = serde_json::from_str(tr).unwrap();
        check_transaction(&tr_val).unwrap();
    }
}

const CORRECT_MESSAGES: [&str; 9] = [
    r#"
    {
        "id" : "de106f9eb1a7cbd2f82ec115ae7fa39f18ca3718be1bf01885505e133ac06638",
        "transaction_id" : "4716b9f26c016ce9f52606ff6f25d4d0e5ca28aec766a58fe1cdb2e34e59d84d",
        "fwd_fee" : "010",
        "ihr_fee" : "010",
        "dst" : "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "src" : "-1:0000000000000000000000000000000000000000000000000000000000000000",
        "bounce" : true,
        "created_at" : 1576487125,
        "bounced" : false,
        "ihr_disabled" : true,
        "block_id" : "f64b17714f593425d24aad7bc8ba14b2b5a08906746f7017df5352f4e8a124dd",
        "boc" : "te6ccgEBAQEAWAAAq2n+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE/zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzSv1W2AAAAAAIHbkRagLvukapA",
        "msg_type" : 0,
        "proof" : "te6ccgECHgEABCEACUYD9ksXcU9ZNCXSSq17yLoUsrWgiQZ0b3AX31NS9OihJN0AFQEkEBHvVar///8RHBsaAiSJSjP2/RLzrjnrb5f5lZE02/pXqRCNhuezbdhSpROEnwKInS/MvdiPrSO+ePxeiwSr0fQcgWxfEL9gcjVavRr76vn6QDjAEhEIAyMXzKVolQL5AESoF8gEBwYEIQFQBSIBYRgUKEgBAfvPOsfc3j/L1kwoTnmX36frfYhTrX2W...",
        "created_lt" : "b103b7222d40",
        "value" : "08afd56d80",
        "status" : 5
      }
    "#,

];

#[test]
fn test_check_messages() {

    for msg in CORRECT_MESSAGES.iter() {
        let msg_val = serde_json::from_str(msg).unwrap();
        check_message(&msg_val).unwrap();
    }
}

const CORRECT_ACCOUNTS: [&str; 6] = [
    r#"
    {
        "id" : "0:c8c910b3b62674a79565c52664941082c09faca8466109085415ccbe94eb63b9",
        "acc_type" : 0,
        "balance" : "097aef409fe",
        "boc" : "te6ccuEBAQEAOQByAG3ADIyRCztiZ0p5VlxSZklBCCwJ+sqEZhCQhUFcy+lOtjuSAlvC7qqeeAAAAKN6zicJQeu9An+EI1sevg==",
        "last_paid" : 1574261711,
        "proof" : "te6ccgECIwEABDkACUYDtp4CWQYl22sgDF4b3UDrtvxx7sNputv7m1bdP7ZCzzsAJgEjW5Ajr+L///8RAgAAAADAAAAAAAAAAAAQP2AAAAAAXfiT4QAAAQwP9cPBAAxdtyACAwQoSAEB2KCBYBJ8FUn979WAWnotgJM4LJ20Es3j6DLpOCsHxkIAASERgcBujs+JBV8QBQDVAAAAAAAAAAD//////////3Abo7PiQVfDeScB0v7ikAAAEMD9c/RA...",
        "last_trans_lt" : "a28deb389c2"
      }
    "#,

];

#[test]
fn test_check_accounts() {

    for acc in CORRECT_ACCOUNTS.iter() {
        let acc_val = serde_json::from_str(acc).unwrap();
        check_account(&acc_val).unwrap();
    }
}