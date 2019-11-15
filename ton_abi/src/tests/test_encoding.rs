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

use std::sync::Arc;
use ed25519_dalek::*;
use num_bigint::{BigInt, BigUint};
use sha2::{Digest, Sha256, Sha512};
use chrono::prelude::*;

use tvm::stack::{BuilderData, IBitstring, SliceData, CellData};
use tvm::stack::dictionary::{HashmapE, HashmapType};
use tvm::block::{AnycastInfo, BlockResult, Grams, MsgAddress, Serializable};
use tvm::types::AccountId;

use {Function, Int, Param, ParamType, Token, TokenValue, Uint};

fn get_function_id(signature: &[u8]) -> u32 {
    // Sha256 hash of signature
    let mut hasher = Sha256::new();

    hasher.input(signature);

    let function_hash = hasher.result();

    let mut bytes = [0; 4];
    bytes.copy_from_slice(&function_hash[..4]);

    u32::from_be_bytes(bytes)
}

fn put_array_into_map<T: Serializable>(array: &[T]) -> HashmapE {
    let mut map = HashmapE::with_bit_len(32);

    for i in 0..array.len() {
        let index = (i as u32).write_to_new_cell().unwrap();
        let data = array[i].write_to_new_cell().unwrap();
        map.set(index.into(), &data.into()).unwrap();
    }

    map
}

fn add_array_as_map<T: Serializable>(builder: &mut BuilderData, array: &[T], fixed: bool) {
    if !fixed {
        builder.append_u32(array.len() as u32).unwrap();
    }

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
    timed_signature: &[u8],
    inputs: &[Token],
    params: Option<&[Param]>,
    params_tree: BuilderData,
) {
    let check_time = params_tree.bits_free() > 64;

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
    timed_function.id = timed_function.get_function_id();

    // simple tree check
    let test_tree = function
        .encode_input(inputs.clone(), false, None)
        .unwrap();

    let mut test_tree = SliceData::from(&test_tree);
    assert_eq!(test_tree.get_next_u32().unwrap(), func_id & 0x7FFFFFFF);
    assert_eq!(test_tree.checked_drain_reference().unwrap(), SliceData::new_empty().cell());
    println!("{:#.2}", test_tree.into_cell());
    println!("{:#.2}", params_slice.into_cell());
    assert_eq!(test_tree, params_slice);

    if check_time {
        // timed tree check
        let test_tree = timed_function
            .encode_input(inputs.clone(), false, None)
            .unwrap();

        let mut test_tree = SliceData::from(&test_tree);
        let func_id = get_function_id(timed_signature);
        assert_eq!(test_tree.get_next_u32().unwrap(), func_id & 0x7FFFFFFF);
        
        // check time is correct
        let tree_time = test_tree.get_next_u64().unwrap();
        let now = Utc::now().timestamp_millis() as u64;
        assert!(tree_time <= now && tree_time >= now - 1000);

        assert_eq!(test_tree.checked_drain_reference().unwrap(), SliceData::new_empty().cell());
        assert_eq!(test_tree, params_slice);
    }
    

    // check signing

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let test_tree = function
        .encode_input(inputs.clone(), false, Some(&pair))
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

    let test_inputs = function.decode_input(input_copy, false).unwrap();
    assert_eq!(test_inputs, inputs);

    // check outputs decoding

    let mut test_tree = BuilderData::new();
    test_tree.append_reference(BuilderData::new());
    test_tree.append_u32(func_id | 0x80000000).unwrap();
    test_tree.checked_append_references_and_data(&params_slice).unwrap();
    let mut test_tree = SliceData::from(test_tree);
    test_tree.checked_drain_reference().unwrap();

    let test_outputs = function.decode_output(test_tree, false).unwrap();
    assert_eq!(test_outputs, inputs);
}

