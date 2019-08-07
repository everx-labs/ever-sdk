#![allow(non_camel_case_types)]

use crate::abi_call::{ABICall, ABI_VERSION};
use crate::abi_response::{ABIResponse};
use crate::types::{
    ABISerialized,
    ABIDeserialized,
    ABIInParameter,
    ABIOutParameter,
    ABITypeSignature};
use crate::types::{Dint, Duint, Bit, Bitstring};

use sha2::{Digest, Sha256, Sha512};
use ed25519_dalek::*;
use rand::rngs::OsRng;
use std::io::Cursor;
use std::sync::Arc;

use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use tvm::stack::{BuilderData, SliceData, CellData, IBitstring};

fn get_function_id(signature: &[u8]) -> u32 {
    // Sha256 hash of signature
    let mut hasher = Sha256::new();

    hasher.input(signature);

    let function_hash = hasher.result();

    let mut bytes = [0; 4];
    bytes.copy_from_slice(&function_hash[..4]);

    u32::from_be_bytes(bytes)
}

fn deserialize(message: Vec<u8>) -> BuilderData {
    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    BuilderData::from(&restored[0])
}

fn test_parameters_set<I, O>(func_name: &str, input: I, expected_tree: BuilderData, expected_decode: I::Out) 
    where
        I: std::fmt::Debug + std::cmp::PartialEq + ABIInParameter + ABISerialized + ABIDeserialized + ABITypeSignature + Clone,
        I::Out: ABIOutParameter + std::fmt::Debug + std::cmp::PartialEq + Clone,
        (u8, u32, I::Out): ABIOutParameter,
        O: ABIOutParameter + ABITypeSignature,
{
    let mut expected_tree_with_ref = expected_tree.clone();
    expected_tree_with_ref.prepend_reference(BuilderData::new());

    let message = ABICall::<I, O>::encode_function_call(func_name, input.clone());
    let test_tree = deserialize(message.clone());

    assert_eq!(test_tree, expected_tree_with_ref);

    let message_tree = ABICall::<I, O>::encode_function_call_into_slice(func_name, input.clone());

    assert_eq!(message_tree, expected_tree_with_ref);


    // check signing

    let pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());

    let message = ABICall::<I, O>::encode_signed_function_call(func_name, input.clone(), &pair);
    let mut message = SliceData::from(deserialize(message.clone()));

    let mut signature = SliceData::from(message.drain_reference());

    assert_eq!(SliceData::from(expected_tree), message);

    let signature_data = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();


    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature_data).unwrap();

    let public_key = signature.get_next_bytes(32).unwrap();
    assert_eq!(public_key, pair.public.to_bytes());


    // check output decoding

    let mut test_tree = SliceData::from(test_tree);
    test_tree.drain_reference();
    let test_tree_copy = test_tree.clone();

    let version = test_tree.get_next_byte().unwrap();
    let function_id = test_tree.get_next_u32().unwrap();

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&BuilderData::from_slice(&test_tree_copy)))
        .write_to(&mut data, false)
        .unwrap();

    // we can't easily remove some data from the beginning of SliceData, so decode the whole input and
    // add version and finction ID to expected decoded parameters
    let test_decode: (u8, u32, I::Out) = ABIResponse::<(u8, u32, I)>::decode_response(&data).unwrap();

    assert_eq!(test_decode, (version, function_id, expected_decode.clone()));


    let test_decode: (u8, u32, I::Out) = ABIResponse::<(u8, u32, I)>::decode_response_from_slice(test_tree_copy).unwrap();

    assert_eq!(test_decode, (version, function_id, expected_decode));
}

#[test]
fn test_one_input_and_output() {

    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder.append_u32(get_function_id(b"test_one_input_and_output(uint128)(bool)")).unwrap();
    builder.append_u128(1123).unwrap();


    let expected_tree = builder.into();

    test_parameters_set::<(u128,), (bool,)>("test_one_input_and_output", (1123,), expected_tree, (1123,));
}

#[test]
fn test_one_input_and_output_by_data() {
    let expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0x87, 0x98, 0x73, 0xe1, 0xFF, 0xFF, 0xFF, 0x75, 0x0C, 0xE4, 0x7B, 0xAC, 0x80,
    ]);

    test_parameters_set::<(i64,), (u8,)>("test_one_input_and_output_by_data", (-596784153684,), expected_tree, (-596784153684,));
}

