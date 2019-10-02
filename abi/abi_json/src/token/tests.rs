mod tokenize_tests {
    use crate::{Int, Param, ParamType, Token, TokenValue, Uint};
    use num_bigint::{BigInt, BigUint};
    // use serde::Serialize;
    use token::{Detokenizer, Tokenizer};
    use ton_abi_core::types::Bitstring;
    use tvm::block::MsgAddressInt;
    use tvm::stack::dictionary::HashmapE;
    use tvm::stack::{SliceData};
    use tvm::types::AccountId;

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
            Param {
                name: "a".to_owned(),
                kind: ParamType::Uint(8),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Int(16),
            },
            Param {
                name: "c".to_owned(),
                kind: ParamType::Duint,
            },
            Param {
                name: "d".to_owned(),
                kind: ParamType::Dint,
            },
            Param {
                name: "e".to_owned(),
                kind: ParamType::Uint(13),
            },
            Param {
                name: "f".to_owned(),
                kind: ParamType::Int(128),
            },
        ];

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Uint(Uint {
                    number: BigUint::from(123u8),
                    size: 8,
                }),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::Int(Int {
                    number: BigInt::from(-456i16),
                    size: 16,
                }),
            },
            Token {
                name: "c".to_owned(),
                value: TokenValue::Duint(BigUint::from(0xABCDEFu32).into()),
            },
            Token {
                name: "d".to_owned(),
                value: TokenValue::Dint(BigInt::from(-0xabcdef).into()),
            },
            Token {
                name: "e".to_owned(),
                value: TokenValue::Uint(Uint {
                    number: BigUint::from(789u64),
                    size: 13,
                }),
            },
            Token {
                name: "f".to_owned(),
                value: TokenValue::Int(Int {
                    number: BigInt::from(-12345678900987654321i128),
                    size: 128,
                }),
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
    fn test_int_checks() {
        // number doesn't fit into parameter size
        let input = r#"{ "a" : 128 }"#;
        let params = vec![Param {
            name: "a".to_owned(),
            kind: ParamType::Uint(7),
        }];

        assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).is_err());

        // number doesn't fit into i64 range used in serde_json
        let input = r#"{ "a" : 12345678900987654321 }"#;
        let params = vec![Param {
            name: "a".to_owned(),
            kind: ParamType::Uint(128),
        }];

        assert!(Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).is_err());

        // test BigInt::bits() case for -2^n values

        let input_fit = r#"{ "a" : -128 }"#;
        let input_not_fit = r#"{ "a" : -129 }"#;
        let params = vec![Param {
            name: "a".to_owned(),
            kind: ParamType::Int(8),
        }];

        assert!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input_fit).unwrap()).is_ok()
        );
        assert!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input_not_fit).unwrap())
                .is_err()
        );
    }

    #[test]
    fn test_tokenize_bool() {
        let input = r#"{
            "a" : true,
            "b" : "false"
        }"#;

        let params = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Bool,
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Bool,
            },
        ];

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Bool(true),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::Bool(false),
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
    fn test_tokenize_empty() {
        let input = r#"{}"#;

        let params = vec![];

        let expected_tokens = vec![];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
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
            Param {
                name: "a".to_owned(),
                kind: ParamType::Array(Box::new(ParamType::Dint)),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::FixedArray(
                    Box::new(ParamType::Array(Box::new(ParamType::Bool))),
                    2,
                ),
            },
        ];

        let dint_array = vec![
            TokenValue::Dint(BigInt::from(123)),
            TokenValue::Dint(BigInt::from(-456)),
            TokenValue::Dint(BigInt::from(789)),
            TokenValue::Dint(BigInt::from(-0x0abc)),
        ];

        let bool_array1 = vec![TokenValue::Bool(false), TokenValue::Bool(true)];

        let bool_array2 = vec![
            TokenValue::Bool(true),
            TokenValue::Bool(true),
            TokenValue::Bool(false),
        ];

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Array(dint_array),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::FixedArray(vec![
                    TokenValue::Array(bool_array1),
                    TokenValue::Array(bool_array2),
                ]),
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
    fn test_tokenize_bitstring() {
        let input = r#"{
            "a" : "101000011101011",
            "b" : "x1234567890ABCDEF",
            "c" : "x1234",
            "d" : "x1234_"
        }"#;

        let params = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Bits(15),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Bitstring,
            },
            Param {
                name: "c".to_owned(),
                kind: ParamType::Bitstring,
            },
            Param {
                name: "d".to_owned(),
                kind: ParamType::Bitstring,
            },
        ];

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Bits(
                    Bitstring::new()
                        .append_bits(0b101000011101011, 15)
                        .to_owned(),
                ),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::Bitstring(Bitstring::create(
                    vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF],
                    64,
                )),
            },
            Token {
                name: "c".to_owned(),
                value: TokenValue::Bitstring(Bitstring::create(vec![0x12, 0x34], 16)),
            },
            Token {
                name: "d".to_owned(),
                value: TokenValue::Bitstring(Bitstring::from_bitstring_with_completion_tag(vec![
                    0x12, 0x34,
                ])),
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
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
            Param {
                name: "a".to_owned(),
                kind: ParamType::Array(Box::new(ParamType::Dint)),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Bool,
            },
            Param {
                name: "c".to_owned(),
                kind: ParamType::Bits(16),
            },
        ];

        let tuple_params2 = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Bool,
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Bitstring,
            },
        ];

        let params = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Tuple(tuple_params1),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Array(Box::new(ParamType::Tuple(tuple_params2))),
            },
        ];

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Tuple(vec![
                    Token {
                        name: "a".to_owned(),
                        value: TokenValue::Array(vec![
                            TokenValue::Dint(BigInt::from(-123)),
                            TokenValue::Dint(BigInt::from(456)),
                            TokenValue::Dint(BigInt::from(0x789)),
                        ]),
                    },
                    Token {
                        name: "b".to_owned(),
                        value: TokenValue::Bool(false),
                    },
                    Token {
                        name: "c".to_owned(),
                        value: TokenValue::Bits(Bitstring::create(vec![0x12, 0x34], 16)),
                    },
                ]),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::Array(vec![
                    TokenValue::Tuple(vec![
                        Token {
                            name: "a".to_owned(),
                            value: TokenValue::Bool(true),
                        },
                        Token {
                            name: "b".to_owned(),
                            value: TokenValue::Bitstring(Bitstring::create(vec![0x12], 8)),
                        },
                    ]),
                    TokenValue::Tuple(vec![
                        Token {
                            name: "a".to_owned(),
                            value: TokenValue::Bool(false),
                        },
                        Token {
                            name: "b".to_owned(),
                            value: TokenValue::Bitstring(Bitstring::create(vec![0x34], 8)),
                        },
                    ]),
                    TokenValue::Tuple(vec![
                        Token {
                            name: "a".to_owned(),
                            value: TokenValue::Bool(true),
                        },
                        Token {
                            name: "b".to_owned(),
                            value: TokenValue::Bitstring(Bitstring::create(vec![0x56], 8)),
                        },
                    ]),
                ]),
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );

        // check that detokenizer gives the same result
        let input = Detokenizer::detokenize(&params, &expected_tokens).unwrap();
        println!("{}", input);
        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(&input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
    fn test_tokenize_hashmap() {
        let input = r#"{
            "a": {
                "-1": 42,
                "12": 37
                }
            },
        }"#;

        let params = vec![
            Param::new("a", ParamType::Map(8, true, Box::new(ParamType::Dint))),
        ];

        let mut hashmap = HashmapE::with_bit_len(8);
        hashmap.set_serializable(&-1i8, &42u32).unwrap();
        hashmap.set_serializable(&12i8, &37u32).unwrap();

        let expected_tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Map(hashmap)
            }
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );
    }

    #[test]
    fn test_tokenize_address() {

        let addr = MsgAddressInt::with_standart(None, -17, AccountId::from([0x55; 32])).unwrap();
        let j = serde_json::to_string_pretty(&addr).unwrap();
        println!("{}", j);

        // let j = serde_json::to_string_pretty(&AccountId::from([0x55; 32])).unwrap();
        // println!("{}", j);

        let addr = MsgAddressInt::with_variant(None, -177, SliceData::new(vec![0x55, 0x58])).unwrap();
        let j = serde_json::to_string_pretty(&addr).unwrap();
        println!("{}", j);

        let input = r#"{
            "std": {
                "anycast": null,
                "workchain_id": -17,
                "address": "5555555555555555555555555555555555555555555555555555555555555555"
            },
            "var": {
                "anycast": null,
                "workchain_id": -177,
                "address": "555"
            }
        }"#;

        let params = vec![
            Param::new("std", ParamType::StdAddress),
            Param::new("var", ParamType::VarAddress),
        ];

        let expected_tokens = vec![
            Token {
                name: "std".to_owned(),
                value: TokenValue::MsgAddress(MsgAddressInt::with_standart(
                    None, -17, AccountId::from([0x55; 32])).unwrap())
            },
            Token {
                name: "std".to_owned(),
                value: TokenValue::MsgAddress(MsgAddressInt::with_variant(
                    None, -177, SliceData::new(vec![0x55, 0x58])).unwrap())
            },
        ];

        assert_eq!(
            Tokenizer::tokenize_all(&params, &serde_json::from_str(input).unwrap()).unwrap(),
            expected_tokens
        );
    }
}

