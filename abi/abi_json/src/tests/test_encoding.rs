use std::sync::Arc;
use ed25519_dalek::*;
use num_bigint::{BigInt, BigUint};
use sha2::{Digest, Sha256, Sha512};
use chrono::prelude::*;

use types::{Bitstring, Bit};
use tvm::stack::{BuilderData, IBitstring, SliceData, CellData};
use tvm::stack::dictionary::{HashmapE, HashmapType};

use {Function, Int, Param, ParamType, Token, TokenValue, Uint, ABI_VERSION};

macro_rules! int {
    ($number:expr, $size:expr) => {
        Int {
            number: BigInt::from($number),
            size: $size,
        }
    };
}

macro_rules! uint {
    ($number:expr, $size:expr) => {
        Uint {
            number: BigUint::from($number),
            size: $size,
        }
    };
}

fn get_function_id(signature: &[u8]) -> u32 {
    // Sha256 hash of signature
    let mut hasher = Sha256::new();

    hasher.input(signature);

    let function_hash = hasher.result();

    let mut bytes = [0; 4];
    bytes.copy_from_slice(&function_hash[..4]);

    u32::from_be_bytes(bytes)
}

fn put_array_into_map<T: Into<Bitstring> + Clone>(array: &[T]) -> HashmapE {
    let mut map = HashmapE::with_bit_len(32);

    for i in 0..array.len() {
        let mut index = BuilderData::new();
        index.append_u32(i as u32).unwrap();

        let bitstring: Bitstring = array[i].clone().into();

        let data = BuilderData::with_raw(bitstring.data().clone(), bitstring.length_in_bits()).unwrap();

        map.set(index.into(), &data.into()).unwrap();
    }

    map
}

fn add_array_as_map<T: Into<Bitstring> + Clone>(builder: &mut BuilderData, array: &[T]) {
    let mut bitstring = Bitstring::new();

    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::One);
    bitstring.append_u32(array.len() as u32);


    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();


    let map = put_array_into_map(array);

    match map.data() {
        Some(cell) => {
            builder.append_bit_one().unwrap();
            builder.append_reference_cell(cell.clone());
        }
        None => { builder.append_bit_zero().unwrap(); }
    }
}

fn test_parameters_set(
    func_name: &str,
    func_signature: &[u8],
    inputs: &[Token],
    params: Option<&[Param]>,
    params_tree: BuilderData,
) {
    let mut params_slice = SliceData::from(&params_tree);
    params_slice.get_next_u32().unwrap();
    params_slice.checked_drain_reference().unwrap();
    let func_id = get_function_id(func_signature);

    let input_params: Vec<Param> = if let Some(params) = params {
        params.to_vec()
    } else {
        params_from_tokens(inputs)
    };

    let mut function = Function {
        name: func_name.to_owned(),
        inputs: input_params.clone(),
        outputs: input_params.clone(),
        set_time: false,
        id: 0
    };

    function.id = function.get_function_id();

    let mut timed_function = function.clone();
    timed_function.set_time = true;

    // simple tree check
    let test_tree = function
        .encode_input(inputs.clone(), None)
        .unwrap();

    let mut test_tree = SliceData::from(&test_tree);
    assert_eq!(test_tree.get_next_u32().unwrap(), func_id & 0x7FFFFFFF);
    assert_eq!(test_tree.checked_drain_reference().unwrap(), SliceData::new_empty().cell());
    assert_eq!(test_tree, params_slice);
/*
    // timed tree check
    let test_tree = timed_function
        .encode_input(inputs.clone(), None)
        .unwrap();

    let test_tree = SliceData::from(&test_tree);
    assert_eq!(test_tree.get_next_u32().unwrap(), func_id & 0x7FFFFFFF);
    
    // check time is correct
    let tree_time = test_tree.get_next_u64().unwrap();
    let now = Utc::now().timestamp_millis() as u64;
    assert!(tree_time <= now && tree_time >= now - 1000);

    assert_eq!(test_tree.checked_drain_reference().unwrap(), SliceData::new_empty().cell());
    assert_eq!(test_tree, params_slice);*/
    

    // check signing

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let test_tree = function
        .encode_input(inputs.clone(), Some(&pair))
        .unwrap();
    let mut test_tree = SliceData::from(test_tree);
    let input_copy = test_tree.clone();

    let mut signature = SliceData::from(test_tree.checked_drain_reference().unwrap());
    let signature_data = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&test_tree))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature_data).unwrap();

    let public_key = signature.get_next_bytes(32).unwrap();
    assert_eq!(public_key, pair.public.to_bytes());

    assert_eq!(test_tree.get_next_u32().unwrap(), func_id & 0x7FFFFFFF);
    assert_eq!(test_tree, params_slice);

    // check inputs decoding

    let test_inputs = function.decode_input(input_copy).unwrap();
    assert_eq!(test_inputs, inputs);

    // check outputs decoding

    let mut test_tree = BuilderData::new();
    test_tree.append_u32(func_id | 0x80000000).unwrap();
    test_tree.checked_append_references_and_data(&params_slice).unwrap();

    let test_outputs = function.decode_output(SliceData::from(test_tree)).unwrap();
    assert_eq!(test_outputs, inputs);
}

