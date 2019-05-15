
use crate::{Token, Param, ParamType, Uint, Int};
use super::{Tokenizer, Detokenizer};
use num_bigint::{BigInt, BigUint};
use tvm::bitstring::Bitstring;

#[test]
fn test_tokenize_ints() {
    let input = r#"{
        "a" : 123,
        "b" : -456,
        "c" : "0xabcdef",
        "d" : "-0xABCDEF",
        "e" : "789",
        "f" : "-12345678900987654321"
    }"#;

    let params = vec![
        Param {name: "a".to_owned(), kind: ParamType::Uint(8)},
        Param {name: "b".to_owned(), kind: ParamType::Int(16)},
        Param {name: "c".to_owned(), kind: ParamType::Duint},
        Param {name: "d".to_owned(), kind: ParamType::Dint},
        Param {name: "e".to_owned(), kind: ParamType::Uint(13)},
        Param {name: "f".to_owned(), kind: ParamType::Int(128)},
    ];

    let expected_tokens = vec![
        Token::Uint(Uint{number: BigUint::from(123u8), size: 8}),
        Token::Int(Int{number: BigInt::from(-456i16), size: 16}),
        Token::Duint(BigUint::from(0xABCDEFu32).into()),
        Token::Dint(BigInt::from(-0xabcdef).into()),
        Token::Uint(Uint{number: BigUint::from(789u64), size: 13}),
        Token::Int(Int{number: BigInt::from(-12345678900987654321i128), size: 128}),
    ];

    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(), expected_tokens);

    // check that detokenizer gives the same result
    let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(), expected_tokens);
}

#[test]
fn test_int_checks() {
    // number doesn't fit into parameter size
    let input = r#"{ "a" : 128 }"#;
    let params = vec![Param {name: "a".to_owned(), kind: ParamType::Uint(7)}];

    assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).is_err());

    // number doesn't fit into i64 range used in serde_json
    let input = r#"{ "a" : 12345678900987654321 }"#;
    let params = vec![ Param {name: "a".to_owned(), kind: ParamType::Uint(128)}];

    assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).is_err());


    // test BigInt::bits() case for -2^n values

    let input_fit = r#"{ "a" : -128 }"#;
    let input_not_fit = r#"{ "a" : -129 }"#;
    let params = vec![Param {name: "a".to_owned(), kind: ParamType::Int(8)}];

    assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input_fit).unwrap()).is_ok());
    assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input_not_fit).unwrap()).is_err());
}

#[test]
fn test_tokenize_bool() {
    let input = r#"{
        "a" : true,
        "b" : "false"
    }"#;

    let params = vec![
        Param {name: "a".to_owned(), kind: ParamType::Bool},
        Param {name: "b".to_owned(), kind: ParamType::Bool},
    ];

    let expected_tokens = vec![
        Token::Bool(true),
        Token::Bool(false),
    ];

    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(), expected_tokens);

    // check that detokenizer gives the same result
    let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(), expected_tokens);
}

#[test]
fn test_tokenize_arrays() {
    let input = r#"{
        "a" : [123, -456, "789", "-0x0ABc"],
        "b" : [
            [false, "true"],
            [true, true, false]
        ]
    }"#;

    let params = vec![
        Param {name: "a".to_owned(), kind: ParamType::Array(Box::new(ParamType::Dint))},
        Param {name: "b".to_owned(), kind: ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2)},
    ];

    let dint_array = vec![
        Token::Dint(BigInt::from(123)),
        Token::Dint(BigInt::from(-456)),
        Token::Dint(BigInt::from(789)),
        Token::Dint(BigInt::from(-0x0abc)),
    ];

    let bool_array1 = vec![
        Token::Bool(false),
        Token::Bool(true),
    ];

    let bool_array2 = vec![
        Token::Bool(true),
        Token::Bool(true),
        Token::Bool(false),
    ];

    let expected_tokens = vec![
        Token::Array(dint_array),
        Token::FixedArray(vec![Token::Array(bool_array1), Token::Array(bool_array2)]),
    ];

    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(), expected_tokens);

    // check that detokenizer gives the same result
    let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(), expected_tokens);
}

#[test]
#[ignore]
fn test_tokenize_bitstring() {
    let input = r#"{
        "a" : "101000011101011",
        "b" : "x1234567890ABCDEF",
        "c" : "x1234",
        "d" : "x1234_"
    }"#;

    let params = vec![
        Param {name: "a".to_owned(), kind: ParamType::Bits(15)},
        Param {name: "b".to_owned(), kind: ParamType::Bitstring},
        Param {name: "c".to_owned(), kind: ParamType::Bitstring},
        Param {name: "d".to_owned(), kind: ParamType::Bitstring},
    ];


    let expected_tokens = vec![
        Token::Bits(Bitstring::new().append_bits(0b101000011101011, 15).to_owned()),
        Token::Bitstring(Bitstring::create(vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF], 64)),
        Token::Bitstring(Bitstring::create(vec![0x12, 0x34], 16)),
        Token::Bitstring(Bitstring::from_bitstring_with_completion_tag(vec![0x12, 0x34]))
    ];

    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(), expected_tokens);

    // check that detokenizer gives the same result
    let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(), expected_tokens);
}

#[test]
#[ignore]
fn test_tokenize_tuple() {
    let input = r#"{
        "a" : {
            "a" : [-123, "456", "0x789"],
            "b" : "false",
            "c" : "x1234"
        },
        "b" : [
            {
                "a" : true,
                "b" : "x12"
            },
            {
                "a" : false,
                "b" : "x34"
            },
            {
                "a" : true,
                "b" : "x56"
            }
        ]
    }"#;

    let tuple_params1 = vec![
        Param {name: "a".to_owned(), kind: ParamType::Array(Box::new(ParamType::Dint))},
        Param {name: "b".to_owned(), kind: ParamType::Bool},
        Param {name: "c".to_owned(), kind: ParamType::Bits(16)},
    ];

    let tuple_params2 = vec![
        Param {name: "a".to_owned(), kind: ParamType::Bool},
        Param {name: "b".to_owned(), kind: ParamType::Bitstring},
    ];

    let params = vec![
        Param {name: "a".to_owned(), kind: ParamType::Tuple(tuple_params1)},
        Param {name: "b".to_owned(), kind: ParamType::Array(Box::new(ParamType::Tuple(tuple_params2)))},
    ];


    let expected_tokens = vec![
        Token::Tuple(vec![
            Token::Array(vec![
                Token::Dint(BigInt::from(-123)),
                Token::Dint(BigInt::from(456)),
                Token::Dint(BigInt::from(0x789))]),
            Token::Bool(false),
            Token::Bits(Bitstring::create(vec![0x12, 0x34], 16))
        ]),
        Token::Array(vec![
            Token::Tuple(vec![
                Token::Bool(true),
                Token::Bitstring(Bitstring::create(vec![0x12], 8))
            ]),
            Token::Tuple(vec![
                Token::Bool(false),
                Token::Bitstring(Bitstring::create(vec![0x34], 8))
            ]),
            Token::Tuple(vec![
                Token::Bool(true),
                Token::Bitstring(Bitstring::create(vec![0x56], 8))
            ]),
        ]),
    ];

    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(), expected_tokens);

    // check that detokenizer gives the same result
    let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
    assert_eq!(Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(), expected_tokens);
}
