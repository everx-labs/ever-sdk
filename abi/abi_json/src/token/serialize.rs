use ton_abi_core::types::{
    get_fixed_array_in_cell_size, prepend_fixed_array, ABISerialized,
};
use super::*;

use tvm::stack::{BuilderData, IBitstring};
use tvm::block::Serializable;

impl ABISerialized for TokenValue {
    fn prepend_to(&self, mut destination: BuilderData) -> BuilderData {
        match self {
            TokenValue::Uint(uint) => uint.prepend_to(destination),
            TokenValue::Int(int) => int.prepend_to(destination),
            TokenValue::Dint(dint) => dint.prepend_to(destination),
            TokenValue::Duint(duint) => duint.prepend_to(destination),
            TokenValue::Bool(b) => b.prepend_to(destination),
            TokenValue::Tuple(ref tokens) => {
                let mut destination = destination;
                for token in tokens.iter().rev() {
                    destination = token.value.prepend_to(destination);
                }
                destination
            }
            TokenValue::Array(ref tokens) => tokens.prepend_to(destination),
            TokenValue::FixedArray(ref tokens) => prepend_fixed_array(destination, &tokens),
            TokenValue::Bits(b) => {
                prepend_fixed_array(destination, &b.bits(0..b.length_in_bits()).data)
            }
            TokenValue::Bitstring(bitstring) => bitstring.prepend_to(destination),
            TokenValue::Map(_key_type, _values) => {
                unimplemented!()
            }
            TokenValue::Address(address) => {
                let builder = address.write_to_new_cell().unwrap();
                destination.prepend_builder(&builder).unwrap();
                destination
            }
        }
    }

    fn get_in_cell_size(&self) -> usize {
        match self {
            TokenValue::Uint(uint) => uint.size,
            TokenValue::Int(int) => int.size,
            TokenValue::Dint(dint) => dint.get_in_cell_size(),
            TokenValue::Duint(duint) => duint.get_in_cell_size(),
            TokenValue::Bool(_) => 1,
            TokenValue::Tuple(ref tokens) => tokens
                .iter()
                .fold(0usize, |size, token| size + token.value.get_in_cell_size()),
            TokenValue::Array(ref tokens) => tokens.get_in_cell_size(),
            TokenValue::FixedArray(ref tokens) => get_fixed_array_in_cell_size(&tokens),
            TokenValue::Bits(b) => {
                get_fixed_array_in_cell_size(&b.bits(0..b.length_in_bits()).data)
            }
            TokenValue::Bitstring(bitstring) => bitstring.get_in_cell_size(),
            TokenValue::Map(_, _) => 1,
            TokenValue::Address(addr) => addr.write_to_new_cell().unwrap().length_in_bits(),
        }
    }
}