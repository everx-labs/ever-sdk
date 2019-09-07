extern crate rand;
use rand::RngCore;

pub fn generate_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::rngs::OsRng::new().unwrap();
    let mut result: Vec<u8> = Vec::new();
    result.resize(len, 0);
    rng.fill_bytes(&mut result);
    result
}

