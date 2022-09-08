use crate::tests::{_enum, _number, _string, _struct};
use api_info::{ApiType, Field};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ApiType)]
#[serde(tag = "type")]
pub enum EnumTypesWithoutValue {
    Foo(String),
    Bar(u32),
}

#[test]
fn test_enum_without_value() {
    let api: Field = EnumTypesWithoutValue::api();
    _enum([
        ("Foo", _struct([("", _string())])),
        ("Bar", _struct([("", _number())])),
    ])
    .check(&api.value, "EnumTypesWithoutValue");
}

#[derive(Serialize, Deserialize, ApiType)]
#[serde(tag = "type", content = "value")]
pub enum EnumTypesWithValue {
    Foo(String),
    Bar(u32),
}

#[test]
fn test_enum_with_value() {
    let api: Field = EnumTypesWithValue::api();
    _enum([
        ("Foo", _struct([("value", _struct([("", _string())]))])),
        ("Bar", _struct([("value", _struct([("", _number())]))])),
    ])
    .check(&api.value, "EnumTypesWithValue");
}
