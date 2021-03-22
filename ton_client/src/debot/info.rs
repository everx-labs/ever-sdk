use super::context::{from_hex_to_utf8_str};
use serde::{de, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;
use crate::encoding::account_decode;

#[derive(Serialize, Deserialize, Default, Debug, Clone, ApiType, PartialEq)]
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
    #[serde(default)]
    pub interfaces: Vec<String>,
}

impl DeBotInfo {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

pub(super) fn validate_ton_address<'de, S, D>(des: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(des)?;
    if s.len() > 0 {
        let _ = account_decode(&s)
            .map_err(|e| format!("failed to parse TON address: {}", e)).unwrap();
    }
    S::from_str(&s).map_err(de::Error::custom)
}