#[test]
fn test_empty_params() {
    // function test_parameters_set makes a liitle trick with decoding output parameters (see comment there)
    // and empty type () can't be used inside of complex types, so we can't use test_parameters_set for
    // testing () and test the only type in this way

    let message = ABICall::<(), ()>::encode_function_call("test_empty_params", ());
    let test_tree = deserialize(message);

    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder.append_u32(get_function_id(b"test_empty_params()()")).unwrap();

    builder.prepend_reference(BuilderData::new());

    let expected_tree = builder.into();

    assert_eq!(test_tree, expected_tree);


    let builder = BuilderData::new();

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&builder))
        .write_to(&mut data, false)
        .unwrap();

    let test_decode = ABIResponse::<()>::decode_response(&data).unwrap();

    assert_eq!(test_decode, ());
}

#[test]
fn test_two_params() {
    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder.append_u32(get_function_id(b"test_two_params(bool,int32)(uint8,uint64)")).unwrap();
    builder.append_bit_one().unwrap();
    builder.append_i32(9434567).unwrap();


    let expected_tree = builder.into();

    let input_data = (true, 9434567);

    test_parameters_set::<(bool, i32), (u8, u64)>("test_two_params", input_data.clone(), expected_tree, input_data);
}

#[test]
fn test_nested_tuples_with_all_simples() {
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))()"));
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

    let input_data = (
        false,
        (-15, 9845, (-1, 12345678, -12345678)),
        (255, 0, (256, 123, 1234567890)),
    );

    test_parameters_set::<
        (
            bool,
            (i8, i16, (i32, i64, i128)),
            (u8, u16, (u32, u64, u128)),
        ),
        (),
    >("test_nested_tuples_with_all_simples", input_data.clone(), expected_tree, input_data);
}

fixed_abi_array!(u32, 8, Array_u32_8);

#[test]
fn test_small_static_array() {
    let input_array: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
  
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_small_static_array(uint32[8])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);

    for &i in &input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 32));
    }

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();


    let expected_tree = builder.into();

    let input_data = (Array_u32_8::from(input_array),);
    let expected_output = (input_array.to_vec(),);

    test_parameters_set::<
        (Array_u32_8,),
        ()
    >("test_small_static_array", input_data, expected_tree, expected_output);
}

fixed_abi_array!(u16, 5, Array_u16_5);

#[test]
fn test_small_static_array_by_data() {
    let input_array: [u16; 5] = [5, 4, 3, 2, 1];

    let expected_tree = BuilderData::with_bitstring(vec![
        0x00, 0xd5, 0x7a, 0x4d, 0xac, 0x80, 0x01, 0x40, 0x01, 0x00, 0x00, 0xc0, 0x00, 0x80, 0x00,
        0x60,
    ]);
    
    let input_data = (Array_u16_5::from(input_array),);
    let expected_output = (input_array.to_vec(),);

    test_parameters_set::<
        (Array_u16_5,),
        ()
    >("test_small_static_array_by_data", input_data, expected_tree, expected_output);
}

#[test]
fn test_empty_dynamic_array() {
    let input_array = Vec::<u16>::new();

    let input_data = (input_array.clone(),);
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_small_dynamic_array(uint16[])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for i in input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 16));
    }

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();


    let expected_tree = builder.into();

    test_parameters_set::<
        (Vec<u16>,),
        ()
    >("test_small_dynamic_array", input_data, expected_tree, expected_output);
}

#[test]
fn test_small_dynamic_array() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let input_data = (input_array.clone(),);
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(b"test_small_dynamic_array(uint16[])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for i in input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 16));
    }

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();


    let expected_tree = builder.into();

    test_parameters_set::<
        (Vec<u16>,),
        ()
    >("test_small_dynamic_array", input_data, expected_tree, expected_output);
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

fixed_abi_array!(u128, 32, u128_array_32);

#[test]
fn test_big_static_array() {
    let mut input_array: [u128; 32] = [0; 32];
    for i in 0..32 {
        input_array[i] = i as u128;
    }

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_big_static_array(uint128[32])()"));

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


    let input_data = (u128_array_32::from(input_array),);
    let expected_output = (input_array.to_vec(),);

    test_parameters_set::<
        (u128_array_32,),
        ()
    >("test_big_static_array", input_data, expected_tree, expected_output);
}

