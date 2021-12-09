/*
* Copyright 2018-2021 TON Labs LTD.
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

use crate::ClientContext;
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfCompressZstd {
    /// Uncompressed data. Must be encoded as base64.
    pub uncompressed: String,
    /// Compression level, from 1 to 21.
    /// Where:
    /// 1 - lowest compression level (fastest compression);
    /// 21 - highest compression level (slowest compression).
    /// If level is omitted, the default compression level is used (currently `3`).
    pub level: Option<i32>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfCompressZstd {
    /// Compressed data. Must be encoded as base64.
    pub compressed: String,
}

/// Compresses data using Zstandard algorithm
#[api_function]
pub fn compress_zstd(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfCompressZstd,
) -> ClientResult<ResultOfCompressZstd> {
    let uncompressed = base64::decode(&params.uncompressed)
        .map_err(
            |err|
                crate::utils::Error::compression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let compressed = crate::utils::compression::compress_zstd(uncompressed.as_slice(), params.level)?;

    Ok(ResultOfCompressZstd {
        compressed: base64::encode(&compressed),
    })
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfDecompressZstd {
    /// Compressed data. Must be encoded as base64.
    pub compressed: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfDecompressZstd {
    /// Decompressed data. Must be encoded as base64.
    pub decompressed: String,
}

/// Decompresses data using Zstandard algorithm
#[api_function]
pub fn decompress_zstd(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfDecompressZstd,
) -> ClientResult<ResultOfDecompressZstd> {
    let compressed = base64::decode(&params.compressed)
        .map_err(
            |err|
                crate::utils::Error::decompression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let decompressed = crate::utils::compression::decompress_zstd(compressed.as_slice())?;

    Ok(ResultOfDecompressZstd {
        decompressed: base64::encode(&decompressed),
    })
}
