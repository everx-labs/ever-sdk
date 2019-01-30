use crate::abi_call::{ABICall, ABI_VERSION};
use crate::types::common::prepend_reference;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::sync::Arc;
use std::io::{Cursor};
use tonlabs_sdk_emulator::stack::{BuilderData, SliceData, CellData};
use tonlabs_sdk_emulator::cells_serialization::{deserialize_cells_tree};
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};

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

#[test]
fn test_one_input_and_output() {
    let message = ABICall::<(u128,), (bool,)>::encode_function_call("test_one_input_and_output".to_string(), 
                            (1123,));
    let test_tree = deserialize(message);


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_one_input_and_output(uint128)(bool)"));
    bitstring.append_u128(1123);


    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_one_input_and_output_by_data() {
    let message = ABICall::<(i64,), (u8,)>::encode_function_call("test_one_input_and_output_by_data".to_string(), 
                            (-596784153684,));
    let test_tree = deserialize(message);

    let expected_tree = SliceData::new(vec![0x00, 0x87, 0x98, 0x73, 0xe1, 0xFF, 0xFF, 0xFF, 0x75, 0x0C, 0xE4, 0x7B, 0xAC, 0x80]);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_empty_params() {
    let message = ABICall::<(), ()>::encode_function_call("test_empty_params".to_string(), 
                            ());
    let test_tree = deserialize(message);


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_empty_params()()"));


    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_two_params() {
    let message = ABICall::<(bool, i32), (u8, u64)>::encode_function_call("test_two_params".to_string(), 
                            (true, 9434567));
    let test_tree = deserialize(message);


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_two_params(bool,int32)(uint8,uint64)"));
    bitstring.append_bit(&Bit::One);
    bitstring.append_i32(9434567);


    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_nested_tuples_with_all_simples() {
    let message = ABICall::<(bool,(i8, i16, (i32, i64, i128)), (u8, u16, (u32, u64, u128))), ()>::encode_function_call("test_nested_tuples_with_all_simples".to_string(), 
                            (false, (-15, 9845, (-1, 12345678, -12345678)), (255, 0, (256, 123, 1234567890))));
    let test_tree = deserialize(message);


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128)))()"));
    bitstring.append_bit(&Bit::Zero);
    bitstring.append(&Bitstring::create((-15 as i8).to_be_bytes().to_vec(), 8));
    bitstring.append(&Bitstring::create((9845 as i16).to_be_bytes().to_vec(), 16));
    bitstring.append(&Bitstring::create((-1 as i32).to_be_bytes().to_vec(), 32));
    bitstring.append(&Bitstring::create((12345678 as i64).to_be_bytes().to_vec(), 64));
    bitstring.append(&Bitstring::create((-12345678 as i128).to_be_bytes().to_vec(), 128));
    bitstring.append(&Bitstring::create((255 as u8).to_be_bytes().to_vec(), 8));
    bitstring.append(&Bitstring::create((0 as u16).to_be_bytes().to_vec(), 16));
    bitstring.append(&Bitstring::create((256 as u32).to_be_bytes().to_vec(), 32));
    bitstring.append(&Bitstring::create((123 as u64).to_be_bytes().to_vec(), 64));
    bitstring.append(&Bitstring::create((1234567890 as u128).to_be_bytes().to_vec(), 128));


    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_static_array() {
    let input_array: [u32;8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let message = ABICall::<([u32;8],), ()>::encode_function_call("test_small_static_array".to_string(), 
                            (input_array,));
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
    let input_array: [u16;5] = [5, 4, 3, 2, 1];
    let message = ABICall::<([u16;5],), ()>::encode_function_call("test_small_static_array_by_data".to_string(), 
                            (input_array,));
    let test_tree = deserialize(message);

    let expected_tree = SliceData::new(vec![0x00, 0xd5, 0x7a, 0x4d, 0xac, 0x80, 0x01, 0x40, 0x01, 0x00, 0x00, 0xc0, 0x00, 0x80, 0x00, 0x60]);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_dynamic_array() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let message = ABICall::<(Vec<u16>,), ()>::encode_function_call("test_small_dynamic_array".to_string(), 
                            (input_array.clone(),));
    let test_tree_vec = deserialize(message);

    let message = ABICall::<(&[u16],), ()>::encode_function_call("test_small_dynamic_array".to_string(), 
                            (&input_array,));
    let test_tree_slice = deserialize(message);


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
    assert_eq!(test_tree_slice, expected_tree);
}

fn put_data_into_chain(bilder: BuilderData, data: Bitstring) -> BuilderData{
    let mut size = data.length_in_bits();
    let mut current_builder = bilder;

    while size != 0 {
        if current_builder.bits_capacity() == current_builder.bits_used()
        {
            let mut temp_builder = BuilderData::new(); 
            temp_builder.append_reference(current_builder);

            current_builder = temp_builder;
        }

        let adding_bits = std::cmp::min(current_builder.bits_capacity() - current_builder.bits_used(), size);

        let mut cut = Bitstring::new();
        data.bits(size - adding_bits .. size).data.iter().for_each(|x| { cut.append_bit(x); });
        current_builder.append_data(&cut);

        size -= adding_bits;
    }

    current_builder
}

#[test]
fn test_big_static_array() {
    let mut input_array: [u128;32] = [0; 32];
    for i in 0..32 {
        input_array[i] = i as u128;
    }
    let message = ABICall::<([u128;32],), ()>::encode_function_call("test_big_static_array".to_string(), 
                            (input_array,));
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

    let message = ABICall::<(Vec<i64>,), ()>::encode_function_call("test_big_dynamic_array".to_string(), 
                            (input_array.clone(),));
    let test_tree_vec = deserialize(message);

    let message = ABICall::<(&[i64],), ()>::encode_function_call("test_big_dynamic_array".to_string(), 
                            (&input_array,));
    let test_tree_slice = deserialize(message.clone());


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
    assert_eq!(test_tree_slice, expected_tree);
}

#[test]
fn test_dynamic_array_of_tuples() {
    let input_array: Vec<(u32, bool)> = vec![(1, true), (2, false), (3, true), (4, false), (5, true)];
    let message = ABICall::<(Vec<(u32, bool)>,), ()>::encode_function_call("test_dynamic_array_of_tuples".to_string(), 
                            (input_array.clone(),));
    let test_tree_vec = deserialize(message);

    let message = ABICall::<(&[(u32, bool)],), ()>::encode_function_call("test_dynamic_array_of_tuples".to_string(), 
                            (&input_array,));
    let test_tree_slice = deserialize(message);


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_dynamic_array_of_tuples((uint32,bool)[])()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);
    
    for i in input_array {
        bitstring.append(&Bitstring::create(i.0.to_be_bytes().to_vec(), 32));
        bitstring.append_bit(if i.1 {&Bit::One} else {&Bit::Zero});
    }

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree_vec, expected_tree); 
    assert_eq!(test_tree_slice, expected_tree);
}

#[test]
fn test_tuples_with_combined_types() {
    let input_array1: Vec<(u32, bool)> = vec![(1, true), (2, false), (3, true), (4, false)];
    
    let mut input_array2 = Vec::<i64>::new();
    for i in 0..73 {
        input_array2.push(i * i as i64);
    }

    let input_array3: [Vec<i64>; 5] = [input_array2.clone(), input_array2.clone(), input_array2.clone(), input_array2.clone(), input_array2.clone()];

    let message = ABICall::<(u8, (Vec<(u32, bool)>, i16), (&[i64], [Vec<i64>; 5])), ()>::encode_function_call("test_tuples_with_combined_types".to_string(), 
                            (18, (input_array1.clone(), -290), (input_array2.as_slice(), input_array3.clone())));
    let test_tree = deserialize(message.clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_tuples_with_combined_types(uint8,((uint32,bool)[],int16),(int64[],int64[][5]))()"));

    // u8
    bitstring.append_u8(18);

    // Vec<(u32, bool)>
    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array1.len() as u8);
    
    for i in input_array1 {
        bitstring.append(&Bitstring::create(i.0.to_be_bytes().to_vec(), 32));
        bitstring.append_bit(if i.1 {&Bit::One} else {&Bit::Zero});
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