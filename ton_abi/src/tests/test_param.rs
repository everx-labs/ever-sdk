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

use serde_json;
use {Param, ParamType};

#[test]
fn test_simple_param_deserialization() {
    let s = r#"{
        "name": "a",
        "type": "int9"
    }"#;

    let deserialized: Param = serde_json::from_str(s).unwrap();

    assert_eq!(deserialized, Param {
        name: "a".to_owned(),
        kind: ParamType::Int(9),
    });
}

#[test]
fn test_tuple_param_deserialization() {
    let s = r#"{
        "name": "a",
        "type": "tuple",
        "components" : [
            {
                "name" : "a",
                "type" : "int8"
            },
            {
                "name" : "b",
                "type" : "int8"
            }
        ]
    }"#;

    let deserialized: Param = serde_json::from_str(s).unwrap();

    assert_eq!(deserialized, Param {
        name: "a".to_owned(),
        kind: ParamType::Tuple(vec![
            Param { name: "a".to_owned(), kind: ParamType::Int(8) },
            Param { name: "b".to_owned(), kind: ParamType::Int(8) },
        ]),
    });
}

#[test]
fn test_tuples_array_deserialization() {
    let s = r#"{
        "name": "a",
        "type": "tuple[]",
        "components" : [
            {
                "name" : "a",
                "type" : "bool"
            },
            {
                "name" : "b",
                "type" : "tuple[5]",
                "components" : [
                    {
                        "name" : "a",
                        "type" : "uint8"
                    },
                    {
                        "name" : "b",
                        "type" : "int15"
                    }
                ]
            }
        ]
    }"#;

    let deserialized: Param = serde_json::from_str(s).unwrap();

    assert_eq!(deserialized, Param {
        name: "a".to_owned(),
        kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
            Param { 
                name: "a".to_owned(),
                kind: ParamType::Bool
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::FixedArray(
                    Box::new(ParamType::Tuple(vec![
                        Param { name: "a".to_owned(), kind: ParamType::Uint(8) },
                        Param { name: "b".to_owned(), kind: ParamType::Int(15) },
                    ])),
                    5
                )
            },
        ]))),
    });
}