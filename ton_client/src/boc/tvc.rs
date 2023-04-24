/*
    Copyright 2023 EverX Labs.

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use crate::boc::internal::{
    deserialize_cell_from_boc, deserialize_object_from_boc, serialize_cell_to_base64,
    serialize_object_to_cell,
};
use crate::boc::tvc_serialization::{TvmSmc, Version, TVC};
use crate::error::ClientResult;
use crate::ClientContext;
use ton_block::StateInit;
use ton_types::Cell;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeTvc {
    /// Contract TVC BOC encoded as base64 or BOC handle
    pub tvc: String,
}

#[derive(Serialize, ApiType, Eq, PartialEq, Debug)]
pub struct ResultOfDecodeTvc {
    /// Decoded TVC
    pub tvc: Tvc,
}

#[derive(Serialize, ApiType, Eq, PartialEq, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Tvc {
    None,
    V1(TvcV1),
}

#[derive(Serialize, ApiType, Eq, PartialEq, Debug)]
pub struct TvcV1 {
    pub code: String,
    pub meta: Option<TvcV1Metadata>,
}

#[derive(Serialize, ApiType, Eq, PartialEq, Debug)]
pub struct TvcV1Metadata {
    pub sold: TvcV1Version,
    pub linker: TvcV1Version,
    pub compiled_at: String,
    pub name: String,
    pub desc: String,
}

#[derive(Serialize, ApiType, Eq, PartialEq, Debug)]
pub struct TvcV1Version {
    pub commit: String,
    pub semantic: String,
}

impl From<Version> for TvcV1Version {
    fn from(value: Version) -> Self {
        Self {
            commit: hex::encode(value.commit),
            semantic: value.semantic,
        }
    }
}

/// Decodes tvc into code, data, libraries and special options.
#[api_function]
pub async fn decode_tvc(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfDecodeTvc,
) -> ClientResult<ResultOfDecodeTvc> {
    let tvc = deserialize_object_from_boc::<TVC>(&context, &params.tvc, "TVC")
        .await?
        .object;
    let tvc = match tvc.tvc {
        TvmSmc::None => Tvc::None,
        TvmSmc::TvcV1(tvc) => Tvc::V1(TvcV1 {
            code: serialize_cell_to_base64(&tvc.code, "TVC code")?,
            meta: tvc.meta.map(|x| TvcV1Metadata {
                sold: x.sold.into(),
                linker: x.linker.into(),
                compiled_at: x.compiled_at.to_string(),
                name: x.name.string,
                desc: x.desc.to_string(),
            }),
        }),
    };
    Ok(ResultOfDecodeTvc { tvc })
}

pub(crate) fn state_init_with_code(code: Cell) -> ClientResult<Cell> {
    let mut state_init = StateInit::default();
    state_init.set_code(code);
    serialize_object_to_cell(&state_init, "state init")
}

pub(crate) async fn resolve_state_init_cell(
    context: &ClientContext,
    tvc_or_state_init: &str,
) -> ClientResult<Cell> {
    if let Ok(tvc) = deserialize_object_from_boc::<TVC>(context, tvc_or_state_init, "TVC").await {
        match &tvc.object.tvc {
            TvmSmc::TvcV1(v1) => state_init_with_code(v1.code.clone()),
            TvmSmc::None => Err(crate::boc::Error::invalid_boc("TVC or StateInit"))?,
        }
    } else {
        Ok(
            deserialize_cell_from_boc(context, tvc_or_state_init, "state init")
                .await?
                .1,
        )
    }
}
