use super::{context::str_hex_to_utf8, Error, JsonValue, TonClient};
use crate::boc::{get_compiler_version, parse_account, ParamsOfGetCompilerVersion, ParamsOfParse};
use crate::encoding::account_decode;
use crate::error::ClientResult;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct DInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub caption: Option<String>,
    pub author: Option<String>,
    #[serde(deserialize_with = "validate_ton_address")]
    pub support: Option<String>,
    pub hello: Option<String>,
    pub language: Option<String>,
    pub dabi: Option<String>,
    pub icon: Option<String>,
    pub interfaces: Vec<String>,
    pub dabi_version: String,
}

impl DInfo {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

fn validate_ton_address<'de, D>(des: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(des)?;
    if let Some(s) = s {
        let _ = account_decode(&s).map_err(serde::de::Error::custom)?;
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

fn convert_to_utf8(hex_str: &mut Option<String>) -> Result<(), String> {
    if let Some(hex) = hex_str {
        *hex_str =
            Some(str_hex_to_utf8(&hex).ok_or(format!("failed to convert bytes to utf8 string"))?);
    }
    Ok(())
}

pub(crate) fn parse_debot_info(value: Option<JsonValue>) -> Result<DInfo, String> {
    let value = value.unwrap_or(json!({}));
    let mut info: DInfo = serde_json::from_value(value)
        .map_err(|e| format!("failed to parse \"DebotInfo\": {}", e))?;
    // Ignore error because debot ABI can be loaded in 2 ways: as string or as bytes.
    let _ = convert_to_utf8(&mut info.dabi);
    Ok(info)
}

pub(crate) fn fetch_target_abi_version(
    ton: TonClient,
    account_boc: String,
) -> ClientResult<String> {
    let json_value = parse_account(ton.clone(), ParamsOfParse { boc: account_boc })?.parsed;
    let code = json_value["code"]
        .as_str()
        .ok_or(Error::debot_has_no_code())?
        .to_owned();
    let result = get_compiler_version(ton.clone(), ParamsOfGetCompilerVersion { code });

    // If If DeBot's code does not contain version or SDK failed to read version,
    // then set empty string.
    let version = result
        .map(|r| r.version.unwrap_or_default())
        .unwrap_or_default();
    let mut iter = version.split(' ');
    let dabi_version = if let Some("sol") = iter.next() {
        // if DeBot's code contains version and it's a solidity DeBot
        match iter.next() {
            Some(compiler_ver) if compiler_ver <= "0.47.0" => "2.0",
            _ => "2.2",
        }
    } else {
        // If DeBot's code does not contain version,
        // then assume that it is very old DeBot built with the compiler
        // older than solc 0.45.0, so let's use abi 2.0 as a target.
        "2.0"
    };

    Ok(dabi_version.to_owned())
}
