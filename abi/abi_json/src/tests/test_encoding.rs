use std::sync::Arc;
use ed25519_dalek::*;
use num_bigint::{BigInt, BigUint};
use sha2::{Digest, Sha256, Sha512};

use ton_abi_core::types::{Dint, Duint, Bitstring, Bit};
use tvm::stack::{BuilderData, IBitstring, SliceData, CellData};

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

fn test_parameters_set(
    func_name: &str,
    inputs: &[Token],
    params: Option<&[Param]>,
    expected_tree: BuilderData,
) {
    let mut expected_tree_with_ref = expected_tree.clone();
    expected_tree_with_ref.prepend_reference(BuilderData::new());

    let input_params: Vec<Param> = if let Some(params) = params {
        params.to_vec()
    } else {
        inputs
            .clone()
            .iter()
            .map(|token| token.get_param())
            .collect()
    };

    let mut not_signed_function = Function {
        name: func_name.to_owned(),
        inputs: input_params.clone(),
        outputs: input_params.clone(),
        signed: false,
        id: 0
    };

    not_signed_function.id = not_signed_function.get_function_id();

    let test_tree = not_signed_function
        .encode_input(inputs.clone(), None)
        .unwrap();
    assert_eq!(test_tree, expected_tree_with_ref);

    // check signing

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let mut signed_function = not_signed_function.clone();
    signed_function.signed = true;

    let signed_test_tree = signed_function
        .encode_input(inputs.clone(), Some(&pair))
        .unwrap();
    let mut message = SliceData::from(signed_test_tree);

    let mut signature = SliceData::from(message.checked_drain_reference().unwrap());

    assert_eq!(SliceData::from(expected_tree), message);

    let signature_data = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature_data).unwrap();

    let public_key = signature.get_next_bytes(32).unwrap();
    assert_eq!(public_key, pair.public.to_bytes());

    // check output decoding

    let mut test_tree = SliceData::from(test_tree);

    let test_inputs = not_signed_function.decode_input(test_tree.clone()).unwrap();
    assert_eq!(test_inputs, inputs);

    test_tree.checked_drain_reference().unwrap();

    let test_outputs = not_signed_function.decode_output(test_tree).unwrap();
    assert_eq!(test_outputs, inputs);
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
    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder
        .append_u32(get_function_id(
            b"test_one_input_and_output(uint128)(uint128)",
        ))
        .unwrap();
    builder.append_u128(1123).unwrap();

    let expected_tree = builder.into();

    let values = vec![TokenValue::Uint(Uint {
        number: BigUint::from(1123u128),
        size: 128,
    })];

    test_parameters_set(
        "test_one_input_and_output",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_one_input_and_output_by_data() {
    let expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0x7B, 0xE7, 0x79, 0x17, 0xFF, 0xFF, 0xFF, 0x75, 0x0C, 0xE4, 0x7B, 0xAC, 0x80,
    ]).unwrap();

    let values = vec![TokenValue::Int(Int {
        number: BigInt::from(-596784153684i64),
        size: 64,
    })];

    test_parameters_set(
        "test_one_input_and_output_by_data",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_empty_params() {
    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder
        .append_u32(get_function_id(b"test_empty_params()()"))
        .unwrap();

    let expected_tree = builder.into();

    test_parameters_set("test_empty_params", &[], None, expected_tree);
}

#[test]
fn test_two_params() {
    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder
        .append_u32(get_function_id(b"test_two_params(bool,int32)(bool,int32)"))
        .unwrap();
    builder.append_bit_one().unwrap();
    builder.append_i32(9434567).unwrap();

    let expected_tree = builder.into();

    let values = vec![
        TokenValue::Bool(true),
        TokenValue::Int(Int {
            number: BigInt::from(9434567),
            size: 32,
        }),
    ];

    test_parameters_set(
        "test_two_params",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

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
    bitstring.append_u32(get_function_id(
        b"test_empty_dynamic_array(uint16[])(uint16[])",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(0);

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

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for &i in &input_array {
        bitstring.append_u16(i);
    }

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);

    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

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

fn put_data_into_chain(bilder: BuilderData, data: Bitstring) -> BuilderData {
    let mut size = data.length_in_bits();
    let mut current_builder = bilder;

    while size != 0 {
        if BuilderData::bits_capacity() == current_builder.bits_used() {
            let mut temp_builder = BuilderData::new();
            temp_builder.append_reference(current_builder);

            current_builder = temp_builder;
        }

        let adding_bits = std::cmp::min(
            BuilderData::bits_capacity() - current_builder.bits_used(),
            size,
        );

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

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for &i in &input_array {
        array_data.append(&Bitstring::create(i.to_be_bytes().to_vec(), 64));
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder);

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

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for &i in &input_array {
        bitstring.append_u32(i.0);
        bitstring.append_bit(if i.1 { &Bit::One } else { &Bit::Zero });
    }

    let mut builder = BuilderData::new();

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();

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

    // u8
    bitstring.append_u8(18);

    // Vec<(u32, bool)>
    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array1.len() as u8);

    for i in &input_array1 {
        bitstring.append_u32(i.0);
        bitstring.append_bit(if i.1 { &Bit::One } else { &Bit::Zero });
    }

    // i16
    bitstring.append(&Bitstring::create((-290 as i16).to_be_bytes().to_vec(), 16));

    // data of input_array2 is used several times
    let mut array2_data = Bitstring::new();

    for i in &input_array2 {
        array2_data.append(&Bitstring::create(i.to_be_bytes().to_vec(), 64));
    }

    // &[i64] - in-cell data
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::Zero);

    // [Vec<i64>; 5]
    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);

    let mut chain_builder = BuilderData::new();
    let mut cell_data = Bitstring::new();

    for _i in 0..5 {
        let mut array_builder = BuilderData::new();

        array_builder = put_data_into_chain(array_builder, array2_data.clone());

        if BuilderData::references_capacity() == chain_builder.references_used() {
            let mut vec = vec![];
            cell_data.into_bitstring_with_completion_tag(&mut vec);
            chain_builder.append_bitstring(&vec).unwrap();

            cell_data.clear();

            let mut temp_builder = BuilderData::new();
            temp_builder.append_reference(chain_builder);
            chain_builder = temp_builder;
        }

        cell_data.append_bit(&Bit::Zero);
        cell_data.append_bit(&Bit::Zero);

        chain_builder.prepend_reference(array_builder);
    }

    bitstring.append(&cell_data);

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    chain_builder.append_bitstring(&vec).unwrap();

    // &[i64] - separate chain data
    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array2_data.clone());

    chain_builder.prepend_reference(array_builder);

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
fn test_arrays_of_dint_and_duint() {
    let input_array_int: Vec<Dint> = vec![
        Dint::from(0),
        Dint::from(1),
        Dint::from(-1),
        Dint::from(0x1234567890i64),
        Dint::from(-0x1234567890i64),
    ];

    let byte_array_int: Vec<u8> = vec![
        0x00, 0x01, 0x7F, 0x90, 0xF1, 0xD9, 0xA2, 0xA3, 0x02, 0xF0, 0x8E, 0xA6, 0xDD, 0xDC, 0x7D,
    ];

    let input_array_uint: Vec<Duint> = vec![
        Duint::from(0u32),
        Duint::from(1u32),
        Duint::from(0x1234567890u64),
    ];

    let byte_array_uint: Vec<u8> = vec![0x00, 0x01, 0x90, 0xF1, 0xD9, 0xA2, 0xA3, 0x02];

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_arrays_of_dint_and_duint(dint[],duint[])(dint[],duint[])",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array_int.len() as u8);

    for i in byte_array_int {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 8));
    }

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array_uint.len() as u8);

    for i in byte_array_uint {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 8));
    }

    let mut builder = BuilderData::new();

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();

    let expected_tree = builder.into();

    let values = vec![
        TokenValue::Array(
            input_array_int
                .iter()
                .map(|i| TokenValue::Dint(i.clone()))
                .collect(),
        ),
        TokenValue::Array(
            input_array_uint
                .iter()
                .map(|i| TokenValue::Duint(i.clone()))
                .collect(),
        ),
    ];

    test_parameters_set(
        "test_arrays_of_dint_and_duint",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_small_bitstring() {
    let byte_array: Vec<u8> = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];

    let input_bitstring = Bitstring::create(byte_array.clone(), byte_array.len() * 8);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_small_bitstring(bitstring)(bitstring)",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_bitstring.length_in_bits() as u8);

    bitstring.append(&input_bitstring);

    let mut builder = BuilderData::new();

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();

    let expected_tree = builder.into();

    let values = vec![TokenValue::Bitstring(input_bitstring)];

    test_parameters_set(
        "test_small_bitstring",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_big_bitstring() {
    let mut byte_array: Vec<u8> = Vec::new();

    for i in 0..33 {
        byte_array.push(i as u8);
    }

    let input_bitstring = Bitstring::create(byte_array.clone(), byte_array.len() * 8);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_big_bitstring(bitstring)(bitstring)"));

    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::Zero);

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, input_bitstring.clone());

    let mut builder = BuilderData::new();

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();

    builder.append_reference(array_builder);

    let expected_tree = builder.into();

    let values = vec![TokenValue::Bitstring(input_bitstring)];

    test_parameters_set(
        "test_big_bitstring",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_small_bits() {
    let mut bits = [Bit::Zero; 982];

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits[i] = Bit::One;
        }
    }

    let mut data = Bitstring::new();

    data.append_bit(&Bit::One);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_bit(&bits[i]);
    }

    data.append(&array_data);

    let mut root_builder = BuilderData::new();

    root_builder = put_data_into_chain(root_builder, data);

    // ABI version and function ID can't be splitted to several cells so create new root cell for them
    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_small_bits(bits982)(bits982)"));

    let mut new_builder = BuilderData::new();
    new_builder.append_reference(root_builder);
    root_builder = new_builder;

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    let expected_tree = root_builder.into();

    let values = vec![TokenValue::Bits(array_data)];

    test_parameters_set(
        "test_small_bits",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_big_bits() {
    let mut bits = [Bit::Zero; 1024];

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits[i] = Bit::One;
        }
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_big_bits(bits1024)(bits1024)"));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_bit(&bits[i]);
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data.clone());

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder.clone());

    let expected_tree = root_builder.into();

    let values = vec![TokenValue::Bits(array_data)];

    test_parameters_set(
        "test_big_bits",
        &tokens_from_values(values),
        None,
        expected_tree,
    );
}

