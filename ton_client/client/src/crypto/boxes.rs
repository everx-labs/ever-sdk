use crate::crypto::keys::KeyPair;

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct SigningBoxHandle(u32);

#[derive(Serialize, Deserialize, TypeInfo)]
pub enum Signing {
    None,
    Keys(KeyPair),
    Box(SigningBoxHandle)
}