fixed_abi_array!(i32, 512, i32_array_512);

#[test]
fn test_huge_static_array() {
    let mut input_array: [i32; 512] = [0; 512];
    for i in 0..input_array.len() {
        input_array[i] = i as i32;
    }

    // since all standard operations are defined only for arrays with up to 32 elements we have to check
    // this huge array explicitly
    let message = ABICall::<(i32_array_512,), ()>::encode_function_call("test_huge_static_array", (input_array.into(),));
    let test_tree = deserialize(message);

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_huge_static_array(int32[512])()"));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for i in 0..input_array.len() {
        array_data.append(&Bitstring::create(input_array[i].to_be_bytes().to_vec(), 32));
    }

    let mut array_builder = BuilderData::new();

    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder.clone());

    root_builder.prepend_reference(BuilderData::new());

    let expected_tree = root_builder.into();

    assert_eq!(test_tree, expected_tree);


    let mut root_builder = BuilderData::new();

    root_builder.append_bit_zero().unwrap();
    root_builder.append_bit_zero().unwrap();
    root_builder.append_reference(array_builder.clone());

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&root_builder))
        .write_to(&mut data, false)
        .unwrap();

    let (test_decode,) = ABIResponse::<(i32_array_512,)>::decode_response(&data).unwrap();

    assert_eq!(input_array.len(), test_decode.len());

    for i in 0..input_array.len() {
        assert_eq!(input_array[i], test_decode[i]);
    }
}

#[test]
fn test_big_dynamic_array() {
    let mut input_array = Vec::<i64>::new();
    for i in 0..73 {
        input_array.push(i * i as i64);
    }

    let input_data = (input_array.clone(),);
    let expected_output = input_data.clone();

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_big_dynamic_array(int64[])()"));

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

    test_parameters_set::<
        (Vec<i64>,),
        ()
    >("test_big_dynamic_array", input_data, expected_tree, expected_output);
}

#[test]
fn test_dynamic_array_of_tuples() {
    let input_array: Vec<(u32, bool)> =
        vec![(1, true), (2, false), (3, true), (4, false), (5, true)];

    let input_data = (input_array.clone(),);
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_dynamic_array_of_tuples((uint32,bool)[])()",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for i in input_array {
        bitstring.append(&Bitstring::create(i.0.to_be_bytes().to_vec(), 32));
        bitstring.append_bit(if i.1 { &Bit::One } else { &Bit::Zero });
    }

    let mut builder = BuilderData::new();
     
    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();


    let expected_tree = builder.into();

    test_parameters_set::<
        (Vec<(u32, bool)>,),
        ()
    >("test_dynamic_array_of_tuples", input_data, expected_tree, expected_output);
}

fixed_abi_array!(Vec<i64>, 5, Veci64_array_5);

#[test]
fn test_tuples_with_combined_types() {
    let input_array1: Vec<(u32, bool)> = vec![(1, true), (2, false), (3, true), (4, false)];

    let mut input_array2 = Vec::<i64>::new();
    for i in 0..73 {
        input_array2.push(i * i as i64);
    }

    let input_array3: [Vec<i64>; 5] = [
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
        input_array2.clone(),
    ];

    let input_data = (
        18,
        (input_array1.clone(), -290),
        (input_array2.clone(), Veci64_array_5::from(input_array3.clone())),
    );

    let expected_output = (
        18,
        (input_array1.clone(), -290),
        (input_array2.clone(), input_array3.to_vec()),
    );

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_tuples_with_combined_types(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))()",
    ));

    // u8
    bitstring.append_u8(18);

    // Vec<(u32, bool)>
    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array1.len() as u8);

    for i in input_array1 {
        bitstring.append(&Bitstring::create(i.0.to_be_bytes().to_vec(), 32));
        bitstring.append_bit(if i.1 { &Bit::One } else { &Bit::Zero });
    }

    // i16
    bitstring.append(&Bitstring::create((-290 as i16).to_be_bytes().to_vec(), 16));

    // data of input_array2 is used several times
    let mut array2_data = Bitstring::new();

    for i in input_array2 {
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

    test_parameters_set::<
        (
            u8,
            (
                Vec<(u32, bool)>,
                i16
            ),
            (
                Vec<i64>,
                Veci64_array_5
            )
        ),
        ()
    >("test_tuples_with_combined_types", input_data, expected_tree, expected_output);
}

