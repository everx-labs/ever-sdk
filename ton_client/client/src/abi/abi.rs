use serde_json::Value;

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, TypeInfo)]
pub enum Abi {
    Value(Value),
    Handle(AbiHandle)
}