fn params_from_tokens(tokens: &[Token]) -> Vec<Param> {
     tokens
        .clone()
        .iter()
        .map(|token| token.get_param())
        .collect()
}

fn tokens_from_values(values: Vec<TokenValue>) -> Vec<Token> {
    let param_names = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    values
        .into_iter()
        .zip(param_names)
        .map(|(value, name)| Token {
            name: name.to_owned(),
            value: value,
        })
        .collect()
}

#[test]
fn test_one_input_and_output() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    builder.append_u128(1123).unwrap();

    let values = vec![TokenValue::Uint(Uint {
        number: BigUint::from(1123u128),
        size: 128,
    })];

    test_parameters_set(
        "test_one_input_and_output",
        b"test_one_input_and_output(uint128)(uint128)",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_one_input_and_output_by_data() {
    // builder with reserved signature reference and function ID
    let mut expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x75, 0x0C, 0xE4, 0x7B, 0xAC, 0x80,
    ]).unwrap();
    expected_tree.append_reference(BuilderData::new());

    let values = vec![TokenValue::Int(Int {
        number: BigInt::from(-596784153684i64),
        size: 64,
    })];

    test_parameters_set(
        "test_one_input_and_output_by_data",
        b"test_one_input_and_output_by_data(int64)(int64)",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_empty_params() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    test_parameters_set(
        "test_empty_params",
        b"test_empty_params()()",
        &[],
        None,
        builder);
}

