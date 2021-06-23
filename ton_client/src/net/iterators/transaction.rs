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

pub const TRANSACTION_FIELDS: &str = r#"
    id
    account_addr
    now
    balance_delta(format:DEC)
    bounce { bounce_type }
    in_message {
        id
        value(format:DEC)
        msg_type
        src
    }
    out_messages {
        id
        value(format:DEC)
        msg_type
        dst
    }
"#;

pub(crate) struct MessageFields<'a>(&'a Value);

impl<'a> MessageFields<'a> {
    pub fn id(&self) -> &str {
        self.0["id"].as_str().unwrap_or("")
    }
    pub fn value(&self) -> &str {
        self.0["value"].as_str().unwrap_or("")
    }
    pub fn src(&self) -> &str {
        self.0["src"].as_str().unwrap_or("")
    }
    pub fn dst(&self) -> &str {
        self.0["dst"].as_str().unwrap_or("")
    }
}

pub(crate) struct TransactionBounceFields<'a>(&'a Value);

impl<'a> TransactionBounceFields<'a> {
    pub fn bounce_type(&self) -> u32 {
        self.0["bounce_type"].as_u64().unwrap_or(0) as u32
    }
}

pub(crate) struct TransactionFields<'a>(pub &'a Value);

impl<'a> TransactionFields<'a> {
    pub fn now(&self) -> u32 {
        self.0["now"].as_u64().unwrap_or(0) as u32
    }
    pub fn bounce(&self) -> Option<TransactionBounceFields> {
        self.0.get("bounce").map(|x| TransactionBounceFields(x))
    }
    pub fn in_message(&self) -> Option<MessageFields> {
        self.0.get("in_message").map(|x| MessageFields(x))
    }
    pub fn out_messages(&self) -> Option<Vec<MessageFields>> {
        self.0["out_messages"]
            .as_array()
            .map(|x| x.iter().map(|x| MessageFields(x)).collect())
    }
}
