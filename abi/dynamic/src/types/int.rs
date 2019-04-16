use num_bigint::{BigInt, Sign};
use tvm::bitstring::Bitstring;
use tvm::stack::BuilderData;
use abi_lib::types::{
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

        let dif = self.size - vec.len() * 8;

        let bitstring = if dif > 0 {
            let padding = if self.number.sign() == Sign::Minus {
                0xFFu8
            } else {
                0u8
            };

            let mut vec_padding = Vec::new();
            vec_padding.resize(dif / 8 + 1, padding);

            let mut bitstring = Bitstring::create(vec_padding, dif);
            bitstring.append(&Bitstring::create(vec, self.size));
            bitstring
        } else {
            Bitstring::create(vec, self.size)
        };

        prepend_data_to_chain(destination, bitstring)
    }

    fn get_in_cell_size(&self) -> usize {
        self.size
    }
}
