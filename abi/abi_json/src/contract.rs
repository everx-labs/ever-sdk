use std::io;
use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::{Unexpected, Error as SerdeError};
use serde_json;
use {Function, Event, ABIError, Token, Param};
use tvm::stack::SliceData;

/// API building calls to contracts ABI.
#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
    /// Contract functions.
    pub functions: HashMap<String, Function>,
    /// Contract events.
    pub events: HashMap<String, Event>,
}

impl<'a> Deserialize<'a> for Contract {
    fn deserialize<D>(deserializer: D) -> Result<Contract, D::Error> where D: Deserializer<'a> {
        // A little trick similar to `Param` deserialization: first deserialize JSON into temporary 
        // struct `SerdeContract` containing necessary fields and then repack functions into HashMap
        let serde_contract = SerdeContract::deserialize(deserializer)?;

        if serde_contract.abi_version != 0 {
            return Err(
                <D::Error as SerdeError>::invalid_value(
                    Unexpected::Unsigned(serde_contract.abi_version as u64), &"ABI version `0`")
            );
        }

        let mut result = Self {
            functions: HashMap::new(),
            events: HashMap::new()
        };

        for mut function in serde_contract.functions {
            function.id = function.get_function_id();
            result.functions.insert(function.name.clone(), function);
        }

        for mut event in serde_contract.events {
            event.id = event.get_function_id();
            result.events.insert(event.name.clone(), event);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeContract {
    /// ABI version.
    #[serde(rename="ABI version")]
    pub abi_version: u8,
    /// Contract functions.
    pub functions: Vec<Function>,
    /// Contract events.
    #[serde(default)]
    pub events: Vec<Event>,
}

pub struct DecodedMessage {
    pub function_name: String,
    pub tokens: Vec<Token>,
    pub params: Vec<Param>
}

impl Contract {
    /// Loads contract from json.
    pub fn load<T: io::Read>(reader: T) -> Result<Self, ABIError> {
        serde_json::from_reader(reader).map_err(|serde_error| ABIError::SerdeError(serde_error))
    }

    /// Returns `Function` struct with provided function name.
    pub fn function(&self, name: &str) -> Result<&Function, ABIError> {
        self.functions.get(name).ok_or_else(|| ABIError::InvalidName(name.to_owned()))
    }

    /// Returns `Function` struct with provided function id.
    pub fn function_by_id(&self, id: u32) -> Result<&Function, ABIError> {
        for (_, func) in &self.functions {
            if func.id == id {
                return Ok(func);
            }
        }

        Err(ABIError::InvalidFunctionId(id))
    }

    /// Returns `Event` struct with provided function id.
    pub fn event_by_id(&self, id: u32) -> Result<&Event, ABIError> {
        for (_, event) in &self.events {
            if event.id == id {
                return Ok(event);
            }
        }

        Err(ABIError::InvalidFunctionId(id))
    }

    /// Returns functions collection
    pub fn functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }

    /// Returns events collection
    pub fn events(&self) -> &HashMap<String, Event> {
        &self.events
    }

    /// Decodes contract answer and returns name of the function called
    pub fn decode_output(&self, data: SliceData) -> Result<DecodedMessage, ABIError> {
        let original_data = data.clone();
        
        let func_id = Function::decode_id(data)
            .map_err(|err| ABIError::DeserializationError(err))?;

        if let Ok(func) = self.function_by_id(func_id){
            let tokens = func.decode_output(original_data)
                .map_err(|err| ABIError::DeserializationError(err))?;

            Ok( DecodedMessage {
                function_name: func.name.clone(),
                tokens: tokens,
                params: func.output_params()
            })
        } else {
            let event = self.event_by_id(func_id)?;
            let tokens = event.decode_input(original_data)
                .map_err(|err| ABIError::DeserializationError(err))?;

            Ok( DecodedMessage {
                function_name: event.name.clone(),
                tokens: tokens,
                params: event.input_params()
            })
        }
    }

    /// Decodes contract answer and returns name of the function called
    pub fn decode_input(&self, data: SliceData) -> Result<DecodedMessage, ABIError> {
        let original_data = data.clone();
        
        let func_id = Function::decode_id(data)
            .map_err(|err| ABIError::DeserializationError(err))?;

        let func = self.function_by_id(func_id)?;

        let tokens = func.decode_input(original_data)
            .map_err(|err| ABIError::DeserializationError(err))?;

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