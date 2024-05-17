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

use num_traits::cast::ToPrimitive;
use std::fmt;
use ever_block::{Result, UInt256};

use crate::error::SdkError;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId(String);

pub type BlockId = StringId;

impl From<UInt256> for StringId {
    fn from(id: UInt256) -> Self {
        StringId(id.as_hex_string())
    }
}

impl From<String> for StringId {
    fn from(id: String) -> Self {
        StringId { 0: id }
    }
}

impl From<&str> for StringId {
    fn from(id: &str) -> Self {
        StringId { 0: id.to_owned() }
    }
}

impl From<Vec<u8>> for StringId {
    fn from(id: Vec<u8>) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl From<&[u8]> for StringId {
    fn from(id: &[u8]) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StringId {
    pub fn to_base64(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(base64::encode(&bytes))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        hex::decode(&self.0).map_err(Into::into)
    }
}

pub fn grams_to_u64(grams: &ever_block::types::Grams) -> Result<u64> {
    grams.as_u128().to_u64().ok_or_else(|| {
        SdkError::InvalidData {
            msg: format!("Cannot convert grams value {}", grams),
        }
        .into()
    })
}
