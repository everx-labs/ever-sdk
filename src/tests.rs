use crate::abi_call::{ABICall, ABI_VERSION};
use crate::abi_response::{ABIResponse};
use crate::types::common::prepend_reference;
use crate::types::{ABIParameter, ABIInParameter, ABIOutParameter};
use crate::types::dynamic_int::Dint;
use crate::types::dynamic_uint::Duint;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use num_bigint::{BigUint, BigInt, Sign};

use std::io::Cursor;
use std::sync::Arc;

use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use tonlabs_sdk_emulator::cells_serialization::{deserialize_cells_tree, BagOfCells};
use tonlabs_sdk_emulator::stack::{BuilderData, CellData, SliceData};

fn get_function_id(signature: &str) -> u32 {
    // Sha256 hash of signature
    let mut hasher = Sha256::new();

    hasher.input_str(&signature);

    let mut function_hash = [0 as u8; 32];
    hasher.result(&mut function_hash);

    let mut bytes = [0; 4];
    bytes.copy_from_slice(&function_hash[..4]);

    u32::from_be_bytes(bytes)
}

fn deserialize(message: Vec<u8>) -> SliceData {
    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    SliceData::from(restored[0].clone())
}

fn test_parameters_set<I, O>(func_name: &str, input: I, expected_tree: SliceData, expected_decode: I::Out) 
    where
        I: std::fmt::Debug + std::cmp::PartialEq + ABIInParameter + ABIParameter,
        I::Out: ABIOutParameter + std::fmt::Debug + std::cmp::PartialEq,
        (u8, u32, I::Out): ABIOutParameter,
        O: ABIInParameter + ABIOutParameter,
{
    let message = ABICall::<I, O>::encode_function_call(func_name, input);
    let mut test_tree = deserialize(message.clone());

    assert_eq!(test_tree, expected_tree);

    let version = test_tree.get_next_byte();
    let function_id = test_tree.get_next_u32();

    let mut data = Vec::new();
    BagOfCells::with_root(test_tree)
        .write_to(&mut data, false, 2, 2)
        .unwrap();

    // we can't easily remove some data from the beginning of SliceData, so decode the whole input and
    // add version and finction ID to expected decoded parameters
    let test_decode: (u8, u32, I::Out) = ABIResponse::<(u8, u32, I)>::decode_response(&data).unwrap();

    assert_eq!(test_decode, (version, function_id, expected_decode));
}

#[test]
fn test_one_input_and_output() {
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_one_input_and_output(uint128)(bool)"));
    bitstring.append_u128(1123);

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    test_parameters_set::<(u128,), (bool,)>("test_one_input_and_output", (1123,), expected_tree, (1123,));
}

#[test]
fn test_one_input_and_output_by_data() {
    let expected_tree = SliceData::new(vec![
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

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_empty_params()()"));

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);


    let builder = BuilderData::new();

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    let mut data = Vec::new();
    BagOfCells::with_root(expected_tree)
        .write_to(&mut data, false, 2, 2)
        .unwrap();

    let test_decode = ABIResponse::<()>::decode_response(&data).unwrap();

    assert_eq!(test_decode, ());
}

#[test]
fn test_two_params() {
    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_two_params(bool,int32)(uint8,uint64)"));
    bitstring.append_bit(&Bit::One);
    bitstring.append_i32(9434567);

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    let input_data = (true, 9434567);

    test_parameters_set::<(bool, i32), (u8, u64)>("test_two_params", input_data.clone(), expected_tree, input_data);
}

#[test]
fn test_nested_tuples_with_all_simples() {
    let message = ABICall::<
        (
            bool,
            (i8, i16, (i32, i64, i128)),
            (u8, u16, (u32, u64, u128)),
        ),
        (),
    >::encode_function_call(
        "test_nested_tuples_with_all_simples".to_string(),
        (
            false,
            (-15, 9845, (-1, 12345678, -12345678)),
            (255, 0, (256, 123, 1234567890)),
        ),
    );
    let test_tree = deserialize(message);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))()"));
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

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_static_array() {
    let input_array: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let message = ABICall::<([u32; 8],), ()>::encode_function_call(
        "test_small_static_array".to_string(),
        (input_array,),
    );
    let test_tree = deserialize(message);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_small_static_array(uint32[8])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);

    for &i in &input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 32));
    }

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_static_array_by_data() {
    let input_array: [u16; 5] = [5, 4, 3, 2, 1];
    let message = ABICall::<([u16; 5],), ()>::encode_function_call(
        "test_small_static_array_by_data".to_string(),
        (input_array,),
    );
    let test_tree = deserialize(message);

    let expected_tree = SliceData::new(vec![
        0x00, 0xd5, 0x7a, 0x4d, 0xac, 0x80, 0x01, 0x40, 0x01, 0x00, 0x00, 0xc0, 0x00, 0x80, 0x00,
        0x60,
    ]);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_dynamic_array() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let message = ABICall::<(Vec<u16>,), ()>::encode_function_call(
        "test_small_dynamic_array".to_string(),
        (input_array.clone(),),
    );
    let test_tree_vec = deserialize(message);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_small_dynamic_array(uint16[])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for i in input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 16));
    }

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree_vec, expected_tree);
}

