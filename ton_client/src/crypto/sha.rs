use sha2::{Digest};

pub fn sha256(bytes: &Vec<u8>) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}

pub fn sha512(bytes: &Vec<u8>) -> Vec<u8> {
    let mut hasher = sha2::Sha512::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}
