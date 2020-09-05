use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub enum Abi {
    Value(Value),
    Handle(AbiHandle)
}
