use {Param, ParamType, Token};
use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeMap};

pub enum DetokenizeError {
    WrongParametersCount,
    SerdeError(serde_json::Error),
    WrongParameterType,
}

/// This trait should be used to parse string values as tokens.
pub struct Detokenizer;

impl Detokenizer {
    fn detokenize(params: &[Param], tokens: &[Token]) -> Result<String, DetokenizeError> {
        if params.len() != tokens.len() {
            return Err(DetokenizeError::WrongParametersCount);
        }

        if !Token::types_check(tokens, params) {
            return Err(DetokenizeError::WrongParameterType);
        }

        let map = params
            .iter()
            .zip(tokens)
            .fold(HashMap::new(), |map, (param, token)| {
                map.insert(*param, *token);
                map
            });

        serde_json::to_string(&FunctionParams{params: map}).map_err(|err| DetokenizeError::SerdeError(err))?;
    }
}

pub struct FunctionParams {
    params: HashMap<Param, Token>,
}

impl Serialize for FunctionParams {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.params.len()))?;
        for (param, token) in self.params {
            if let ParamType::Tuple(tuple_params) = param.kind && Token::Tuple(tuple_tokens) = token {
                
            }
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}