mod types_check_tests {
    use num_bigint::{BigInt, BigUint};
    use ton_abi_core::types::Bitstring;
    use {Int, Param, ParamType, Token, TokenValue, Uint};

    #[test]
    fn test_type_check() {
        fn assert_type_check(tokens: &[Token], params: &[Param]) {
            assert!(Token::types_check(&tokens, params))
        }

        fn assert_not_type_check(tokens: &[Token], params: &[Param]) {
            assert!(!Token::types_check(&tokens, params))
        }

        let big_int = BigInt::from(123);
        let big_uint = BigUint::from(456u32);

        let tokens = vec![
            Token {
                name: "a".to_owned(),
                value: TokenValue::Uint(Uint {
                    number: big_uint.clone(),
                    size: 32,
                }),
            },
            Token {
                name: "b".to_owned(),
                value: TokenValue::Int(Int {
                    number: big_int.clone(),
                    size: 64,
                }),
            },
            Token {
                name: "c".to_owned(),
                value: TokenValue::Dint(big_int.clone().into()),
            },
            Token {
                name: "d".to_owned(),
                value: TokenValue::Duint(big_uint.clone().into()),
            },
            Token {
                name: "e".to_owned(),
                value: TokenValue::Bool(false),
            },
            Token {
                name: "f".to_owned(),
                value: TokenValue::Array(vec![TokenValue::Bool(false), TokenValue::Bool(true)]),
            },
            Token {
                name: "g".to_owned(),
                value: TokenValue::FixedArray(vec![
                    TokenValue::Dint(big_int.clone().into()),
                    TokenValue::Dint(big_int.clone().into()),
                ]),
            },
            Token {
                name: "h".to_owned(),
                value: TokenValue::Bits(Bitstring::create(vec![1, 2, 3], 15)),
            },
            Token {
                name: "i".to_owned(),
                value: TokenValue::Bitstring(Bitstring::create(vec![1, 2, 3], 7)),
            },
            Token {
                name: "j".to_owned(),
                value: TokenValue::Tuple(vec![
                    Token {
                        name: "a".to_owned(),
                        value: TokenValue::Bool(true),
                    },
                    Token {
                        name: "b".to_owned(),
                        value: TokenValue::Duint(big_uint.clone().into()),
                    },
                ]),
            },
        ];

        let tuple_params = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Bool,
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Duint,
            },
        ];

        let params = vec![
            Param {
                name: "a".to_owned(),
                kind: ParamType::Uint(32),
            },
            Param {
                name: "b".to_owned(),
                kind: ParamType::Int(64),
            },
            Param {
                name: "c".to_owned(),
                kind: ParamType::Dint,
            },
            Param {
                name: "d".to_owned(),
                kind: ParamType::Duint,
            },
            Param {
                name: "e".to_owned(),
                kind: ParamType::Bool,
            },
            Param {
                name: "f".to_owned(),
                kind: ParamType::Array(Box::new(ParamType::Bool)),
            },
            Param {
                name: "g".to_owned(),
                kind: ParamType::FixedArray(Box::new(ParamType::Dint), 2),
            },
            Param {
                name: "h".to_owned(),
                kind: ParamType::Bits(15),
            },
            Param {
                name: "i".to_owned(),
                kind: ParamType::Bitstring,
            },
            Param {
                name: "j".to_owned(),
                kind: ParamType::Tuple(tuple_params),
            },
        ];

        assert_type_check(&tokens, &params);

        let mut tokens_wrong_type = tokens.clone();
        tokens_wrong_type[0] = Token {
            name: "a".to_owned(),
            value: TokenValue::Bool(false),
        };
        assert_not_type_check(&tokens_wrong_type, &params);

        let mut tokens_wrong_int_size = tokens.clone();
        tokens_wrong_int_size[0] = Token {
            name: "a".to_owned(),
            value: TokenValue::Uint(Uint {
                number: big_uint.clone(),
                size: 30,
            }),
        };
        assert_not_type_check(&tokens_wrong_int_size, &params);

        let mut tokens_wrong_parameters_count = tokens.clone();
        tokens_wrong_parameters_count.pop();
        assert_not_type_check(&tokens_wrong_parameters_count, &params);

        let mut tokens_wrong_fixed_array_size = tokens.clone();
        tokens_wrong_fixed_array_size[6] = Token {
            name: "g".to_owned(),
            value: TokenValue::FixedArray(vec![TokenValue::Dint(big_int.clone().into())]),
        };
        assert_not_type_check(&tokens_wrong_fixed_array_size, &params);

        let mut tokens_wrong_array_type = tokens.clone();
        tokens_wrong_array_type[5] = Token {
            name: "f".to_owned(),
            value: TokenValue::Array(vec![
                TokenValue::Bool(false),
                TokenValue::Dint(big_int.clone().into()),
            ]),
        };
        assert_not_type_check(&tokens_wrong_array_type, &params);

        let mut tokens_wrong_tuple_type = tokens.clone();
        tokens_wrong_tuple_type[9] = Token {
            name: "f".to_owned(),
            value: TokenValue::Tuple(vec![
                Token {
                    name: "a".to_owned(),
                    value: TokenValue::Int(Int {
                        number: big_int.clone(),
                        size: 16,
                    }),
                },
                Token {
                    name: "b".to_owned(),
                    value: TokenValue::Duint(big_uint.clone().into()),
                },
            ]),
        };
        assert_not_type_check(&tokens_wrong_tuple_type, &params);
    }
}
