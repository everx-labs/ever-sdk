/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use std::fmt;
use crate::*;

pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";


pub const CONTRACT_CALL_STATE_FIELDS: &str = "id status transaction_id";

pub const MSG_STATE_FIELD_NAME: &str = "status";

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct QueriesConfig {
    pub queries_server: String,
    pub subscriptions_server: String,
}

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestsConfig {
    pub requests_server: String,
}

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub queries_config: QueriesConfig,
    pub requests_config: RequestsConfig,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct StringId (String);

impl From<String> for StringId {
    fn from(id: String) -> Self {
        StringId{0: id}
    }
}

impl From<&str> for StringId {
    fn from(id: &str) -> Self {
        StringId{0: id.to_owned()}
    }
}

impl From<Vec<u8>> for StringId {
    fn from(id: Vec<u8>) -> Self {
        StringId{0: hex::encode(id)}
    }
}

impl From<&[u8]> for StringId {
    fn from(id: &[u8]) -> Self {
        StringId{0: hex::encode(id)}
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}", self.0)
    }
}

impl StringId {
    pub fn to_base64(&self) -> SdkResult<String> {
        let bytes = self.to_bytes()?;
        Ok(base64::encode(&bytes))
    }

    pub fn to_bytes(&self) -> SdkResult<Vec<u8>> {
        hex::decode(&self.0).map_err(Into::into)
    }
}