fn params_from_tokens(tokens: &[Token]) -> Vec<Param> {
     tokens.iter().map(|ref token| token.get_param()).collect()
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
        b"test_one_input_and_output(uint128)(uint128)v1",
        b"test_one_input_and_output(time,uint128)(uint128)v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_with_grams() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    let grams = Grams::from(173742);
    grams.write_to(&mut builder).unwrap();

    let values = vec![TokenValue::Gram(grams)];

    test_parameters_set(
        "test_with_grams",
        b"test_with_grams(gram)(gram)v1",
        b"test_with_grams(time,gram)(gram)v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_with_address() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    let anycast = AnycastInfo::with_rewrite_pfx(SliceData::new(vec![0x77, 0x78, 0x79, 0x80])).unwrap();
    let addresses = vec![
        MsgAddress::AddrNone,
        MsgAddress::with_extern(SliceData::new(vec![0x55, 0x80])).unwrap(),
        MsgAddress::with_standart(Some(anycast.clone()), -1, AccountId::from([0x11; 32])).unwrap(),
        MsgAddress::with_standart(Some(anycast.clone()), -1, AccountId::from([0x11; 32])).unwrap(),
        MsgAddress::with_variant(Some(anycast.clone()), -128, SliceData::new(vec![0x66, 0x67, 0x68, 0x69, 0x80])).unwrap(),
        MsgAddress::with_standart(Some(anycast.clone()), -1, AccountId::from([0x11; 32])).unwrap(),
    ];
    builder.append_reference(BuilderData::with_bitstring(vec![1, 2, 3, 0x80]).unwrap());
    let mut values = vec![TokenValue::Cell(BuilderData::with_bitstring(vec![1, 2, 3, 0x80]).unwrap().into())];
    // we don't know about serilization changes in MsgAddress if them don't fit in one cell - split to references
    addresses.iter().take(5).for_each(|address| address.write_to(&mut builder).unwrap());
    builder.append_reference(addresses.last().unwrap().write_to_new_cell().unwrap());
    addresses.iter().for_each(|address| {
        values.push(TokenValue::Address(address.clone()));
    });

    test_parameters_set(
        "test_with_address",
        b"test_with_address(cell,address,address,address,address,address,address)(cell,address,address,address,address,address,address)v1",
        b"test_with_address(time,cell,address,address,address,address,address,address)(cell,address,address,address,address,address,address)v1",
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
        b"test_one_input_and_output_by_data(int64)(int64)v1",
        b"test_one_input_and_output_by_data(time,int64)(int64)v1",
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
        b"test_empty_params()()v1",
        b"test_empty_params(time)()v1",
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
        b"test_two_params(bool,int32)(bool,int32)v1",
        b"test_two_params(time,bool,int32)(bool,int32)v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_four_refs() {
    // builder with reserved signature reference and function ID
    let bytes = vec![0x55; 300]; // 300 = 127 + 127 + 46
    let mut builder = BuilderData::with_raw(vec![0x55; 127], 127 * 8).unwrap();
    builder.append_reference(BuilderData::with_raw(vec![0x55; 127], 127 * 8).unwrap());
    let mut bytes_builder = BuilderData::with_raw(vec![0x55; 46], 46 * 8).unwrap();
    bytes_builder.append_reference(builder);

    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_bit_one().unwrap();
    builder.append_reference(BuilderData::new());
    builder.append_reference(bytes_builder.clone());
    builder.append_reference(bytes_builder.clone());

    let mut new_builder = BuilderData::new();
    new_builder.append_i32(9434567).unwrap();
    new_builder.append_reference(bytes_builder.clone());
    new_builder.append_reference(bytes_builder.clone());
    builder.append_reference(new_builder);

    let values = vec![
        TokenValue::Bool(true),
        TokenValue::Bytes(bytes.clone()),
        TokenValue::Bytes(bytes.clone()),
        TokenValue::Bytes(bytes.clone()),
        TokenValue::Bytes(bytes.clone()),
        TokenValue::Int(Int::new(9434567, 32)),
    ];

    test_parameters_set(
        "test_four_refs",
        b"test_four_refs(bool,bytes,bytes,bytes,bytes,int32)(bool,bytes,bytes,bytes,bytes,int32)v1",
        b"test_four_refs(time,bool,bytes,bytes,bytes,bytes,int32)(bool,bytes,bytes,bytes,bytes,int32)v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_nested_tuples_with_all_simples() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

   
    builder.append_bit_zero().unwrap();
    builder.append_i8(-15 as i8).unwrap();
    builder.append_i16(9845 as i16).unwrap();
    builder.append_i32(-1 as i32).unwrap();
    builder.append_i64(12345678 as i64).unwrap();
    builder.append_i128(-12345678 as i128).unwrap();
    builder.append_u8(255 as u8).unwrap();
    builder.append_u16(0 as u16).unwrap();
    builder.append_u32(256 as u32).unwrap();
    builder.append_u64(123 as u64).unwrap();
    builder.append_u128(1234567890 as u128).unwrap();

    let values = vec![
        TokenValue::Bool(false),
        TokenValue::Tuple(tokens_from_values(vec![
            TokenValue::Int(Int::new(-15, 8)),
            TokenValue::Int(Int::new(9845, 16)),
            TokenValue::Tuple(tokens_from_values(vec![
                TokenValue::Int(Int::new(-1, 32)),
                TokenValue::Int(Int::new(12345678, 64)),
                TokenValue::Int(Int::new(-12345678, 128)),
            ])),
        ])),
        TokenValue::Tuple(tokens_from_values(vec![
            TokenValue::Uint(Uint::new(255, 8)),
            TokenValue::Uint(Uint::new(0, 16)),
            TokenValue::Tuple(tokens_from_values(vec![
                TokenValue::Uint(Uint::new(256, 32)),
                TokenValue::Uint(Uint::new(123, 64)),
                TokenValue::Uint(Uint::new(1234567890, 128)),
            ])),
        ])),
    ];

    test_parameters_set(
        "test_nested_tuples_with_all_simples",
        b"test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))v1",
        b"test_nested_tuples_with_all_simples(time,bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_static_array_of_ints() {
    let input_array: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    add_array_as_map(&mut builder, &input_array, true);

    let values = vec![TokenValue::FixedArray(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(Uint::new(i.to_owned() as u128, 32)))
            .collect(),
    )];

    test_parameters_set(
        "test_static_array_of_ints",
        b"test_static_array_of_ints(uint32[8])(uint32[8])v1",
        b"test_static_array_of_ints(time,uint32[8])(uint32[8])v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

#[test]
fn test_empty_dynamic_array() {
    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    add_array_as_map(&mut builder, &Vec::<u16>::new(), false);

    let values = vec![TokenValue::Array(vec![])];

    let params = vec![Param {
        name: "a".to_owned(),
        kind: ParamType::Array(Box::new(ParamType::Uint(16))),
    }];

    test_parameters_set(
        "test_empty_dynamic_array",
        b"test_empty_dynamic_array(uint16[])(uint16[])v1",
        b"test_empty_dynamic_array(time,uint16[])(uint16[])v1",
        &tokens_from_values(values),
        Some(&params),
        builder,
    );
}

#[test]
fn test_dynamic_array_of_ints() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    add_array_as_map(&mut builder, &input_array, false);

    let values = vec![TokenValue::Array(
        input_array
            .iter()
            .map(|i| TokenValue::Uint(Uint::new(i.to_owned() as u128, 16)))
            .collect(),
    )];

    test_parameters_set(
        "test_dynamic_array_of_ints",
        b"test_dynamic_array_of_ints(uint16[])(uint16[])v1",
        b"test_dynamic_array_of_ints(time,uint16[])(uint16[])v1",
        &tokens_from_values(values),
        None,
        builder,
    );
}

struct TupleDwordBool(u32, bool);

impl Serializable for TupleDwordBool {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.0.write_to(cell)?;
        self.1.write_to(cell)?;
        Ok(())
    }
}

impl From<&(u32, bool)> for TupleDwordBool {
    fn from(a: &(u32, bool)) -> Self {
        TupleDwordBool(a.0, a.1)
    }
}

#[test]
fn test_dynamic_array_of_tuples() {
    let input_array: Vec<(u32, bool)> =
        vec![(1, true), (2, false), (3, true), (4, false), (5, true)];

    // builder with reserved signature reference and function ID
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder.append_reference(BuilderData::new());

    let bitstring_array: Vec<TupleDwordBool> = input_array
        .iter()
        .map(|a| TupleDwordBool::from(a))
        .collect();

    add_array_as_map(&mut builder, &bitstring_array, false);

    let expected_tree = builder.into();

    let values = vec![TokenValue::Array(
        input_array
            .iter()
            .map(|i| {
                TokenValue::Tuple(tokens_from_values(vec![
                    TokenValue::Uint(Uint::new(i.0 as u128, 32)),
                    TokenValue::Bool(i.1),
                ]))
            })
            .collect(),
    )];

    test_parameters_set(
        "test_dynamic_array_of_tuples",
        b"test_dynamic_array_of_tuples((uint32,bool)[])((uint32,bool)[])v1",
        b"test_dynamic_array_of_tuples(time,(uint32,bool)[])((uint32,bool)[])v1",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_tuples_with_combined_types() {
    let input_array1: Vec<(u32, bool)> = vec![(1, true), (2, false), (3, true), (4, false)];

    let bitstring_array1: Vec<TupleDwordBool> = input_array1
        .iter()
        .map(|a| TupleDwordBool::from(a))
        .collect();

    let mut input_array2 = Vec::<u64>::new();
    for i in 0..73 {
        input_array2.push(i * i);
    }

    // builder with reserved signature reference and function ID
    let mut chain_builder = BuilderData::new();
    chain_builder.append_u32(0).unwrap();
    chain_builder.append_reference(BuilderData::new());

    // u8
    chain_builder.append_u8(18).unwrap();


    // Vec<(u32, bool)>
    add_array_as_map(&mut chain_builder, &bitstring_array1, false);

    // i16
    chain_builder.append_i16(-290 as i16).unwrap();

    // input_array2
    add_array_as_map(&mut chain_builder, &input_array2, false);

    let mut map = HashmapE::with_bit_len(32);

    // [Vec<i64>; 5]
    for i in 0..5 {
        let mut builder = BuilderData::new();
        add_array_as_map(&mut builder, &input_array2, false);

        let mut index = BuilderData::new();
        index.append_u32(i).unwrap();

        map.set(index.into(), &builder.into()).unwrap();
    }

    let mut second_builder = BuilderData::new();
    second_builder.append_bit_one().unwrap();
    second_builder.append_reference(BuilderData::from(map.data().unwrap()));

    chain_builder.append_reference(second_builder);

    let array1_token_value = TokenValue::Array(
        input_array1
            .iter()
            .map(|i| {
                TokenValue::Tuple(tokens_from_values(vec![
                    TokenValue::Uint(Uint::new(i.0 as u128, 32)),
                    TokenValue::Bool(i.1),
                ]))
            })
            .collect(),
    );

    let array2_token_value = TokenValue::Array(
        input_array2
            .iter()
            .map(|i| TokenValue::Int(Int::new(*i as i128, 64)))
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
        TokenValue::Uint(Uint::new(18, 8)),
        TokenValue::Tuple(tokens_from_values(vec![
            array1_token_value,
            TokenValue::Int(Int::new(-290, 16)),
        ])),
        TokenValue::Tuple(tokens_from_values(vec![
            array2_token_value,
            array3_token_value,
        ])),
    ];

    test_parameters_set(
        "test_tuples_with_combined_types",
        b"test_tuples_with_combined_types(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))v1",
        b"test_tuples_with_combined_types(time,uint8,((uint32,bool)[],int16),(int64[],int64[][5]))(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))v1",
        &tokens_from_values(values),
        None,
        chain_builder,
    );
}

#[test]
fn test_add_signature() {
    let tokens = tokens_from_values(vec![TokenValue::Uint(Uint::new(456, 32))]);

    let mut function = Function {
        name: "test_add_signature".to_owned(),
        inputs: params_from_tokens(&tokens),
        outputs: vec![],
        set_time: false,
        id: 0
    };

    function.id = function.get_function_id();

    let (msg, data_to_sign) = function.create_unsigned_call(&tokens, false).unwrap();

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());
    let signature = pair.sign::<Sha512>(&data_to_sign).to_bytes().to_vec();

    let msg = Function::add_sign_to_encoded_input(&signature, &pair.public.to_bytes(), msg.into()).unwrap();

    assert_eq!(function.decode_input(msg.into(), false).unwrap(), tokens);
}
