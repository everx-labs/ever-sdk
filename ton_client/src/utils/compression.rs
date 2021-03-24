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

use std::io::Cursor;

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
                super::errors::Error::compression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let compressed = compress_zstd_internal(uncompressed.as_slice(), params.level)?;

    Ok(ResultOfCompressZstd {
        compressed: base64::encode(&compressed),
    })
}

/// Compresses data using Zstandard algorithm. Useful for Rust API.
pub fn compress_zstd_internal(uncompressed: &[u8], level: Option<i32>) -> ClientResult<Vec<u8>> {
    let level =  match level {
        None => 0,
        Some(level) => {
            if !(1..=21).contains(&level) {
                return Err(super::errors::Error::compression_error(
                    format!("Invalid compression level: {}", level)
                ));
            }
            level
        }
    };

    let mut compressed = Vec::new();
    zstd::stream::copy_encode(
        &mut Cursor::new(uncompressed),
        &mut compressed,
        level
    ).map_err(|err| super::errors::Error::compression_error(err))?;

    Ok(compressed)
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
                super::errors::Error::decompression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let decompressed = decompress_zstd_internal(compressed.as_slice())?;

    Ok(ResultOfDecompressZstd {
        decompressed: base64::encode(&decompressed),
    })
}

/// Decompresses data using Zstandard algorithm. Useful for Rust API.
pub fn decompress_zstd_internal(compressed: &[u8]) -> ClientResult<Vec<u8>> {
    let mut decompressed = Vec::new();
    zstd::stream::copy_decode(&mut Cursor::new(compressed), &mut decompressed)
        .map_err(|err| super::errors::Error::decompression_error(err))?;

    Ok(decompressed)
}
