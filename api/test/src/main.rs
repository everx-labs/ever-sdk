#[macro_use]
extern crate api_derive;

use api_info;

use serde_derive::{Deserialize, Serialize};
use api_info::{ApiType, ApiModule};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId(String);
pub type BlockId = StringId;

#[derive(Serialize, Deserialize, ApiType)]
pub enum EnumWithValues {
    Foo = 2,
    Bar,
}

#[derive(Serialize, Deserialize, ApiType)]
pub enum EnumWithTypes {
    Foo(String, String),
    Bar(u32),
    Baz { a: String, b: String },
}

#[doc(summary = "Foo")]
/// Foo struct
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct Foo {
    pub address: Option<String>,
    pub message_id: String,
    pub message_body_base64: String,
    pub expire: Option<u32>,
    last_block_id: BlockId,
    sending_time: u32,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
struct Bar {
    #[doc(summary = "summary")]
    #[doc = "description"]
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
#[api_module(name="module")]
struct Module;

/// This is baz function
#[api_function]
fn _foo(_params: Foo) -> Result<Bar, Foo> {
    Ok(Bar::default())
}

fn reflect<T: ApiType>() {
    let info = serde_json::to_string_pretty(&T::api()).unwrap();
    println!("{}", info);
}

fn reflect_module<T: ApiModule>() {
    let info = serde_json::to_string_pretty(&T::api()).unwrap();
    println!("{}", info);
}

fn main() {
    reflect_module::<Module>();
    reflect::<Foo>();
    // reflect::<Bar>();
    // reflect::<EnumWithValues>();
    // reflect::<EnumWithTypes>();
    // reflect::<FooHandle>();
    // println!("{}", serde_json::to_string_pretty(&_baz_info()).unwrap());
    // println!("{}", serde_json::to_string_pretty(&_foo_info()).unwrap());
}
