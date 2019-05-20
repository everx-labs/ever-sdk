use {Contract, Function, Param, ParamType};
use std::collections::HashMap;

const TEST_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
            "name": "input_and_output",
            "inputs": [
                {"name": "a","type": "uint64"},
                {"name": "b","type": "uint8[]"},
                {"name": "c","type": "bitstring"}
            ],
            "outputs": [
                {"name": "a","type": "dint"},
                {"name": "b","type": "bits8"}
            ]
        }, {
            "name": "no_output",
            "inputs": [{"name": "a", "type": "uint15"}],
            "outputs": []
        }, {
            "name": "no_input",
            "inputs": [],
            "outputs": [{"name": "a", "type": "duint"}]
        }, {
            "name": "constructor",
            "inputs": [],
            "outputs": [],
            "signed": false
        }, {
            "name": "signed",
            "inputs": [{"name": "a", "type": "bool"}],
            "outputs": [],
            "signed": true
        }]
}"#;

#[test]
fn test_abi_parse() {
    let parsed_contract = Contract::load(TEST_ABI.as_bytes()).unwrap();

    let mut functions = HashMap::new();

    functions.insert(
        "input_and_output".to_owned(),
        Function {
                name: "input_and_output".to_owned(),
                inputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Uint(64) },
                    Param { name: "b".to_owned(), kind: ParamType::Array(Box::new(ParamType::Uint(8))) },
                    Param { name: "c".to_owned(), kind: ParamType::Bitstring },
                ],
                outputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Dint },
                    Param { name: "b".to_owned(), kind: ParamType::Bits(8) },
                ],
                signed: false
        });

    functions.insert(
        "no_output".to_owned(),
        Function {
                name: "no_output".to_owned(),
                inputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Uint(15) },
                ],
                outputs: vec![],
                signed: false
        });

    functions.insert(
        "no_input".to_owned(),
        Function {
                name: "no_input".to_owned(),
                inputs: vec![],
                outputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Duint },
                ],
                signed: false
        });

    functions.insert(
        "constructor".to_owned(),
        Function {
                name: "constructor".to_owned(),
                inputs: vec![],
                outputs: vec![],
                signed: false
        });

    functions.insert(
        "signed".to_owned(),
        Function {
                name: "signed".to_owned(),
                inputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Bool },
                ],
                outputs: vec![],
                signed: true
        });

    let expected_contract = Contract { functions };

    assert_eq!(parsed_contract, expected_contract);
}

#[test]
fn print_function_singnatures() {
    let contract = Contract::load(TEST_ABI.as_bytes()).unwrap();

    let functions = contract.functions();

    for function in functions {
        //println!("{}", function.name);
        println!("{}", function.get_function_signature());
        let id = u32::from_be_bytes(function.get_function_id());
        println!("{:X?}\n", id);
    }
}

const TEST_ABI_WRONG_VERSION: &str = r#"
{
    "ABI version": 1,
    "functions": [{
            "name": "constructor",
            "inputs": [],
            "outputs": [],
            "signed": false
        }]
}"#;

#[test]
fn test_abi_wrong_version() {
    assert!(Contract::load(TEST_ABI_WRONG_VERSION.as_bytes()).is_err());
}