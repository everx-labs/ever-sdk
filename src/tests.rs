use crate::abi_call::{ABICall, ABI_VERSION};
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

#[test]
fn test_one_input_and_output() {
    let message = ABICall::<(u128,), (bool,)>::encode_function_call("test_one_input_and_output".to_string(), 
                            (1123,));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


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

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());

    let expected_tree = SliceData::new(vec![0x00, 0x87, 0x98, 0x73, 0xe1, 0xFF, 0xFF, 0xFF, 0x75, 0x0C, 0xE4, 0x7B, 0xAC, 0x80]);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_empty_params() {
    let message = ABICall::<((),), ((),)>::encode_function_call("test_empty_params".to_string(), 
                            ((),));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_empty_params()()"));


    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}
/*
#[test]
fn test_two_params() {
    let message = ABICall::<(bool, i32), (u8, u64)>::encode_function_call("test_two_params".to_string(), 
                            (true, 9434567));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_two_params(int32,bool)(uint8,uint64)"));
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
    let message = ABICall::<(bool,(i8, i16, (i32, i64, i128)), (u8, u16, (u32, u64, u128))), ((),)>::encode_function_call("test_nested_tuples_with_all_simples".to_string(), 
                            (false, (-15, 9845, (-1, 12345678, -12345678)), (255, 0, (256, 123, 1234567890))));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_nested_tuples_with_all_simples(bool,(int8,int16,(int32,int64,int128)),(uint8,uint16,(uint32,uint64,uint128))),()"));
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
    let message = ABICall::<([u32;8],), ((),)>::encode_function_call("test_small_static_array".to_string(), 
                            (input_array,));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_small_static_array(uint32[8]),()"));

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
    let message = ABICall::<([u16;5],), ((),)>::encode_function_call("test_small_static_array_by_data".to_string(), 
                            (input_array,));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());

    let expected_tree = SliceData::new(vec![0x00, 0xbb, 0xa6, 0x67, 0xed, 0x00, 0x05, 0x00, 0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x01, 0x80]);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_small_dynamic_array() {
    let input_array: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let message = ABICall::<(Vec<u16>,), ((),)>::encode_function_call("test_small_dynamic_array".to_string(), 
                            (input_array,));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_small_dynamic_array(uint16[]),()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_u8(input_array.len() as u8);
    
    for &i in &input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 16));
    }

    let mut builder = BuilderData::new();
    builder.append_data(&bitstring);

    let root_cell = Arc::<CellData>::from(&builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}

#[test]
fn test_big_static_array() {
    let mut input_array: [u128;32] = [0; 32];
    for i in 0..32 {
        input_array[i] = i as u128;
    }
    let message = ABICall::<([u128;32],), ((),)>::encode_function_call("test_big_static_array".to_string(), 
                            (input_array,));

    let mut data_cur = Cursor::new(message);
    let restored = deserialize_cells_tree(&mut data_cur).unwrap();
    let test_tree = SliceData::from(restored[0].clone());


    let mut bitstring = Bitstring::new();

    bitstring.append_u8(ABI_VERSION);
    bitstring.append_u32(get_function_id("test_big_static_array(uint128[32]),()"));

    bitstring.append_bit(&Bit::One);
    bitstring.append_bit(&Bit::Zero);
    
    for &i in &input_array {
        bitstring.append(&Bitstring::create(i.to_be_bytes().to_vec(), 128));
    }

    let size = input_array.len() * 128;
    let mut offset = 0;
    let mut builders = Vec::<BuilderData>::new();
    let mut current_builder = BuilderData::new();

    while size != offset {
        if current_builder.bits_capacity() == current_builder.bits_used()
        {
            builders.push(current_builder);
            current_builder = BuilderData::new();
        }

        println!("size = {}, offset = {}", size, offset);
        println!("bits_capacity = {}, bits_used = {}", current_builder.bits_capacity(), current_builder.bits_used());

        let adding_bits = std::cmp::min(current_builder.bits_capacity() - current_builder.bits_used(), size - offset);

        let bits = bitstring.bits(offset..(offset + adding_bits));

        let mut temp_bitstring = Bitstring::new();
        for bit in bits.data {
            temp_bitstring.append_bit(&bit);
        }

        current_builder.append_data(&temp_bitstring);

        offset += adding_bits;
    }

    while builders.len() != 0 {
        let temp_builder = current_builder;
        current_builder = builders.pop().unwrap();

        current_builder.append_reference(temp_builder);
    }
    
    let root_cell = Arc::<CellData>::from(&current_builder);
	let expected_tree = SliceData::from(root_cell);

    assert_eq!(test_tree, expected_tree);
}
*/
