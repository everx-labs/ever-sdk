/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

use ever_block::{Number5, StateInit, StateInitLib, TickTock, SERDE_OPTS_EMPTY};
use ever_block::{BuilderData, Cell};

use super::BocCacheType;
use crate::boc::internal::{
    deserialize_cell_from_boc, deserialize_object_from_boc, deserialize_object_from_cell,
    serialize_cell_to_boc, serialize_object_to_boc,
};
use crate::boc::tvc::resolve_state_init_cell;
use crate::boc::Error;
use crate::client::ClientContext;
use crate::encoding::slice_from_cell;
use crate::error::ClientResult;

const OLD_CPP_SELECTOR_DATA: &[u8] = &[
    0xff, 0x00, 0x20, 0xc1, 0x01, 0xf4, 0xa4, 0x20, 0x58, 0x92, 0xf4, 0xa0, 0xe0, 0x5f, 0x02, 0x8a,
    0x20, 0xed, 0x53, 0xd9,
];
const OLD_SOL_SELECTOR_DATA: &[u8] = &[
    0xff, 0x00, 0xf4, 0xa4, 0x20, 0x22, 0xc0, 0x01, 0x92, 0xf4, 0xa0, 0xe1, 0x8a, 0xed, 0x53, 0x58,
    0x30, 0xf4, 0xa1,
];
const NEW_SELECTOR_DATA: &[u8] = &[
    0x8a, 0xed, 0x53, 0x20, 0xe3, 0x03, 0x20, 0xc0, 0xff, 0xe3, 0x02
];
const MYCODE_SELECTOR_DATA: &[u8] = &[0x8A, 0xDB, 0x35];

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
pub fn get_code_from_tvc(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCodeFromTvc,
) -> ClientResult<ResultOfGetCodeFromTvc> {
    let state_init_cell = resolve_state_init_cell(&context, &params.tvc)?;
    let object = deserialize_object_from_cell::<StateInit>(state_init_cell, "TVC")?;

    let code = object
        .code
        .ok_or(Error::invalid_boc("TVC image has no code"))?;

    Ok(ResultOfGetCodeFromTvc {
        code: super::internal::serialize_cell_to_base64(&code, "code")?,
    })
}

fn get_old_selector_salt(code: &Cell) -> ClientResult<Option<Cell>> {
    Ok(code.reference(2).ok())
}

fn get_new_selector_salt_and_ver(code: &Cell) -> ClientResult<(Option<Cell>, Cell)> {
    let private_selector = code
        .reference(0)
        .map_err(|_| Error::invalid_boc("no private functions selector in new selector"))?;
    let mut private_selector = slice_from_cell(private_selector)?;
    if private_selector.get_next_bits(13).ok() != Some(vec![0xf4, 0xa0]) {
        return Err(Error::invalid_boc(
            "invalid private functions selector data",
        ));
    }
    private_selector.get_dictionary_opt();
    let version = private_selector
        .reference_opt(0)
        .ok_or_else(|| Error::invalid_boc("no compiler version in contract code"))?;
    Ok((private_selector.reference_opt(1), version))
}

fn get_mycode_selector_salt_and_ver(code: &Cell) -> ClientResult<(Option<Cell>, Cell)> {
    let new_selector = code
        .reference(1)
        .map_err(|_| Error::invalid_boc("no new selector in mycode selector"))?;
    get_new_selector_salt_and_ver(&new_selector)
}

