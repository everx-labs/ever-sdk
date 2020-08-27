use opendoc::reflect::TypeInfo;
use opendoc_derive::{TypeInfo, method_info};
use opendoc;
use serde_derive::{Serialize, Deserialize};
use opendoc::api::{Type, Method};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId (String);
pub type BlockId = StringId;

#[doc(summary = "Foo")]
/// Foo struct
#[derive(Serialize, Deserialize, TypeInfo, Default)]
pub struct Foo {
    pub address: Option<String>,
    pub message_id: String,
    pub message_body_base64: String,
    pub expire: Option<u32>,
    last_block_id: BlockId,
    sending_time: u32,
}

#[derive(Serialize, Deserialize, TypeInfo, Default)]
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

#[method_info(name = "module.baz")]
fn baz(_params: Foo) -> Bar {
    Bar::default()
}

fn reflect<T: TypeInfo>() {
    let info = serde_json::to_string_pretty(&T::type_info()).unwrap();
    println!("{}", info);

}

fn main() {
    reflect::<Foo>();
    reflect::<Bar>();
    println!("{:?}", baz_method());

    let _ = baz(Foo::default());
}
