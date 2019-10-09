use num_bigint::{BigInt, Sign};
use tvm::stack::BuilderData;
use ton_abi_core::types::{
    ABISerialized,
    prepend_data_to_chain,
    Bitstring
};

#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    pub number: BigInt,
    pub size: usize,
}

impl Int {
    pub fn new(number: i128, size: usize) -> Self {
        Self { number: BigInt::from(number), size }
    }
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

            let mut bitstring = Bitstring::create(vec_padding, dif);
            bitstring.append(&Bitstring::create(vec, self.size - dif));
            bitstring
        } else {
            let offset = vec_bits_length - self.size;
            Bitstring::create(vec, vec_bits_length).substring(offset..)
        };

        prepend_data_to_chain(destination, bitstring)
    }

    fn get_in_cell_size(&self) -> usize {
        self.size
    }
}
