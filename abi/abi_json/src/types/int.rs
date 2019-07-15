use num_bigint::{BigInt, Sign};
use tvm::stack::{BuilderData, SliceData, IBitstring};
use ton_abi_core::types::{
    ABISerialized,
    prepend_data_to_chain
};

#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    pub number: BigInt,
    pub size: usize,
}

impl ABISerialized for Int {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let vec = self.number.to_signed_bytes_be();
        let vec_bits_length = vec.len() * 8;

        let bitstring = if self.size > vec_bits_length {
            let padding = if self.number.sign() == Sign::Minus {
                0xFFu8
            } else {
                0u8
            };

            let dif = self.size - vec_bits_length;

            let mut vec_padding = Vec::new();
            vec_padding.resize(dif / 8 + 1, padding);

            let mut bitstring = BuilderData::with_raw(vec_padding, dif);
            bitstring.append_builder(&BuilderData::with_raw(vec, self.size - dif)).unwrap();
            bitstring
        } else {
            let offset = vec_bits_length - self.size;
            let builder = BuilderData::with_raw(vec, vec_bits_length);
            let mut slice: SliceData = builder.into();
            slice.shrink_data(offset..);
            BuilderData::from_slice(&slice)
        };

        prepend_data_to_chain(destination, bitstring)
    }

    fn get_in_cell_size(&self) -> usize {
        self.size
    }
}
