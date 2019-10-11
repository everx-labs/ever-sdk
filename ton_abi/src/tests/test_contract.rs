use {Contract, Function, Param, ParamType};
use std::collections::HashMap;

const TEST_ABI: &str = r#"
{
    "ABI version": 1,
    "functions": [{
            "name": "input_and_output",
            "inputs": [
                {"name": "a","type": "uint64"},
                {"name": "b","type": "uint8[]"},
                {"name": "c","type": "bytes"}
            ],
            "outputs": [
                {"name": "a","type": "int16"},
                {"name": "b","type": "uint8"}
            ]
        }, {
            "name": "no_output",
            "inputs": [{"name": "a", "type": "uint15"}],
            "outputs": []
        }, {
            "name": "no_input",
            "inputs": [],
            "outputs": [{"name": "a", "type": "uint8"}]
        }, {
            "name": "constructor",
            "inputs": [],
            "outputs": []
        }, {
            "name": "signed",
            "inputs": [{"name": "a", "type": "bool"}],
            "outputs": []
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
                    Param { name: "b".to_owned(), kind: ParamType::Array(
                        Box::new(ParamType::Uint(8))) },
                    Param { name: "c".to_owned(), kind: ParamType::Bytes },
                ],
                outputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Int(16) },
                    Param { name: "b".to_owned(), kind: ParamType::Uint(8) },
                ],
                set_time: true,
                id: Function::calc_function_id("input_and_output(time,uint64,uint8[],bytes)(int16,uint8)v1")
        });

    functions.insert(
        "no_output".to_owned(),
        Function {
                name: "no_output".to_owned(),
                inputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Uint(15) },
                ],
                outputs: vec![],
                set_time: true,
                id: Function::calc_function_id("no_output(time,uint15)()v1")
        });

    functions.insert(
        "no_input".to_owned(),
        Function {
                name: "no_input".to_owned(),
                inputs: vec![],
                outputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Uint(8) },
                ],
                set_time: true,
                id: Function::calc_function_id("no_input(time)(uint8)v1")
        });

    functions.insert(
        "constructor".to_owned(),
        Function {
                name: "constructor".to_owned(),
                inputs: vec![],
                outputs: vec![],
                set_time: true,
                id: Function::calc_function_id("constructor(time)()v1")
        });

    functions.insert(
        "signed".to_owned(),
        Function {
                name: "signed".to_owned(),
                inputs: vec![
                    Param { name: "a".to_owned(), kind: ParamType::Bool },
                ],
                outputs: vec![],
                set_time: true,
                id: Function::calc_function_id("signed(time,bool)()v1")
        });

    let expected_contract = Contract { functions, events: HashMap::new() };

    assert_eq!(parsed_contract, expected_contract);
}

#[test]
fn print_function_singnatures() {
    let contract = Contract::load(TEST_ABI.as_bytes()).unwrap();

    println!("Functions\n");

    let functions = contract.functions();

    for (_, function) in functions {
        //println!("{}", function.name);
        println!("{}", function.get_function_signature());
        let id = function.get_function_id();
        println!("{:X?}\n", id);
    }

    println!("Events\n");

    let events = contract.events();

    for (_, event) in events {
        //println!("{}", function.name);
        println!("{}", event.get_function_signature());
        let id = event.get_function_id();
        println!("{:X?}\n", id);
    }
}

const TEST_ABI_WRONG_VERSION: &str = r#"
{
    "ABI version": 0,
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
