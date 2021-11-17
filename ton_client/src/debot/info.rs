use super::{context::str_hex_to_utf8, TonClient, Error};
use crate::boc::{get_compiler_version, parse_account, ParamsOfGetCompilerVersion, ParamsOfParse};
use crate::encoding::account_decode;
use crate::error::ClientResult;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct DInfo {
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub name: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub version: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub publisher: Option<String>,
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub caption: Option<String>,
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
    #[serde(deserialize_with = "from_opt_hex_to_str")]
    pub icon: Option<String>,
    pub interfaces: Vec<String>,
    pub target_abi: String,
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

fn from_opt_hex_to_str<'de, D>(des: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(des)?;
    if let Some(s) = s {
        let utf8_str = str_hex_to_utf8(&s)
            .ok_or(format!("failed to convert bytes to utf8 string"))
            .unwrap();
        Ok(Some(utf8_str))
    } else {
        Ok(None)
    }
}

pub(crate) async fn fetch_target_abi_version(
    ton: TonClient,
    account_boc: String,
) -> ClientResult<String> {
    let json_value = parse_account(ton.clone(), ParamsOfParse { boc: account_boc })
        .await?
        .parsed;
    let code = json_value["code"].as_str()
        .ok_or(Error::debot_has_no_code())?
        .to_owned();
    let result = get_compiler_version(ton.clone(), ParamsOfGetCompilerVersion { code }).await;

    // If If DeBot's code does not contain version or SDK failed to read version,
    // then set empty string.
    let version = result
        .map(|r| r.version.unwrap_or_default())
        .unwrap_or_default();
    let mut iter = version.split(' ');
    let target_abi = if let Some("sol") = iter.next() {
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

    Ok(target_abi.to_owned())
}
