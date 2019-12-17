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

// TODO add more cases
const CORRECT_TRANSACTIONS: [&str; 1] = [
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
        "boc" : "te6ccgECBgEAATIAA69zRRfHvfUYfFWvT4th/cMhWIx6t2je4ksAbfKRBkWNfPAAABA7L2D8Pt6mbw/GH4JB/TqWUzK6/J0jPT+Zljqd3iaaBsDXHQlwAAAQOy9g/BXfdILQABQIBQQBAgUwMDQDAgBpYAAAAJYAAAAEAAYAAAAAAAUZroTxe4+LIgJql1/1Xxqxn95KdodE0heN+mO7Uz4QekCQJrwAnkJlrmJaAAAAAAAAAAAAMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgnI3a1agF09Sw1qu0rrDUsZG2IzfhLWg+fvOgiXoPim3HuGKYBDueYbsFN0AdUGa7wNSCnDqTjkL8ROZIctwoeWXAAEg",
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
        "proof" : "te6ccgECJQEABmIACUYD4Yxjd2v4Hj67bRQ807pCuN2r7rpwctuJo8erDBfNo+wAEwEkEBHvVar///8RIyIhAiSJSjP2/Xzg/XgH5gmmnltiSVUTAOugjvH3nvqag+iXZdfsUwItwpBAOC1uaapbyFmgmhn0eLEi+W8n190i1KIYVJfvuqTAIB8EAyhIAQFGm8vuuh1q4NeKJ/rZ0pdM0SnnFfs95FrnawWsWhfrJgAFIQGCBSIDQEAKBiKXv5VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVAqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0AAAAEDsvYPwwQcJI691VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVAAABA7L2D8MrsmacBucEj2zjBxx+B+/6pgukIATfKMRdC4jAEiaCPAAAAQOy14tDXfdILQABQIHgkIKEgBAWdDF4hIEana5g0jYLq3irIbpDshjLyaCnlnzjq5J1q4AAEoSAEBv6tDrdP8VfLSeYGPI7Qk5PIStqnQFk9UdvTXbHfRBsYAACIDdgQXCyOXvoUXx731GHxVr0+LYf3DIViMerdo3uJLAG3ykQZFjXzwKaKL4976jD4q16fFsP7hkKxGPVu0b3ElgDb5SIMixr58+AAABA7L2D8AQBENDChIAQGWkeEBp9zVKcnJpAzUxaxEokKBR742/r+x/ITGceut/QAAAQNQQA4Dr3NFF8e99Rh8Va9Pi2H9wyFYjHq3aN7iSwBt8pEGRY188AAAEDsvYPw+3qZvD8YfgkH9OpZTMrr8nSM9P5mWOp3eJpoGwNcdCXAAABA7L2D8Fd90gtAAFAgeEA8CBTAwNBUUAIJyN2tWoBdPUsNartK6w1LGRtiM34S1oPn7zoIl6D4ptx7himAQ7nmG7BTdAHVBmu8DUgpw6k45C/ETmSHLcKHllyEDUEASI69zRRfHvfUYfFWvT4th/cMhWIx6t2je4ksAbfKRBkWNfPAAABA7L2D8HHvTvFtqYCmtg+CMlvAcddl+ld4I5DTBEtZMYF7mckGwAAAQOy14tDXfdILQABQIHhYTAgUgMDQVFABpYAAAAJYAAAAEAAYAAAAAAAUZroTxe4+LIgJql1/1Xxqxn95KdodE0heN+mO7Uz4QekCQJrwAnkJlrmJaAAAAAAAAAAAAMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAoSAEBHo2ZBnfiVjCLIP+6I42KtxJ6lgMekMdQLxjnKdaWaRIAACOXvrMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMwKZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmc+AAABA7L2D8AQBoZGChIAQGFOPJqloLYv2TsiiemrZ8v2MYKxQILMhH2+TywpLh7FgAAKEgBARtIoeFL2H+SFQuHGsxdU1nSSpEELn4KkI9wuwtj9A20AAMhA1BAGyOvczMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMwAAAQOy9g/B7y1k7Flhff2kV2saVDW65TwsgD5XXJftWzpZ+LSzZSoAAAEDsteLQl33SC0AAUCB4dHChIAQHPKSUsGzc4kdthRNjSaVraU/CGyzZdCAhXZ3bn+/TkWwABKEgBATHbTb+bsbmvAZgyvq4uAMqD99IxPh5W3FtkFFItgUrmAAAAASAoSAEBrksygOVuL6+D9BSm49q+nV++GJdlRMBf7RIazLhbU/wAAChIAQE7vKV+JTYeRWFECe4Ny4hIZNrFvodX/oHGvHeuMXZ4xAAEKEgBARjgXfZJHi+4jyWz5ZXCUgIWd6FZCDT4oDkXNGtE8rQuABIoSAEBG47ENWsGgKQYffjDnv7QnMtDH1xSaQFVDy2dAWL3+GEAAwGGm8ephwAAAAAEAAAL/sQAAAAAAP////8AAAAAAAAAAF33SC0AAAEDsvYPwAAAAQOy9g/EBjdd/QAAKewAC/7BAAvtICQAmAAAAQOy14tEAAv+w/L+UPaUWZYDYBBJZfpvqB4I21F6Tvs+plrz6Gmaez5ieFzl5GOtv7kpVZl8K0PMCEwHIBEDQ68nMp5Zqzvn0DU=",
        "outmsg_cnt" : 0,
        "status" : 3,
        "new_hash" : "e18a6010ee7986ec14dd0075419aef03520a70ea4e390bf1139921cb70a1e597"
      }
    "#,

];