#[test]
fn test_reserving_reference() {
    let mut bits = [Bit::Zero; 1024];

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits[i] = Bit::One;
        }
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(
        b"test_reserving_reference(bits1024[4])(bits1024[4])",
    ));

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_bit(&bits[i]);
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data.clone());

    let mut root_builder = BuilderData::new();

    for _ in 0..4 {
        root_builder.append_reference(array_builder.clone());
    }
    root_builder.append_raw(&[0x80, 0x00], 10).unwrap(); // array of 4 arrays in separate cells

    let mut new_builder = BuilderData::new();
    new_builder.append_reference(root_builder);
    root_builder = new_builder;

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    let expected_tree: SliceData = root_builder.into();

    let values = vec![TokenValue::FixedArray(vec![
        TokenValue::Bits(array_data.clone()),
        TokenValue::Bits(array_data.clone()),
        TokenValue::Bits(array_data.clone()),
        TokenValue::Bits(array_data.clone()),
    ])];

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
        signed: true,
        id: 0
    };

    let pair = Keypair::generate::<Sha512, _>(&mut rand::rngs::OsRng::new().unwrap());

    let signed_test_tree = signed_function.encode_input(&tokens, Some(&pair)).unwrap();
    let mut signed_test_tree = SliceData::from(signed_test_tree);

    let mut signature = SliceData::from(signed_test_tree.checked_drain_reference().unwrap());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&signed_test_tree))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature)
        .unwrap();

    assert_eq!(expected_tree, signed_test_tree);
}
