use serde_json::Value;

pub struct AbiHandle(u32);

pub enum Abi {
    Abi(Value),
    Handle(AbiHandle)
}
