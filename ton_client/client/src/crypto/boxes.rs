use crate::crypto::keys::KeyPair;

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct SigningBoxHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub enum Signing {
    Keys(KeyPair),
    Box(SigningBoxHandle)
}

