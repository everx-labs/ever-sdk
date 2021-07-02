use crate::abi::types::Abi;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::token::Detokenizer;
use ton_abi::{Param, ParamType, Token, TokenValue};
use ton_types::{deserialize_tree_of_cells, SliceData};

//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeAccountData {
    /// Contract ABI
    pub abi: Abi,

    /// Data BOC
    ///
    /// Must be encoded with base64
    pub data: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfDecodeData {
    /// Decoded data as a JSON structure.
    pub data: Value,
}

#[derive(Deserialize)]
struct Abi21Fields {
    #[serde(rename = "ABI version")]
    abi_version: u8,
    #[serde(default)]
    fields: Vec<Param>,
}

/// Decodes account data using provided data BOC and ABI.
///
/// Note: this feature requires ABI 2.1 or higher.
#[api_function]
pub async fn decode_account_data(
    _context: Arc<ClientContext>,
    params: ParamsOfDecodeAccountData,
) -> ClientResult<ResultOfDecodeData> {
    let data = base64_decode(&params.data)?;
    let data =
        deserialize_tree_of_cells(&mut data.as_slice()).map_err(|x| Error::invalid_json(x))?;
    let abi: Abi21Fields =
        serde_json::from_str(&params.abi.json_string()?).map_err(|x| Error::invalid_json(x))?;

    let mut fields = vec![
        Param::new("pubkey", ParamType::Uint(256)),
        Param::new("time", ParamType::Time),
        Param::new("someFlag", ParamType::Bool),
    ];
    fields.extend(abi.fields);
    let mut cursor = SliceData::from(&data);
    let mut tokens = vec![];

    for param in &fields {
        let last = Some(param) == fields.last();
        let (token_value, new_cursor) =
            TokenValue::read_from(&param.kind, cursor, last, abi.abi_version)
                .map_err(|e| Error::invalid_data_for_decode(e))?;

        cursor = new_cursor;
        tokens.push(Token {
            name: param.name.clone(),
            value: token_value,
        });
    }

    let data = Detokenizer::detokenize_to_json_value(&fields, &tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;
    Ok(ResultOfDecodeData { data })
}