fn put_data_into_chain(bilder: BuilderData, data: Bitstring) -> BuilderData {
    let mut size = data.length_in_bits();
    let mut current_builder = bilder;

    while size != 0 {
        if current_builder.bits_capacity() == current_builder.bits_used() {
            let mut temp_builder = BuilderData::new();
            temp_builder.append_reference(current_builder);

            current_builder = temp_builder;
        }

        let adding_bits = std::cmp::min(
            current_builder.bits_capacity() - current_builder.bits_used(),
            size,
        );

        let mut cut = Bitstring::new();
        data.bits(size - adding_bits..size)
            .data
            .iter()
            .for_each(|x| {
                cut.append_bit(x);
            });
        current_builder.append_data(&cut);

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
    let message = ABICall::<([u128; 32],), ()>::encode_function_call(
        "test_big_static_array".to_string(),
        (input_array,),
    );
    let test_tree = deserialize(message);

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id("test_big_static_array(uint128[32])()"));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for &i in &input_array {
        array_data.append(&Bitstring::create(i.to_be_bytes().to_vec(), 128));
    }

    let mut array_builder = BuilderData::new();

    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    root_builder.append_data(&data);
    root_builder.append_reference(array_builder);

    let root_cell = Arc::<CellData>::from(&root_builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_big_dynamic_array() {
    let mut input_array = Vec::<i64>::new();
    for i in 0..73 {
        input_array.push(i * i as i64);
    }

    let message = ABICall::<(Vec<i64>,), ()>::encode_function_call(
        "test_big_dynamic_array".to_string(),
        (input_array.clone(),),
    );
    let test_tree_vec = deserialize(message);

    let mut data = Bitstring::new();

    data.append_u8(ABI_VERSION);
    data.append_u32(get_function_id("test_big_dynamic_array(int64[])()"));

    data.append_bit(&Bit::Zero);
    data.append_bit(&Bit::Zero);

    let mut array_data = Bitstring::new();

    for &i in &input_array {
        array_data.append(&Bitstring::create(i.to_be_bytes().to_vec(), 64));
    }

    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array_data);

    let mut root_builder = BuilderData::new();

    root_builder.append_data(&data);
    root_builder.append_reference(array_builder);

    let root_cell = Arc::<CellData>::from(&root_builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree_vec, expected_tree);
}

#[test]
fn test_dynamic_array_of_tuples() {
    let input_array: Vec<(u32, bool)> =
        vec![(1, true), (2, false), (3, true), (4, false), (5, true)];
    let message = ABICall::<(Vec<(u32, bool)>,), ()>::encode_function_call(
        "test_dynamic_array_of_tuples".to_string(),
        (input_array.clone(),),
    );
    let test_tree_vec = deserialize(message);

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        "test_dynamic_array_of_tuples((uint32,bool)[])()",
    ));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);

    for i in input_array {
        bitstring.append(&Bitstring::create(i.0.to_be_bytes().to_vec(), 32));
        bitstring.append_bit(if i.1 { &Bit::One } else { &Bit::Zero });
    }

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree_vec, expected_tree);
}

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

    let message =
        ABICall::<(u8, (Vec<(u32, bool)>, i16), (Vec<i64>, [Vec<i64>; 5])), ()>::encode_function_call(
            "test_tuples_with_combined_types".to_string(),
            (
                18,
                (input_array1.clone(), -290),
                (input_array2.clone(), input_array3.clone()),
            ),
        );
    let test_tree = deserialize(message.clone());

    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id(
        "test_tuples_with_combined_types(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))()",
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

        if chain_builder.references_capacity() == chain_builder.references_used() {
            chain_builder.append_data(&cell_data);
            cell_data.clear();

            let mut temp_builder = BuilderData::new();
            temp_builder.append_reference(chain_builder);
            chain_builder = temp_builder;
        }

        cell_data.append_bit(&Bit::Zero);
        cell_data.append_bit(&Bit::Zero);

        prepend_reference(&mut chain_builder, array_builder);
    }

    bitstring.append(&cell_data);

    chain_builder.append_data(&bitstring);

    // &[i64] - separate chain data
    let mut array_builder = BuilderData::new();
    array_builder = put_data_into_chain(array_builder, array2_data.clone());

    prepend_reference(&mut chain_builder, array_builder);

    let root_cell = Arc::<CellData>::from(&chain_builder);
    let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

mod decode_encoded {
    use super::*;

    fn validate<T>(input: T)
    where
        T: ABIParameter,
        T::Out: std::fmt::Debug + std::cmp::PartialEq + From<T>,
    {
        let buffer = input.prepend_to(BuilderData::new());
        let slice = SliceData::from(Arc::new(buffer.cell().clone()));
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
        let num = BigInt::parse_bytes(b"b884d718567fd5fb9b0b54f2de27b5dad7c769f0024091230b7ca90c63af27035039d22b47dfc90e7e6661f435eb9e503c73ef62b803df9070af4e13366b55a795b9d862902703a9da29b71d391f93223b39fcd938a5860bfae17b7a56ccdb4ea0cd55da7c6b44d54dcc34b716455b073bf731c5547728b6a9abf7fd7d468ee7bd668f109a05625342dc67f0d295f90b6e7732b19eda0b920ea5ef51cbca25d8c8596706d93938dd4861652a53a68bca2e5082700df032272e46c471c22522d7257a8fa620f9a9e15ab72c5df0d8cd8db731064ebeadce25f04bb6ed42fb4d1b5c8e40c684eaa03ba1a2a0733e7fb9247edd20e16deab2ee095078dad3d50444", 16).unwrap();
        validate(Dint{data: num});
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