#[test]
fn test_two_params() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    builder.append_bit_one().unwrap();
    builder.append_i32(9434567).unwrap();

    let values = vec![
        TokenValue::Bool(true),
        TokenValue::Int(Int {
            number: BigInt::from(9434567),
            size: 32,
        }),
    ];

    test_parameters_set(
        "test_two_params",
        b"test_two_params(bool,int32)(bool,int32)",
        &tokens_from_values(values),
        None,
        builder,
    );
}
/*
#[test]
fn test_nested_tuples_with_all_simples() {
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))"));
    bitstring.append_bit(&Bit::Zero);
    bitstring.append(&Bitstring::create((-15 as i8).to_be_bytes().to_vec(), 8));
    bitstring.append(&Bitstring::create((9845 as i16).to_be_bytes().to_vec(), 16));
    bitstring.append(&Bitstring::create((-1 as i32).to_be_bytes().to_vec(), 32));
    bitstring.append(&Bitstring::create(
        (12345678 as i64).to_be_bytes().to_vec(),
        64,
    ));
    bitstring.append(&Bitstring::create(
        (-12345678 as i128).to_be_bytes().to_vec(),
        128,
    ));
    bitstring.append(&Bitstring::create((255 as u8).to_be_bytes().to_vec(), 8));
    bitstring.append(&Bitstring::create((0 as u16).to_be_bytes().to_vec(), 16));
    bitstring.append(&Bitstring::create((256 as u32).to_be_bytes().to_vec(), 32));
    bitstring.append(&Bitstring::create((123 as u64).to_be_bytes().to_vec(), 64));
    bitstring.append(&Bitstring::create(
        (1234567890 as u128).to_be_bytes().to_vec(),
        128,
    ));

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);

    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

    let expected_tree = builder.into();

    let values = vec![
        TokenValue::Bool(false),
        TokenValue::Tuple(tokens_from_values(vec![
            TokenValue::Int(int!(-15, 8)),
            TokenValue::Int(int!(9845, 16)),
            TokenValue::Tuple(tokens_from_values(vec![
                TokenValue::Int(int!(-1, 32)),
                TokenValue::Int(int!(12345678, 64)),
                TokenValue::Int(int!(-12345678, 128)),
            ])),
        ])),
        TokenValue::Tuple(tokens_from_values(vec![
            TokenValue::Uint(uint!(255u8, 8)),
            TokenValue::Uint(uint!(0u16, 16)),
            TokenValue::Tuple(tokens_from_values(vec![
                TokenValue::Uint(uint!(256u32, 32)),
                TokenValue::Uint(uint!(123u64, 64)),
                TokenValue::Uint(uint!(1234567890u128, 128)),
            ])),
        ])),
    ];

    test_parameters_set(
        "test_nested_tuples_with_all_simples",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_small_static_array() {
    let input_array: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_small_static_array(uint32[8])(uint32[8])",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);

    for &i in &input_array {
        bitstring.append_u32(i);
    }

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);

    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

    let expected_tree = builder.into();

    let values = vec![TokenValue::FixedArray(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(uint!(i.to_owned(), 32)))
            .collect(),
    )];

    test_parameters_set(
        "test_small_static_array",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_small_static_array_by_data() {
    let input_array: [u16; 5] = [5, 4, 3, 2, 1];

    let expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0x1A, 0x03, 0x2B, 0xB8, 0x80, 0x01, 0x40, 0x01, 0x00, 0x00, 0xc0, 0x00, 0x80, 0x00,
        0x60,
    ]).unwrap();

    let values = vec![TokenValue::FixedArray(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(uint!(i.to_owned(), 16)))
            .collect(),
    )];

    test_parameters_set(
        "test_small_static_array_by_data",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_empty_dynamic_array() {
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_empty_dynamic_array(uint16[])(uint16[])"));

    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::One);
    bitstring.append_u32(0);
    bitstring.append_bit(&Bit::Zero);

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

    let expected_tree = builder.into();

    let values = vec![TokenValue::Array(vec![])];

    let params = vec![Param {
        name: "a".to_owned(),
        kind: ParamType::Array(Box::new(ParamType::Uint(16))),
    }];

    test_parameters_set(
        "test_empty_dynamic_array",
        &tokens_from_values(values),
        Some(&params),
        expected_tree,
    );
}

#[test]
fn test_small_dynamic_array() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_small_dynamic_array(uint16[])(uint16[])",
    ));

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

    add_array_as_map(&mut builder, &input_array);

    let expected_tree = builder.into();

    let values = vec![TokenValue::Array(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(uint!(i.to_owned(), 16)))
            .collect(),
    )];

    test_parameters_set(
        "test_small_dynamic_array",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

fn put_data_into_chain(builder: BuilderData, data: Bitstring) -> BuilderData {
    let mut size = data.length_in_bits();
    let mut current_builder = builder;

    while size != 0 {
        if 0 == current_builder.bits_free() {
            let mut temp_builder = BuilderData::new();
            temp_builder.append_reference(current_builder);

            current_builder = temp_builder;
        }

        let adding_bits = std::cmp::min(current_builder.bits_free(), size);

        let cut = data.substring(size - adding_bits..size);

        let mut vec = vec![];
        cut.into_bitstring_with_completion_tag(&mut vec);

        current_builder.append_bitstring(&vec).unwrap();

        size -= adding_bits;
    }

    current_builder
}

#[test]
fn test_big_static_array() {
    let mut input_array: [u128; 32] = [0; 32];
    for i in 0..32 {
        input_array[i] = i as u128;
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(
        b"test_big_static_array(uint128[32])(uint128[32])",
    ));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for &i in &input_array {
        array_data.append(&Bitstring::create(i.to_be_bytes().to_vec(), 128));
    }

    let mut array_builder = BuilderData::new();

    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder);

    let expected_tree = root_builder.into();

    let values = vec![TokenValue::FixedArray(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(uint!(i.to_owned(), 128)))
            .collect(),
    )];

    test_parameters_set(
        "test_big_static_array",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_huge_static_array() {
    let mut input_array: [i32; 512] = [0; 512];
    for i in 0..input_array.len() {
        input_array[i] = i as i32;
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(
        b"test_huge_static_array(int32[512])(int32[512])",
    ));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for i in 0..input_array.len() {
        array_data.append(&Bitstring::create(
            input_array[i].to_be_bytes().to_vec(),
            32,
        ));
    }

    let mut array_builder = BuilderData::new();

    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder.clone());

    let expected_tree = root_builder.into();

    let values = vec![TokenValue::FixedArray(
        input_array
            .iter()
            .map(|i| TokenValue::Int(int!(i.to_owned(), 32)))
            .collect(),
    )];

    test_parameters_set(
        "test_huge_static_array",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_big_dynamic_array() {
    let mut input_array = Vec::<i64>::new();
    for i in 0..73 {
        input_array.push(i * i as i64);
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_big_dynamic_array(int64[])(int64[])"));

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    add_array_as_map(&mut root_builder, &input_array);

    let expected_tree = root_builder.into();

    let values = vec![TokenValue::Array(
        input_array
            .iter()
            .map(|i| TokenValue::Int(int!(i.to_owned(), 64)))
            .collect(),
    )];

    test_parameters_set(
        "test_big_dynamic_array",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_dynamic_array_of_tuples() {
    let input_array: Vec<(u32, bool)> =
        vec![(1, true), (2, false), (3, true), (4, false), (5, true)];

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_dynamic_array_of_tuples((uint32,bool)[])((uint32,bool)[])",
    ));

    let mut builder = BuilderData::new();
     
    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();


    let bitstring_array: Vec<Bitstring> = input_array
        .iter()
        .cloned()
        .map(|(u, b)| {
            Bitstring::new().append_u32(u).append_bit_bool(b).to_owned()
        })
        .collect();

    add_array_as_map(&mut builder, &bitstring_array);

    let expected_tree = builder.into();

    let values = vec![TokenValue::Array(
        input_array
            .iter()
            .map(|i| {
                TokenValue::Tuple(tokens_from_values(vec![
                    TokenValue::Uint(uint!(i.0, 32)),
                    TokenValue::Bool(i.1),
                ]))
            })
            .collect(),
    )];

    test_parameters_set(
        "test_dynamic_array_of_tuples",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_tuples_with_combined_types() {
    let input_array1: Vec<(u32, bool)> = vec![(1, true), (2, false), (3, true), (4, false)];

    let bitstring_array1: Vec<Bitstring> = input_array1
        .iter()
        .cloned()
        .map(|(u, b)| {
            Bitstring::new().append_u32(u).append_bit_bool(b).to_owned()
        })
        .collect();

    let mut input_array2 = Vec::<i64>::new();
    for i in 0..73 {
        input_array2.push(i * i as i64);
    }
    /*
    let input_array3: [Vec<i64>; 5] = [
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
    ];*/

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_tuples_with_combined_types(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))",
    ));


    let mut chain_builder = BuilderData::new();

    // u8
    chain_builder.append_u8(18).unwrap();


    // Vec<(u32, bool)>
    add_array_as_map(&mut chain_builder, &bitstring_array1);

    // i16
    chain_builder.append_i16(-290 as i16).unwrap();

    // input_array2
    add_array_as_map(&mut chain_builder, &input_array2);

    // [Vec<i64>; 5]
    chain_builder.append_bit_one().unwrap();
    chain_builder.append_bit_zero().unwrap();

    add_array_as_map(&mut chain_builder, &input_array2);

    let mut second_builder = BuilderData::new();

    for _i in 1..5 {
        add_array_as_map(&mut second_builder, &input_array2);
    }

    chain_builder.append_reference(second_builder);

    second_builder = chain_builder;
    chain_builder = BuilderData::new();
    chain_builder.append_reference(second_builder);

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    chain_builder.append_bitstring(&vec).unwrap();

    let expected_tree = chain_builder.into();

    let array1_token_value = TokenValue::Array(
        input_array1
            .iter()
            .map(|i| {
                TokenValue::Tuple(tokens_from_values(vec![
                    TokenValue::Uint(uint!(i.0, 32)),
                    TokenValue::Bool(i.1),
                ]))
            })
            .collect(),
    );

    let array2_token_value = TokenValue::Array(
        input_array2
            .iter()
            .map(|i| TokenValue::Int(int!(*i, 64)))
            .collect(),
    );

    let array3_token_value = TokenValue::FixedArray(vec![
        array2_token_value.clone(),
        array2_token_value.clone(),
        array2_token_value.clone(),
        array2_token_value.clone(),
        array2_token_value.clone(),
    ]);

    let values = vec![
        TokenValue::Uint(uint!(18u8, 8)),
        TokenValue::Tuple(tokens_from_values(vec![
            array1_token_value,
            TokenValue::Int(int!(-290, 16)),
        ])),
        TokenValue::Tuple(tokens_from_values(vec![
            array2_token_value,
            array3_token_value,
        ])),
    ];

    test_parameters_set(
        "test_tuples_with_combined_types",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_reserving_reference() {
    let mut root_builder = BuilderData::new();
    root_builder.append_u8(ABI_VERSION).unwrap();
    root_builder.append_u32(get_function_id(
        b"test_reserving_reference(bytes)(bytes)",
    )).unwrap();

    let array_data = vec![1, 2, 3, 4];
    let array_builder = BuilderData::with_raw(array_data.clone(), array_data.len() * 8).unwrap();
    root_builder.append_reference(array_builder);

    let expected_tree: SliceData = root_builder.into();

    let values = vec![TokenValue::Bytes(array_data.clone())];

    let tokens = tokens_from_values(values);
    let params: Vec<Param> = tokens
        .clone()
        .iter()
        .map(|token| token.get_param())
        .collect();

    let signed_function = Function {
        name: "test_reserving_reference".to_owned(),
        inputs: params.clone(),
        outputs: params.clone(),
        set_time: true,
        id: 0
    };

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let signed_test_tree = signed_function.encode_input(&tokens, Some(&pair)).unwrap();
    let mut signed_test_tree = SliceData::from(signed_test_tree);

    let mut signature = SliceData::from(signed_test_tree.checked_drain_reference().unwrap());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = signed_test_tree.into_cell().repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature).unwrap();

    println!("{:#.2}", expected_tree.into_cell());
    println!("{:#.2}", signed_test_tree.into_cell());
    assert_eq!(expected_tree, signed_test_tree);
}

#[test]
fn test_add_signature() {
    let tokens = tokens_from_values(vec![TokenValue::Uint(uint!(456u32, 32))]);

    let mut function = Function {
        name: "test_add_signature".to_owned(),
        inputs: params_from_tokens(&tokens),
        outputs: vec![],
        signed: false,
        id: 0
    };

    function.id = function.get_function_id();

    let (msg, data_to_sign) = function.prepare_input_for_sign(&tokens).unwrap();

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());
    let signature = pair.sign::<Sha512>(&data_to_sign).to_bytes().to_vec();

    let msg = Function::add_sign_to_encoded_input(&signature, &pair.public.to_bytes(), msg.into()).unwrap();

    assert_eq!(function.decode_input(msg.into()).unwrap(), tokens);
}
*/