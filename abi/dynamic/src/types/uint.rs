use types::int::Int;

use num_bigint::{BigUint, BigInt, Sign};
use tvm::stack::BuilderData;
use abi_lib::types::ABISerialized;

#[derive(Clone, Debug, PartialEq)]
pub struct Uint {
    pub number: BigUint,
    pub size: usize,
}

impl ABISerialized for Uint {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let int = Int{
            number: BigInt::from_biguint(Sign::Plus, self.number.clone()),
            size: self.size};

        int.prepend_to(destination)
    }

    fn get_in_cell_size(&self) -> usize {
        self.size
    }
}