#[test]
fn test_arrays_of_dint_and_duint() {
    let input_array_int: Vec<Dint> =
        vec![Dint::from(0), Dint::from(1), Dint::from(-1), Dint::from(0x1234567890i64), Dint::from(-0x1234567890i64)];

    let byte_array_int: Vec<u8> = 
        vec![0x00, 0x01, 0x7F, 0x90, 0xF1, 0xD9, 0xA2, 0xA3, 0x02, 0xF0, 0x8E, 0xA6, 0xDD, 0xDC, 0x7D];

    let input_array_uint: Vec<Duint> =
        vec![Duint::from(0u32), Duint::from(1u32), Duint::from(0x1234567890u64)];

    let byte_array_uint: Vec<u8> = 
        vec![0x00, 0x01, 0x90, 0xF1, 0xD9, 0xA2, 0xA3, 0x02];    

    let input_data = (input_array_int.clone(), input_array_uint.clone());
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_arrays_of_dint_and_duint(dint[],duint[])()",
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

    test_parameters_set::<
        (Vec<Dint>, Vec<Duint>),
        ()
    >("test_arrays_of_dint_and_duint", input_data, expected_tree, expected_output);
}

#[test]
fn test_small_bitstring() {
    let byte_array: Vec<u8> = 
        vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];

    let input_bitstring = Bitstring::create(byte_array.clone(), byte_array.len() * 8);

    let input_data = (input_bitstring.clone(), );
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_small_bitstring(bitstring)()",
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

    test_parameters_set::<
        (Bitstring,),
        ()
    >("test_small_bitstring", input_data, expected_tree, expected_output);
}

#[test]
fn test_big_bitstring() {
    let mut byte_array: Vec<u8> = Vec::new();

    for i in 0..33 {
        byte_array.push(i as u8);
    }

    let input_bitstring = Bitstring::create(byte_array.clone(), byte_array.len() * 8);

    let input_data = (input_bitstring.clone(), );
    let expected_output = input_data.clone();

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        b"test_big_bitstring(bitstring)()",
    ));

    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::Zero);
    
    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, input_bitstring);

    let mut builder = BuilderData::new();

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    builder.append_bitstring(&vec).unwrap();

    builder.append_reference(array_builder);


    let expected_tree = builder.into();

    test_parameters_set::<
        (Bitstring,),
        ()
    >("test_big_bitstring", input_data, expected_tree, expected_output);
}

fixed_abi_array!(Bit, 982, Bits982);

#[test]
fn test_small_bits() {
    let mut bits: Bits982 = [Bit::Zero; 982].into();

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits.data[i] = Bit::One;
        }        
    }

    let message = ABICall::<(Bits982,), ()>::encode_function_call("test_small_bits", (bits.clone(),));
    let test_tree = deserialize(message);

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
    data.append_u32(get_function_id(b"test_small_bits(bits982)()"));

    let mut new_builder = BuilderData::new();
    new_builder.append_reference(root_builder);
    root_builder = new_builder;

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.prepend_reference(BuilderData::new());

    let expected_tree = root_builder.into();

    assert_eq!(test_tree, expected_tree);


    let mut root_builder = BuilderData::new();

    root_builder.append_bit_one().unwrap();
    root_builder.append_bit_zero().unwrap();

    let mut vec = vec![];
    array_data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&root_builder))
        .write_to(&mut data, false)
        .unwrap();

    let (test_decode,) = ABIResponse::<(Bits982,)>::decode_response(&data).unwrap();

    assert_eq!(bits.len(), test_decode.len());

    for i in 0..bits.len(){
        assert_eq!(bits[i], test_decode[i]);
    }
}

fixed_abi_array!(Bit, 1024, Bits1024);

