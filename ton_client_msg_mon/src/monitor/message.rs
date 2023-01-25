use serde::{Deserializer, Serializer};
use serde_json::Value;
use std::str::FromStr;
use ton_block::MsgAddrStd;
use ton_types::UInt256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MessageMonitoringParams {
    #[serde(serialize_with = "serialize_uint256")]
    pub hash: UInt256,
    #[serde(serialize_with = "serialize_address")]
    pub address: MsgAddrStd,
    pub wait_until: u32,
    pub user_data: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct MessageMonitoringResult {
    #[serde(deserialize_with = "deserialize_uint256")]
    pub hash: UInt256,
    pub user_data: Option<Value>,
    pub status: MessageMonitoringStatus,
    pub transaction: Option<MessageMonitoringTransaction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum MessageMonitoringStatus {
    Finalized,
    Timeout,
    Reserved,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct MessageMonitoringTransaction {
    #[serde(deserialize_with = "deserialize_uint256")]
    pub hash: UInt256,
}

fn serialize_uint256<S>(value: &UInt256, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&value.to_hex_string())
}

fn serialize_address<S>(value: &MsgAddrStd, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&value.to_string())
}

fn deserialize_uint256<'de, D: Deserializer<'de>, R: From<UInt256>>(
    deserializer: D,
) -> Result<R, D::Error> {
    let string = deserializer.deserialize_string(StringVisitor)?;
    Ok(UInt256::from_str(&string)
        .map_err(|err| serde::de::Error::custom(err))?
        .into())
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}
