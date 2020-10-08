/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use serde_json::Value;
use ton_client::error::ClientError;

#[derive(Serialize)]
pub struct CliError {
    pub message: String,
    pub data: Option<Value>,
}

impl CliError {
    pub fn with_message(message: String) -> Self {
        Self {
            message,
            data: None,
        }
    }
    pub fn with_message_and_data(message: String, data: Value) -> Self {
        Self {
            message,
            data: Some(data),
        }
    }
}

impl From<ClientError> for CliError {
    fn from(e: ClientError) -> Self {
        if let Ok(data) = serde_json::to_value(&e) {
            Self::with_message_and_data(e.message, data)
        } else {
            Self::with_message(e.message)
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::with_message(e.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self::with_message(e.to_string())
    }
}

impl From<regex::Error> for CliError {
    fn from(e: regex::Error) -> Self {
        Self::with_message(e.to_string())
    }
}

impl From<json5::Error> for CliError {
    fn from(e: json5::Error) -> Self {
        Self::with_message(e.to_string())
    }
}
