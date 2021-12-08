/*
* Copyright 2018-2021 TON Labs LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::MessageType;
use serde::de::Error;
use std::fmt;
use std::str::FromStr;
use ton_block::{
    AccStatusChange, AccountStatus, ComputeSkipReason, MsgAddressInt, TransactionProcessingStatus,
};
use ton_types::Cell;

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String")
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

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok("null".to_owned())
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_string(StringVisitor)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok("null".to_owned())
    }
}

struct U8Visitor;

impl<'de> serde::de::Visitor<'de> for U8Visitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Number")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v as u8)
    }
}

pub mod opt_cell {
    use super::*;

    pub fn deserialize<'de, D>(d: D) -> Result<Option<Cell>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b64 = d.deserialize_option(StringVisitor)?;

        if "null" == b64 {
            Ok(None)
        } else {
            Ok(Some(deserialize_tree_of_cells_from_base64::<D>(&b64)?))
        }
    }

    pub fn serialize<S>(value: &Option<Cell>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(cell) = value {
            let str_value = base64::encode(&ton_types::serialize_toc(&cell).map_err(|err| {
                serde::ser::Error::custom(format!("Cannot serialize BOC: {}", err))
            })?);
            serializer.serialize_some(&str_value)
        } else {
            serializer.serialize_none()
        }
    }
}

pub fn deserialize_tree_of_cells_from_base64<'de, D>(b64: &str) -> Result<Cell, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = base64::decode(&b64)
        .map_err(|err| D::Error::custom(format!("error decode base64: {}", err)))?;

    ton_types::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| D::Error::custom(format!("BOC read error: {}", err)))
}

pub mod address {
    use super::*;

    pub fn deserialize<'de, D>(d: D) -> Result<MsgAddressInt, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = d.deserialize_string(StringVisitor)?;

        MsgAddressInt::from_str(&string)
            .map_err(|err| D::Error::custom(format!("Address parsing error: {}", err)))
    }

    pub fn serialize<S>(value: &MsgAddressInt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}

pub mod uint {
    use super::*;

    pub fn deserialize<'de, D>(d: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = d.deserialize_option(StringVisitor)?;

        if "null" == string {
            return Ok(0);
        }

        if !string.starts_with("0x") {
            return Err(D::Error::custom(format!(
                "Number parsing error: number must be prefixed with 0x ({})",
                string
            )));
        }

        u64::from_str_radix(&string[2..], 16)
            .map_err(|err| D::Error::custom(format!("Error parsing number: {}", err)))
    }

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{:x}", value))
    }
}

pub fn deserialize_tr_state<'de, D>(d: D) -> Result<TransactionProcessingStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match d.deserialize_u8(U8Visitor) {
        Err(_) => Ok(TransactionProcessingStatus::Unknown),
        Ok(0) => Ok(TransactionProcessingStatus::Unknown),
        Ok(1) => Ok(TransactionProcessingStatus::Preliminary),
        Ok(2) => Ok(TransactionProcessingStatus::Proposed),
        Ok(3) => Ok(TransactionProcessingStatus::Finalized),
        Ok(4) => Ok(TransactionProcessingStatus::Refused),
        Ok(num) => Err(D::Error::custom(format!(
            "Invalid transaction state: {}",
            num
        ))),
    }
}

pub fn transaction_status_to_u8(status: TransactionProcessingStatus) -> u8 {
    match status {
        TransactionProcessingStatus::Unknown => 0,
        TransactionProcessingStatus::Preliminary => 1,
        TransactionProcessingStatus::Proposed => 2,
        TransactionProcessingStatus::Finalized => 3,
        TransactionProcessingStatus::Refused => 4,
    }
}

pub fn deserialize_acc_state_change<'de, D>(d: D) -> Result<AccStatusChange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let num = d.deserialize_u8(U8Visitor)?;

    match num {
        0 => Ok(AccStatusChange::Unchanged),
        1 => Ok(AccStatusChange::Frozen),
        2 => Ok(AccStatusChange::Deleted),
        num => Err(D::Error::custom(format!(
            "Invalid account change state: {}",
            num
        ))),
    }
}

pub fn deserialize_skipped_reason<'de, D>(d: D) -> Result<Option<ComputeSkipReason>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match d.deserialize_u8(U8Visitor) {
        Err(_) => Ok(None),
        Ok(0) => Ok(Some(ComputeSkipReason::NoState)),
        Ok(1) => Ok(Some(ComputeSkipReason::BadState)),
        Ok(2) => Ok(Some(ComputeSkipReason::NoGas)),
        Ok(num) => Err(D::Error::custom(format!("Invalid skip reason: {}", num))),
    }
}

pub fn deserialize_message_type<'de, D>(d: D) -> Result<MessageType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let num = d.deserialize_u8(U8Visitor)?;

    match num {
        0 => Ok(MessageType::Internal),
        1 => Ok(MessageType::ExternalInbound),
        2 => Ok(MessageType::ExternalOutbound),
        num => Err(D::Error::custom(format!("Invalid message type: {}", num))),
    }
}

pub mod account_status {
    use super::*;

    pub fn deserialize<'de, D>(d: D) -> Result<AccountStatus, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let num = d.deserialize_u8(U8Visitor)?;

        match num {
            0 => Ok(AccountStatus::AccStateUninit),
            1 => Ok(AccountStatus::AccStateActive),
            2 => Ok(AccountStatus::AccStateFrozen),
            3 => Ok(AccountStatus::AccStateNonexist),
            num => Err(D::Error::custom(format!("Invalid account status: {}", num))),
        }
    }

    pub fn serialize<S>(value: &AccountStatus, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(match value {
            AccountStatus::AccStateUninit => 0,
            AccountStatus::AccStateActive => 1,
            AccountStatus::AccStateFrozen => 2,
            AccountStatus::AccStateNonexist => 3,
        })
    }
}

pub fn account_status_to_u8(status: AccountStatus) -> u8 {
    match status {
        AccountStatus::AccStateUninit => 0,
        AccountStatus::AccStateActive => 1,
        AccountStatus::AccStateFrozen => 2,
        AccountStatus::AccStateNonexist => 3,
    }
}

pub fn deserialize_shard<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = d.deserialize_string(StringVisitor)?;

    u64::from_str_radix(&string, 16)
        .map_err(|err| D::Error::custom(format!("Error parsing shard: {}", err)))
}
