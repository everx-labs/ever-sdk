use crate::crypto::keys::KeyPair;

pub struct SigningBoxHandle(u32);

pub enum Signing {
    Keys(KeyPair),
    Box(SigningBoxHandle)
}

