use super::context::{from_hex_to_utf8_str};
use serde_json::Value;

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct DeBotInfo {
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub name: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub version: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub publisher: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub key: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub author: String,
    #[serde(deserialize_with = "validate_ton_address")]
    pub support: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub hello: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub language: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub dabi: String,
}

impl DeBotInfo {
    pub fn validate(&self) -> Result<, String> {

    }
}