pub fn get_salt_and_ver(code: Cell) -> ClientResult<(Option<Cell>, Option<Cell>)> {
    match code.data() {
        OLD_CPP_SELECTOR_DATA => get_old_selector_salt(&code).map(|salt| (salt, None)),
        OLD_SOL_SELECTOR_DATA => Ok((None, None)),
        code_data if code_data.starts_with(NEW_SELECTOR_DATA) => {
            get_new_selector_salt_and_ver(&code).map(|(salt, ver)| (salt, Some(ver)))
        }
        MYCODE_SELECTOR_DATA => {
            get_mycode_selector_salt_and_ver(&code).map(|(salt, ver)| (salt, Some(ver)))
        }
        _ => Err(Error::invalid_boc("unknown contract type")),
    }
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfGetCodeSalt {
    /// Contract code BOC encoded as base64 or code BOC handle
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

/// Returns the contract code's salt if it is present.
#[api_function]
pub fn get_code_salt(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCodeSalt,
) -> ClientResult<ResultOfGetCodeSalt> {
    let (_, code) = deserialize_cell_from_boc(&context, &params.code, "contract code")?;

    let (salt, _) = get_salt_and_ver(code)?;

    let salt = if let Some(salt) = salt {
        Some(serialize_cell_to_boc(
            &context,
            salt,
            "code salt",
            params.boc_cache,
        )?)
    } else {
        None
    };
    Ok(ResultOfGetCodeSalt { salt })
}

pub(crate)fn builder_to_cell(builder: BuilderData) -> ClientResult<Cell> {
    builder
        .into_cell()
        .map_err(|err| Error::invalid_boc(format!("can not convert builder to cell: {}", err)))
}

fn set_salt(cell: Cell, salt: Cell, replace_last_ref: bool) -> ClientResult<Cell> {
    let mut builder: BuilderData = BuilderData::from_cell(&cell)
        .map_err(|_| Error::invalid_boc("can't create Builder from Cell"))?;

    if replace_last_ref {
        builder.replace_reference_cell(builder.references_used() - 1, salt);
    } else {
        builder
            .checked_append_reference(salt)
            .map_err(|_| Error::invalid_boc("no empty reference for salt"))?;
    }
    builder_to_cell(builder)
}

fn set_old_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let salt_present = get_old_selector_salt(&code)?.is_some();
    set_salt(code, salt, salt_present)
}

fn set_new_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let private_selector = code
        .reference(0)
        .map_err(|_| Error::invalid_boc("no private functions selector in new selector"))?;

    let private_selector = set_salt(
        private_selector,
        salt,
        get_new_selector_salt_and_ver(&code)?.0.is_some(),
    )?;

    let mut builder: BuilderData = BuilderData::from_cell(&code)
        .map_err(|_| Error::invalid_boc("can't create Builder from Cell"))?;

    builder.replace_reference_cell(0, private_selector);
    builder_to_cell(builder)
}

