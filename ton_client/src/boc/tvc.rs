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
*/

use ton_types::{BuilderData, Cell, SliceData};

use crate::boc::internal::{deserialize_cell_from_boc, deserialize_object_from_boc, serialize_cell_to_boc};
use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;

use super::BocCacheType;

const OLD_SELECTOR_DATA: &[u8] = &[0xff, 0x00, 0x20, 0xc1, 0x01, 0xf4, 0xa4, 0x20, 0x58, 0x92, 0xf4, 0xa0, 0xe0, 0x5f, 0x02, 0x8a, 0x20, 0xed, 0x53, 0xd9, 0x80];
const NEW_SELECTOR_DATA: &[u8] = &[0x8a, 0xed, 0x53, 0x20, 0xe3, 0x03, 0x20, 0xc0, 0xff, 0xe3, 0x02, 0x20, 0xc0, 0xfe, 0xe3, 0x02, 0xf2, 0x0b, 0x80];
const MYCODE_SELECTOR_DATA: &[u8] = &[0x8A, 0xDB, 0x35, 0x80];

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfGetCodeFromTvc {
    /// Contract TVC image or image BOC handle
    pub tvc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfGetCodeFromTvc {
    /// Contract code encoded as base64
    pub code: String,
}

/// Extracts code from TVC contract image
#[api_function]
pub async fn get_code_from_tvc(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCodeFromTvc,
) -> ClientResult<ResultOfGetCodeFromTvc> {
    let object = deserialize_object_from_boc::<ton_block::StateInit>(&context, &params.tvc, "TVC").await?;
    
    let code = object.object.code.ok_or(Error::invalid_boc("TVC image has no code"))?;

    Ok(ResultOfGetCodeFromTvc {
        code: super::internal::serialize_cell_to_base64(&code, "code")?,
    })
}

fn get_old_selector_salt(code: &Cell) -> ClientResult<Option<Cell>> {
    Ok(code.reference(2).ok())
}

fn get_new_selector_salt(code: &Cell) -> ClientResult<Option<Cell>> {
    let mut private_selector: SliceData = code.reference(0)
        .map_err(|_| Error::invalid_boc("no private functions selector in new selector"))?
        .into();
    if private_selector.get_next_bits(13).ok() != Some(vec![0xf4, 0xa0]) {
        return Err(Error::invalid_boc("invalid private functions selector data"))
    }
    private_selector.get_dictionary_opt();
    Ok(private_selector.reference_opt(1))
}

fn get_mycode_selector_salt(code: &Cell) -> ClientResult<Option<Cell>> {
    let new_selector = code.reference(1)
        .map_err(|_| Error::invalid_boc("no new selector in mycode selector"))?;
    get_new_selector_salt(&new_selector)
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfGetCodeSalt {
    /// Contract code BOC encoded as base64 or image BOC handle
    pub code: String,
    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfGetCodeSalt {
    /// Contract code salt if present. BOC encoded as base64 or BOC handle
    pub salt: Option<String>,
}

/// Returns contract code salt if present.
#[api_function]
pub async fn get_code_salt(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCodeSalt,
) -> ClientResult<ResultOfGetCodeSalt> {
    let (_, code) = deserialize_cell_from_boc(&context, &params.code, "contract code").await?;
    
    let salt = match code.data() {
        OLD_SELECTOR_DATA => get_old_selector_salt(&code),
        NEW_SELECTOR_DATA => get_new_selector_salt(&code),
        MYCODE_SELECTOR_DATA => get_mycode_selector_salt(&code),
        _ => Err(Error::invalid_boc("unknown contract type")),
    }?;
    
    let salt = if let Some(salt) = salt {
        Some(serialize_cell_to_boc(&context, salt, "code salt", params.boc_cache).await?)
    } else {
        None
    };
    Ok(ResultOfGetCodeSalt { salt })
}

fn builder_to_cell(builder: BuilderData) -> ClientResult<Cell> {
    builder
        .into_cell()
        .map_err(|err| Error::invalid_boc(format!("can not convert builder to cell: {}", err)))
}

fn set_salt(cell: Cell, salt: Cell, replace_last_ref: bool) -> ClientResult<Cell> {
    let mut builder: BuilderData = cell.into();
    if replace_last_ref {
        builder.replace_reference_cell(builder.references_used() - 1, salt);
    } else {
        builder.checked_append_reference(salt)
            .map_err(|_| Error::invalid_boc("no empty reference for salt"))?;
    }
    builder_to_cell(builder)
}

fn set_old_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let salt_present = get_old_selector_salt(&code)?.is_some();
    set_salt(code, salt, salt_present)
}

fn set_new_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let private_selector = code.reference(0)
        .map_err(|_| Error::invalid_boc("no private functions selector in new selector"))?;

    let private_selector = set_salt(
        private_selector, salt, get_new_selector_salt(&code)?.is_some()
    )?;

    let mut builder: BuilderData = code.into();
    builder.replace_reference_cell(0, private_selector);
    builder_to_cell(builder)
}

fn set_mycode_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let new_selector = code.reference(1)
        .map_err(|_| Error::invalid_boc("no new selector in mycode selector"))?;
    let new_selector = set_new_selector_salt(new_selector, salt)?;
    
    let mut builder: BuilderData = code.into();
    builder.replace_reference_cell(1, new_selector);
    builder_to_cell(builder)
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfSetCodeSalt {
    /// Contract TVC image BOC encoded as base64 or image BOC handle
    pub code: String,
    /// Code salt to set. BOC encoded as base64 or BOC handle
    pub salt: String,
    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfSetCodeSalt {
    /// Contract code with salt set. BOC encoded as base64 or BOC handle
    pub code: String,
}

/// Sets new salt to contract code.
#[api_function]
pub async fn set_code_salt(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSetCodeSalt,
) -> ClientResult<ResultOfSetCodeSalt> {
    let (_, code) = deserialize_cell_from_boc(&context, &params.code, "contract code").await?;
    let (_, salt) = deserialize_cell_from_boc(&context, &params.salt, "salt").await?;
    
    let code = match code.data() {
        OLD_SELECTOR_DATA => set_old_selector_salt(code, salt),
        NEW_SELECTOR_DATA => set_new_selector_salt(code, salt),
        MYCODE_SELECTOR_DATA => set_mycode_selector_salt(code, salt),
        _ => Err(Error::invalid_boc("unknown contract type")),
    }?;
    
    Ok(ResultOfSetCodeSalt { 
        code: serialize_cell_to_boc(&context, code, "contract code", params.boc_cache).await?
    })
}
