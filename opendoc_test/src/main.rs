use opendoc::reflect::TypeInfo;
use opendoc_derive::TypeInfo;
use opendoc;
use serde_derive::{Serialize, Deserialize};
use opendoc::api::Method;

#[doc(summary = "Enc mes")]
/// Encoded message
#[derive(Serialize, Deserialize, TypeInfo)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncodedMessage {
    pub address: Option<String>,
    pub message_id: String,
    pub message_body_base64: String,
    pub expire: Option<u32>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId (String);
pub type BlockId = StringId;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypeInfo)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessingState {
    last_block_id: BlockId,
    sending_time: u32,
}

#[derive(Serialize, Deserialize, TypeInfo)]
struct Foo {
    #[doc(summary = "summary")]
    #[doc = "description"]
    pub abi: Option<serde_json::Value>,
    pub function_name: Option<String>,
    pub message: EncodedMessage,
    pub message_processing_state: MessageProcessingState,
    #[serde(default)]
    pub infinite_wait: bool,
    pub ids: Vec<String>,
}

fn reflect<T: TypeInfo>() {
    let info = serde_json::to_string_pretty(&T::type_info()).unwrap();
    println!("{}", info);

}

fn main() {
    reflect::<EncodedMessage>();
    reflect::<MessageProcessingState>();
    reflect::<Foo>();

    println!("{}", serde_json::to_string_pretty(&Method::from_types::<MessageProcessingState, EncodedMessage>("meth")).unwrap());
}