#[test]
fn test_big_bits() {
    let mut bits: Bits1024 = [Bit::Zero; 1024].into();

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits.data[i] = Bit::One;
        }        
    }

    let message = ABICall::<(Bits1024,), ()>::encode_function_call("test_big_bits", (bits.clone(),));
    let test_tree = deserialize(message);

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_big_bits(bits1024)()"));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_bit(&bits[i]);
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    root_builder.append_reference(array_builder.clone());

    root_builder.prepend_reference(BuilderData::new());

    let expected_tree = root_builder.into();

    assert_eq!(test_tree, expected_tree);


    let mut root_builder = BuilderData::new();

    root_builder.append_bit_zero().unwrap();
    root_builder.append_bit_zero().unwrap();
    root_builder.append_reference(array_builder.clone());

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&root_builder))
        .write_to(&mut data, false)
        .unwrap();

    let (test_decode,) = ABIResponse::<(Bits1024,)>::decode_response(&data).unwrap();

    assert_eq!(bits.len(), test_decode.len());

    for i in 0..bits.len(){
        assert_eq!(bits[i], test_decode[i]);
    }
}


mod decode_encoded {
    use super::*;

    fn validate<T>(input: T)
    where
        T: ABISerialized + ABIDeserialized,
        T::Out: std::fmt::Debug + std::cmp::PartialEq + From<T>,
    {
        let buffer = input.prepend_to(BuilderData::new());
        let slice = buffer.into();
        let (output, _) = <T>::read_from(slice).unwrap();
        assert_eq!(output, input.into());
    }

    #[test]
    fn boolean() {
        validate(true);
        validate(false);
    }

    #[test]
    fn tuples_with_booleans() {
        validate((true, false));
        validate((false, (true, true)));
    }

    #[test]
    fn tuples_with_ints() {
        validate((-1 as i128, 687 as u32));
        validate((8 as u16, (97 as i8, 328 as u64)));
    }

    #[test]
    fn dynamic_int() {
        let num = Dint::parse_bytes(b"b884d718567fd5fb9b0b54f2de27b5dad7c769f0024091230b7ca90c63af27035039d22b47dfc90e7e6661f435eb9e503c73ef62b803df9070af4e13366b55a795b9d862902703a9da29b71d391f93223b39fcd938a5860bfae17b7a56ccdb4ea0cd55da7c6b44d54dcc34b716455b073bf731c5547728b6a9abf7fd7d468ee7bd668f109a05625342dc67f0d295f90b6e7732b19eda0b920ea5ef51cbca25d8c8596706d93938dd4861652a53a68bca2e5082700df032272e46c471c22522d7257a8fa620f9a9e15ab72c5df0d8cd8db731064ebeadce25f04bb6ed42fb4d1b5c8e40c684eaa03ba1a2a0733e7fb9247edd20e16deab2ee095078dad3d50444", 16).unwrap();
        validate(num);
    }

    #[test]
    fn dynamic_array() {
        validate(vec![0u8, 1, 2, 3, 4]);

        let mut vec = Vec::<u64>::new();

        for i in 0..100 {
            vec.push(i);
        }
        validate(vec);
    }
}

#[test]
fn test_signed_one_input_and_output() {
    let pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());

    let func_name = "test_one_input_and_output";
    let message = ABICall::<(u128,), (bool,)>::encode_signed_function_call(func_name, (1979,), &pair);
    let mut message = SliceData::from(deserialize(message.clone()));

    let mut signature = SliceData::from(message.drain_reference());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature).unwrap();
}

fixed_abi_array!(Bits1024, 4, bits1024_array4);

#[test]
fn test_reserving_reference() {

    let mut bits: Bits1024 = [Bit::Zero; 1024].into();

    for i in 0..bits.len() {
        if i % 2 != 0 {
            bits.data[i] = Bit::One;
        }        
    }

    let input_data = [bits.clone(), bits.clone(), bits.clone(), bits.clone()];

    let pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());

    let func_name = "test_reserving_reference";
    let message = ABICall::<(bits1024_array4,), ()>::encode_signed_function_call(func_name, (input_data.into(),), &pair);
    let mut message = SliceData::from(deserialize(message.clone()));

    let mut signature = SliceData::from(message.drain_reference());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature).unwrap();


    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_reserving_reference(bits1024[4])()"));

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_bit(&bits[i]);
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    for _ in 0..4 {
        root_builder.append_reference(array_builder.clone());
    }
    root_builder.append_raw(&[0x80,0x00], 10).unwrap(); // array of 4 arrays in separate cells

    let mut new_builder = BuilderData::new();
    new_builder.append_reference(root_builder);
    root_builder = new_builder;

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    let expected_tree: SliceData = root_builder.into();

    assert_eq!(expected_tree, message);
}