#[test]
fn test_check_transaction() {

    for tr in CORRECT_TRANSACTIONS.iter() {
        let tr_val = serde_json::from_str(tr).unwrap();
        check_transaction(&tr_val).unwrap();
    }
}

// TODO add more cases
const CORRECT_MESSAGES: [&str; 1] = [
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
        "proof" : "te6ccgECHgEABCEACUYD9ksXcU9ZNCXSSq17yLoUsrWgiQZ0b3AX31NS9OihJN0AFQEkEBHvVar///8RHBsaAiSJSjP2/RLzrjnrb5f5lZE02/pXqRCNhuezbdhSpROEnwKInS/MvdiPrSO+ePxeiwSr0fQcgWxfEL9gcjVavRr76vn6QDjAEhEIAyMXzKVolQL5AESoF8gEBwYEIQFQBSIBYRgUKEgBAfvPOsfc3j/L1kwoTnmX36frfYhTrX2W8HmedE9yBxlRAAIoSAEBRLiN6VOYnY8d1fwG/IjacxwvymV2AD4P/QjN6CswBJ4AAyEBggkiA0BACwooSAEBsMOkFgsnjaBaSsEt4In6v78zRSExcp3p4edeo9OqiOcAAyIDdgQNDChIAQHIykbGIvoUzFsqO+aK3c9NpQ/o+Lu/6K4xZosYDkSpvQAEI5e+szMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzApmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZz4AAAEDtyItQBAEA8OKEgBAdBHCIdeBUZfzumTEhAWIkayLNOQG11F8FEJMTDvOXSlAAAhA0BAFChIAQEBUYROXcjZ3nYXS62lV+cIUZPkiEWpEI1ttEba+EazbwADKEgBAa5LMoDlbi+vg/QUpuPavp1fvhiXZUTAX+0SGsy4W1P8AAAhA4AgEyJHoBvCDfPWNPl6XwXYIrXP9HPjGUbjF8N+AxCqC8JnWAzHAAYQGBQjr3MzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMAAAEDtyItQsTy/M3dnaOxUapxks6CDIKlA05ebboqMQWMln3C9GJEAAABA7ciLUFd90jVAAFAgXFhUoSAEBj2p3Kn5hzguxuTYkPO6wlCSE8071+o8lY1OyfrOQdpwAAShIAQFE4Sp2Hp8vTPNDj3MUIciFDpNogwbkCYJVi/e3MvdbVQAAAQGgGQEGRgYAGQCraf4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT/MzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzNK/VbYAAAAAAgduRFqAu+6RqkAoSAEBTWXgo1rwzL6ff1PrcpJF/hcp2CLrXYaKSYarCkzRcEIAFChIAQFCmzsxU7AnyS8Ky64WvehU1JD6Q+MHAmXSU+bo4/u8AQADAYabx6mHAAAAAAQAAAv+9AAAAAAA/////wAAAAAAAAAAXfdI1QAAAQO3Ii1AAAABA7ciLUTMzm4DAAAp7QAL/vEAC+0gHQCYAAABA7cDqMQAC/7zz41IzMO0R5eUYKWlImbaDUhE7y/k0dpKO2GEl4FRO5pcqlRzOKYgbHLxIgS8LIaRD4UE4oYzEoqQw2bbR3m5NQ==",
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

// TODO add more cases
const CORRECT_ACCOUNTS: [&str; 1] = [
    r#"
    {
        "id": "0:c8c910b3b62674a79565c52664941082c09faca8466109085415ccbe94eb63b9",
        "acc_type": 0,
        "balance": "09c1b70b2d5",
        "boc": "te6ccuEBAQEAOQByAG3ADIyRCztiZ0p5VlxSZklBCCwJ+sqEZhCQhUFcy+lOtjuSAlvC78gPYAAABDtQ99IRQwbcLLVEAacP6A==",
        "proof": "te6ccgECIwEABDoACUYDfpBcTyTjlv2fHcZvGGW8Zk4NkhT+Z5FMEIESUgEH0PEAJgEjW5Ajr+L///8RAgAAAADAAAAAAAAAAAAQebQAAAAAXfkq/gAAAQ/ZAyhDAAyInCACAwQoSAEBLgiGV5xaiyBZAJl0I1n72C84u10gXww2XYET4MS4zQgABCERgcBtbB0sM+qQBQDVAAAAAAAAAAD//////////3AbWwdLDPqjetspmT30EAAAEP2PPmBAAMiJxKWlsXqU6F9v4hdxdKYowOtJe6E3mqoolAsFeQLJszSwNpSlxELS97L92ESOEDYobOwSDi9fFC6QqNj5oUraEZgiEmwOA2tg6WGfVAYHIg8A2bKhWI8sSAgJKEgBARtPT8vERwdukAwHwki8eUwaY76MaefBPYVGj+xXvqeJACIiDwDK/klXKaIICgsoSAEB3E8Edgtxoo7ieQhrf5qwR6qQUvz4taIeTNVwlULnGroAIihIAQGfqGgXha/KBjvBNHQFsR5S0Mgi7MQcBqhpX2BfrYzvpwAcIg8Aw9EOZevS6AwNIg8AwfEsBIOeqA4PKEgBAdopCGv1gJFtl23S0nEsLcaIwZk5LAqMJrBCrXw7sxmZABwiDwDBANbcJL6IEBEoSAEBcoL8wAZzJ+K3LetKoFBvoEpKSdhqvS6oCfMHolNxJTgAFyIPAMCMv3htwMgSEyhIAQExMpYheMq5m/HWNeMkqBvieRK7l75+A08MLj7mpgn5SwASKEgBAd6nmkEkTUCG+RAPUYsfh2yGpekhjQCebIKD5xUBE3x4ABciDwDAOYPwGW8oFBUoSAEBQul6TAjeQV287lB56EM6K+7jHn8nloMLqPS6GT4YwGAAGSIPAMAgE4AK80gWFyINAKwwwA9HaBgZKEgBAdpgzvZSa+twohtk9/rBpC8oNqPOIWy0Xre+WB3gtgrOAAgiDQCk0I+OSMgaGyhIAQHt2FHCRFBQv78k4HtcAjoE+1h3ybw5JvjYfKWGDu0uqQARKEgBAWYs2/gL8Qtl6R4MnlzVZEnLfq88YhfN3Av15e5HreRwAAMiDQCi7D+15IgcHSINQCiZiNvZMh4fKEgBARmBziU9CStNi6WLJwvVD2novrOSOKijRqQVHWHcGwzfAAEoSAEBh/X663QBFWoccBLGATZ7cUwm2qFbBN4QVJwdUZdDercADiIPeECiYvg8ZuggIQGZusWdsTOlPKsuKTMkoIQWBP1lQjMISEKgrmX0p1sdyBQwbcLLVcEUNOG9A1JbJuvdu+frprHL3jAHxT/BDoK/hwHkKnJYAAACHah76QciKEgBAa4UQVBSskHt9D+ndXj2GLiVe4+IUgc0B6Q/wjp/+T5zAAIAbcAMjJELO2JnSnlWXFJmSUEILAn6yoRmEJCFQVzL6U62O5ICW8LvyA9gAAAEO1D30hFDBtwstUQ=",
        "last_paid": 1576600044,
        "last_trans_lt": "b10ed43df484"
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