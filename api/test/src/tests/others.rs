use serde_derive::{Deserialize, Serialize};
use crate::tests::{reflect, reflect_module};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId(String);
pub type BlockId = StringId;

#[derive(Serialize, Deserialize, ApiType)]
pub enum EnumConsts {
    Foo = 2,
    Bar,
}

#[derive(Serialize, Deserialize, ApiType)]
pub enum EnumTypes {
    Foo(String, String),
    Bar(u32),
    Baz { a: String, b: String },
}

/// Foo
///
/// Foo struct
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct Foo {
    pub address: Option<String>,
    #[serde(default)]
    pub message_id: String,
    pub message_body_base64: String,
    pub expire: Option<u32>,
    last_block_id: BlockId,
    sending_time: u32,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
struct Bar {
    /// summary
    ///
    /// description
    pub abi: Option<serde_json::Value>,
    pub function_name: Option<String>,
    pub message: Foo,
    #[serde(default)]
    pub infinite_wait: bool,
    pub ids: Vec<String>,
    pub arr: [u64; 2],
}

#[derive(Serialize, Deserialize, ApiType)]
struct FooHandle(u32);

#[derive(ApiModule)]
#[api_module(name = "module")]
struct Module;

/// This is baz function
#[api_function]
fn _foo(_params: Foo) -> Result<Bar, Foo> {
    Ok(Bar::default())
}

#[test]
fn test_dev() {
    let _ = Module {};
    reflect_module::<Module>();
    reflect::<Foo>();
    reflect::<Bar>();
    reflect::<EnumConsts>();
    reflect::<EnumTypes>();
    reflect::<FooHandle>();
    println!("{}", serde_json::to_string_pretty(&_foo_api()).unwrap());
}