fn set_mycode_selector_salt(code: Cell, salt: Cell) -> ClientResult<Cell> {
    let new_selector = code
        .reference(1)
        .map_err(|_| Error::invalid_boc("no new selector in mycode selector"))?;
    let new_selector = set_new_selector_salt(new_selector, salt)?;

    let mut builder: BuilderData = BuilderData::from_cell(&code)
        .map_err(|_| Error::invalid_boc("can't create Builder from Cell"))?;

    builder.replace_reference_cell(1, new_selector);
    builder_to_cell(builder)
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfSetCodeSalt {
    /// Contract code BOC encoded as base64 or code BOC handle
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

/// Sets new salt to contract code. Returns the new contract code with salt.
#[api_function]
pub fn set_code_salt(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSetCodeSalt,
) -> ClientResult<ResultOfSetCodeSalt> {
    let (_, code) = deserialize_cell_from_boc(&context, &params.code, "contract code")?;
    let (_, salt) = deserialize_cell_from_boc(&context, &params.salt, "salt")?;

    let code = match code.data() {
        OLD_CPP_SELECTOR_DATA => set_old_selector_salt(code, salt),
        code_data if code_data.starts_with(NEW_SELECTOR_DATA) => set_new_selector_salt(code, salt),
        MYCODE_SELECTOR_DATA => set_mycode_selector_salt(code, salt),
        OLD_SOL_SELECTOR_DATA => Err(Error::invalid_boc(
            "the contract doesn't support salt adding",
        )),
        _ => Err(Error::invalid_boc("unknown contract type")),
    }?;

    Ok(ResultOfSetCodeSalt {
        code: serialize_cell_to_boc(&context, code, "contract code", params.boc_cache)?,
    })
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfGetCompilerVersion {
    /// Contract code BOC encoded as base64 or code BOC handle
    pub code: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfGetCompilerVersion {
    /// Compiler version, for example 'sol 0.49.0'
    pub version: Option<String>,
}

pub fn get_compiler_version_from_cell(code: Cell) -> ClientResult<Option<String>> {
    let (_, version) = get_salt_and_ver(code)?;

    version
        .map(|cell| {
            let bytes = cell.data();
            String::from_utf8(bytes[..bytes.len()].to_vec()).map_err(|err| {
                Error::invalid_boc(format!("can not convert version cell to string: {}", err))
            })
        })
        .transpose()
}

/// Returns the compiler version used to compile the code.
#[api_function]
pub fn get_compiler_version(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCompilerVersion,
) -> ClientResult<ResultOfGetCompilerVersion> {
    let (_, code) = deserialize_cell_from_boc(&context, &params.code, "contract code")?;
    let version = get_compiler_version_from_cell(code)?;

    Ok(ResultOfGetCompilerVersion { version })
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfEncodeStateInit {
    /// Contract code BOC encoded as base64 or BOC handle
    pub code: Option<String>,
    /// Contract data BOC encoded as base64 or BOC handle
    pub data: Option<String>,
    /// Contract library BOC encoded as base64 or BOC handle
    pub library: Option<String>,
    /// `special.tick` field. Specifies the contract ability to handle tick transactions
    pub tick: Option<bool>,
    /// `special.tock` field. Specifies the contract ability to handle tock transactions
    pub tock: Option<bool>,
    /// Is present and non-zero only in instances of large smart contracts
    pub split_depth: Option<u32>,

    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfEncodeStateInit {
    /// Contract StateInit image BOC encoded as base64 or BOC handle of boc_cache parameter was specified
    pub state_init: String,
}

/// Encodes initial contract state from code, data, libraries ans special options (see input params)
#[api_function]
pub fn encode_state_init(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeStateInit,
) -> ClientResult<ResultOfEncodeStateInit> {
    let get_cell = |name, boc| {
        let context = context.clone();
        if let Some(boc) = boc {
            deserialize_cell_from_boc(&context, boc, name).map(|val| Some(val.1))
        } else {
            Ok(None)
        }
    };
    let code = get_cell("code", params.code.as_deref())?;
    let data = get_cell("data", params.data.as_deref())?;
    let library = StateInitLib::with_hashmap(get_cell("library", params.library.as_deref())?, SERDE_OPTS_EMPTY);

    let special = if params.tick.is_some() || params.tock.is_some() {
        Some(TickTock {
            tick: params.tick.unwrap_or_default(),
            tock: params.tock.unwrap_or_default(),
        })
    } else {
        None
    };

    let split_depth = params
        .split_depth
        .map(|split_depth| Number5::new(split_depth).unwrap());

    let state = StateInit {
        code,
        data,
        library,
        special,
        split_depth,
    };

    Ok(ResultOfEncodeStateInit {
        state_init: serialize_object_to_boc(&context, &state, "TVC", params.boc_cache)?,
    })
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeStateInit {
    /// Contract StateInit image BOC encoded as base64 or BOC handle
    pub state_init: String,
    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug, PartialEq)]
pub struct ResultOfDecodeStateInit {
    /// Contract code BOC encoded as base64 or BOC handle
    pub code: Option<String>,
    /// Contract code hash
    pub code_hash: Option<String>,
    /// Contract code depth
    pub code_depth: Option<u32>,
    /// Contract data BOC encoded as base64 or BOC handle
    pub data: Option<String>,
    /// Contract data hash
    pub data_hash: Option<String>,
    /// Contract data depth
    pub data_depth: Option<u32>,
    /// Contract library BOC encoded as base64 or BOC handle
    pub library: Option<String>,
    /// `special.tick` field. Specifies the contract ability to handle tick transactions
    pub tick: Option<bool>,
    /// `special.tock` field. Specifies the contract ability to handle tock transactions
    pub tock: Option<bool>,
    /// Is present and non-zero only in instances of large smart contracts
    pub split_depth: Option<u32>,
    /// Compiler version, for example 'sol 0.49.0'
    pub compiler_version: Option<String>,
}

/// Decodes contract's initial state into code, data, libraries and special options.
#[api_function]
pub fn decode_state_init(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfDecodeStateInit,
) -> ClientResult<ResultOfDecodeStateInit> {
    let state_init = deserialize_object_from_boc::<StateInit>(&context, &params.state_init, "TVC")?;

    let serialize = |name, cell, boc_cache| {
        let context = context.clone();
        if let Some(cell) = cell {
            serialize_cell_to_boc(&context, cell, name, boc_cache).map(Some)
        } else {
            Ok(None)
        }
    };
    let code_depth = state_init
        .object
        .code
        .as_ref()
        .map(|cell| cell.repr_depth() as u32);
    let code_hash = state_init
        .object
        .code
        .as_ref()
        .map(|cell| cell.repr_hash().as_hex_string());
    let compiler_version = state_init
        .object
        .code
        .clone()
        .map(|cell| get_compiler_version_from_cell(cell).ok())
        .flatten()
        .flatten();
    let code = serialize("code", state_init.object.code, params.boc_cache.clone())?;

    let data_depth = state_init
        .object
        .data
        .as_ref()
        .map(|cell| cell.repr_depth() as u32);
    let data_hash = state_init
        .object
        .data
        .as_ref()
        .map(|cell| cell.repr_hash().as_hex_string());
    let data = serialize("data", state_init.object.data, params.boc_cache.clone())?;

    let library = serialize(
        "library",
        state_init.object.library.root().cloned(),
        params.boc_cache.clone(),
    )?;

    Ok(ResultOfDecodeStateInit {
        code,
        code_depth,
        code_hash,
        data,
        data_depth,
        data_hash,
        library,
        tick: state_init.object.special.as_ref().map(|val| val.tick),
        tock: state_init.object.special.as_ref().map(|val| val.tick),
        split_depth: state_init.object.split_depth.map(|val| val.as_u32()),
        compiler_version,
    })
}
