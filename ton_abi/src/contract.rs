/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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

use std::io;
use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::{Unexpected, Error as SerdeError};
use serde_json;
use {DataItem, Function, Event, Token, Param};
use ton_vm::stack::SliceData;
use crate::error::*;
use super::function::ABI_VERSION;

/// API building calls to contracts ABI.
#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
    /// Contract functions.
    functions: HashMap<String, Function>,
    /// Contract events.
    events: HashMap<String, Event>,
    /// Contract initila data.
    data: HashMap<String, DataItem>,
}

impl<'a> Deserialize<'a> for Contract {
    fn deserialize<D>(deserializer: D) -> Result<Contract, D::Error> where D: Deserializer<'a> {
        // A little trick similar to `Param` deserialization: first deserialize JSON into temporary 
        // struct `SerdeContract` containing necessary fields and then repack functions into HashMap
        let serde_contract = SerdeContract::deserialize(deserializer)?;

        if serde_contract.abi_version != ABI_VERSION {
            return Err(
                <D::Error as SerdeError>::invalid_value(
                    Unexpected::Unsigned(serde_contract.abi_version as u64),
                    &format!("ABI version `{}`", ABI_VERSION).as_str())
            );
        }

        let mut result = Self {
            functions: HashMap::new(),
            events: HashMap::new(),
            data: HashMap::new(),
        };

        for mut function in serde_contract.functions {
            function.set_time = serde_contract.set_time;
            function.id = function.get_function_id();
            result.functions.insert(function.name.clone(), function);
        }

        for mut event in serde_contract.events {
            event.id = event.get_function_id();
            result.events.insert(event.name.clone(), event);
        }

        for data in serde_contract.data {
            result.data.insert(data.value.name.clone(), data);
        }

        Ok(result)
    }
}

fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeContract {
    /// ABI version.
    #[serde(rename="ABI version")]
    pub abi_version: u8,
    /// Set timestamp in message.
    #[serde(rename="setTime")]
    #[serde(default="bool_true")]
    pub set_time: bool,
    /// Contract functions.
    pub functions: Vec<Function>,
    /// Contract events.
    #[serde(default)]
    pub events: Vec<Event>,
    /// Contract initial data.
    #[serde(default)]
    pub data: Vec<DataItem>,

}

pub struct DecodedMessage {
    pub function_name: String,
    pub tokens: Vec<Token>,
    pub params: Vec<Param>
}

impl Contract {
    /// Loads contract from json.
    pub fn load<T: io::Read>(reader: T) -> AbiResult<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

    /// Returns `Function` struct with provided function name.
    pub fn function(&self, name: &str) -> AbiResult<&Function> {
        self.functions.get(name).ok_or(AbiErrorKind::InvalidName(name.to_owned()).into())
    }

    /// Returns `Function` struct with provided function id.
    pub fn function_by_id(&self, id: u32, input: bool) -> AbiResult<&Function> {
        for (_, func) in &self.functions {
            let func_id = if input { func.get_input_id() } else { func.get_output_id() };
            if func_id == id {
                return Ok(func);
            }
        }

        bail!(AbiErrorKind::InvalidFunctionId(id))
    }

    /// Returns `Event` struct with provided function id.
    pub fn event_by_id(&self, id: u32) -> AbiResult<&Event> {
        for (_, event) in &self.events {
            if event.id == id {
                return Ok(event);
            }
        }

        bail!(AbiErrorKind::InvalidFunctionId(id))
    }

    /// Returns functions collection
    pub fn functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }

    /// Returns events collection
    pub fn events(&self) -> &HashMap<String, Event> {
        &self.events
    }
    /// Returns data collection
    pub fn data(&self) -> &HashMap<String, DataItem> {
        &self.data
    }

    /// Decodes contract answer and returns name of the function called
    pub fn decode_output(&self, data: SliceData, internal: bool) -> AbiResult<DecodedMessage> {
        let original_data = data.clone();
        
        let func_id = Function::decode_id(data)?;

        if let Ok(func) = self.function_by_id(func_id, false){
            let tokens = func.decode_output(original_data, internal)?;

            Ok( DecodedMessage {
                function_name: func.name.clone(),
                tokens: tokens,
                params: func.output_params()
            })
        } else {
            let event = self.event_by_id(func_id)?;
            let tokens = event.decode_input(original_data)?;

            Ok( DecodedMessage {
                function_name: event.name.clone(),
                tokens: tokens,
                params: event.input_params()
            })
        }
    }

    /// Decodes contract answer and returns name of the function called
    pub fn decode_input(&self, data: SliceData, internal: bool) -> AbiResult<DecodedMessage> {
        let original_data = data.clone();
        
        let func_id = Function::decode_id(data)?;

        let func = self.function_by_id(func_id, true)?;

        let tokens = func.decode_input(original_data, internal)?;

        Ok( DecodedMessage {
            function_name: func.name.clone(),
            tokens: tokens,
            params: func.input_params()
        })
    }
}

#[cfg(test)]
#[path = "tests/test_contract.rs"]
mod tests;