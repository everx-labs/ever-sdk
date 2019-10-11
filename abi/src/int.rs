use num_bigint::{BigInt, BigUint};

#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    pub number: BigInt,
    pub size: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uint {
    pub number: BigUint,
    pub size: usize,
}


impl Int {
    pub fn new(number: i128, size: usize) -> Self {
        Self { number: BigInt::from(number), size }
    }
}


impl Uint {
    pub fn new(number: u128, size: usize) -> Self {
        Self { number: BigUint::from(number), size }
    }
}
