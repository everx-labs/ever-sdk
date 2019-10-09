#![allow(non_camel_case_types)]

use crate::abi_call::{ABICall, ABI_VERSION};
use crate::abi_response::{ABIResponse};
use crate::types::{
    ABISerialized,
    ABIDeserialized,
    ABIInParameter,
    ABIOutParameter,
    ABITypeSignature};
use crate::types::{Bit, Bitstring};

use sha2::{Digest, Sha256, Sha512};
use ed25519_dalek::*;
use rand::rngs::OsRng;
use std::io::Cursor;
use std::sync::Arc;

use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use tvm::stack::{BuilderData, SliceData, CellData, IBitstring};
use tvm::stack::dictionary::{HashmapE, HashmapType};

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

fn test_parameters_set<I, O>(func_name: &str, input: I, expected_tree: BuilderData, expected_decode: I::Out) 
    where
        I: std::fmt::Debug + std::cmp::PartialEq + ABIInParameter + ABIOutParameter + ABITypeSignature + Clone,
        I::Out: std::fmt::Debug + std::cmp::PartialEq + Clone,
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

    let mut signature = SliceData::from(message.checked_drain_reference().unwrap());

    assert_eq!(SliceData::from(expected_tree), message);

    let signature_data = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();


    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature_data).unwrap();

    let public_key = signature.get_next_bytes(32).unwrap();
    assert_eq!(public_key, pair.public.to_bytes());


    // check output decoding

    let mut test_tree = SliceData::from(test_tree);
    test_tree.checked_drain_reference().unwrap();
    let test_tree_copy = test_tree.clone();

    let _version = test_tree.get_next_byte().unwrap();
    let function_id = test_tree.get_next_u32().unwrap();

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&BuilderData::from_slice(&test_tree_copy)))
        .write_to(&mut data, false)
        .unwrap();

    // we can't easily remove some data from the beginning of SliceData, so decode the whole input and
    // add version and finction ID to expected decoded parameters
    let test_decode: (u32, I::Out) = ABIResponse::<I>::decode_response(&data).unwrap();

    assert_eq!(test_decode, (function_id, expected_decode.clone()));


    let test_decode: (u32, I::Out) = ABIResponse::<I>::decode_response_from_slice(test_tree_copy).unwrap();

    assert_eq!(test_decode, (function_id, expected_decode));
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
    ]).unwrap();

    test_parameters_set::<(i64,), (u8,)>("test_one_input_and_output_by_data", (-596784153684,), expected_tree, (-596784153684,));
}

#[test]
fn test_empty_params() {
    // function test_parameters_set makes a liitle trick with decoding output parameters (see comment there)
    // and empty type () can't be used inside of complex types, so we can't use test_parameters_set for
    // testing () and test the only type in this way

    let message = ABICall::<(), ()>::encode_function_call("test_empty_params", ());
    let test_tree = deserialize(message);

    let func_id = get_function_id(b"test_empty_params()()");

    let mut builder = BuilderData::new();
    builder.append_u8(ABI_VERSION).unwrap();
    builder.append_u32(func_id).unwrap();

    builder.prepend_reference(BuilderData::new());

    let expected_tree = builder.clone().into();

    assert_eq!(test_tree, expected_tree);


    let mut slice = SliceData::from(builder);
    slice.checked_drain_reference().unwrap();

    let test_decode = ABIResponse::<()>::decode_response_from_slice(slice).unwrap();

    assert_eq!(test_decode, (func_id, ()));
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
    ]).unwrap();
    
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

    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::One);
    bitstring.append_u32(input_array.len() as u32);
    bitstring.append_bit(&Bit::Zero);

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


    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    
    let mut builder = BuilderData::new();
    builder.append_bitstring(&vec).unwrap();

    add_array_as_map(&mut builder, &input_array);

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
        if current_builder.bits_free() == 0 {
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

    let expected_tree = root_builder.clone().into();

    assert_eq!(test_tree, expected_tree);


    let mut slice = SliceData::from(root_builder);
    slice.checked_drain_reference().unwrap();

    let (_func_id, (test_decode,)) = ABIResponse::<(i32_array_512,)>::decode_response_from_slice(slice).unwrap();

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

    let mut root_builder = BuilderData::new();

    let mut vec = vec![];
    data.into_bitstring_with_completion_tag(&mut vec);
    root_builder.append_bitstring(&vec).unwrap();

    add_array_as_map(&mut root_builder, &input_array);

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

    test_parameters_set::<
        (Vec<(u32, bool)>,),
        ()
    >("test_dynamic_array_of_tuples", input_data, expected_tree, expected_output);
}

fixed_abi_array!(Vec<i64>, 5, Veci64_array_5);

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

    add_array_as_map(&mut chain_builder, &input_array3[0]);

    let mut second_builder = BuilderData::new();

    for i in 1..5 {
        add_array_as_map(&mut second_builder, &input_array3[i]);
    }

    chain_builder.append_reference(second_builder);

    second_builder = chain_builder;
    chain_builder = BuilderData::new();
    chain_builder.append_reference(second_builder);

    let mut vec = vec![];
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    chain_builder.append_bitstring(&vec).unwrap();


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

    let mut signature = SliceData::from(message.checked_drain_reference().unwrap());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature).unwrap();
}

fixed_abi_array!(u8, 128, Bits1024);
fixed_abi_array!(Bits1024, 4, bits1024_array4);

#[test]
fn test_reserving_reference() {

    let bits: Bits1024 = [0x55u8; 128].into();

    let input_data = [bits.clone(), bits.clone(), bits.clone(), bits.clone()];

    let pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());

    let func_name = "test_reserving_reference";
    let message = ABICall::<(bits1024_array4,), ()>::encode_signed_function_call(func_name, (input_data.into(),), &pair);
    let mut message = SliceData::from(deserialize(message.clone()));

    let mut signature = SliceData::from(message.checked_drain_reference().unwrap());
    let signature = Signature::from_bytes(signature.get_next_bytes(64).unwrap().as_slice()).unwrap();
    let bag_hash = (&Arc::<CellData>::from(&BuilderData::from_slice(&message))).repr_hash();
    pair.verify::<Sha512>(bag_hash.as_slice(), &signature).unwrap();


    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id(b"test_reserving_reference(bits1024[4])()"));

    let mut array_data = Bitstring::new();

    for i in 0..bits.len() {
        array_data.append_u8(bits[i]);
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
