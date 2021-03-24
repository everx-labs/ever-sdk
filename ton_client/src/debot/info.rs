use super::context::{str_hex_to_utf8};
use serde::{Deserialize, Deserializer};
use crate::encoding::account_decode;

#[derive(Deserialize, Default, Debug, Clone)]
pub struct DInfo {
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub name: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub version: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub publisher: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub key: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub author: Option<String>,
    #[serde(deserialize_with = "validate_ton_address")]
    pub support: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub hello: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub language: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub dabi: Option<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
}

impl DInfo {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

fn validate_ton_address<'de, D>(des: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>
{
    let s: Option<String> = Deserialize::deserialize(des)?;
    if let Some(s) = s {
        let _ = account_decode(&s)
            .map_err(serde::de::Error::custom)?;
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

fn from_opt_hex_to_str<'de, D>(des: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>
{
    let s: Option<String> = Deserialize::deserialize(des)?;
    if let Some(s) = s {
        let utf8_str = str_hex_to_utf8(&s)
            .ok_or(format!("failed to convert bytes to utf8 string")).unwrap();
        Ok(Some(utf8_str))
    } else {
        Ok(None)
    }
}