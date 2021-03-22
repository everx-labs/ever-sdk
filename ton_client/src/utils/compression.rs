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

use std::ops::RangeInclusive;

use crate::ClientContext;
use crate::error::ClientResult;
use std::io::Cursor;

const COMPRESSION_LEVELS: RangeInclusive<i32> = 0..=21;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfCompress {
    /// Uncompressed data. Must be encoded as base64.
    pub uncompressed: String,
    /// Compression level, from 0 to 21.
    /// Where:
    /// 0 - default compression level (currently `3`);
    /// 1 - lowest compression level (fastest compression);
    /// 21 - highest compression level (slowest compression).
    pub level: i32,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfCompress {
    /// Compressed data. Must be encoded as base64.
    pub compressed: String,
}

/// Compresses data using Zstandard algorithm
#[api_function]
pub fn compress(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfCompress,
) -> ClientResult<ResultOfCompress> {
    let uncompressed = base64::decode(&params.uncompressed)
        .map_err(
            |err|
                super::errors::Error::compression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let compressed = compress_zstd(uncompressed.as_slice(), params.level)?;

    Ok(ResultOfCompress {
        compressed: base64::encode(&compressed),
    })
}

/// Compresses data using Zstandard algorithm. Useful for Rust API.
pub fn compress_zstd(uncompressed: &[u8], level: i32) -> ClientResult<Vec<u8>> {
    if !COMPRESSION_LEVELS.contains(&level) {
        return Err(
            super::errors::Error::compression_error(
                format!("Invalid compression level: {}", level)
            )
        );
    }

    let mut compressed = Vec::new();
    zstd::stream::copy_encode(
        &mut Cursor::new(uncompressed),
        &mut compressed,
        level
    ).map_err(|err| super::errors::Error::compression_error(err))?;

    Ok(compressed)
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfDecompress {
    /// Compressed data. Must be encoded as base64.
    pub compressed: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfDecompress {
    /// Decompressed data. Must be encoded as base64.
    pub decompressed: String,
}

/// Decompresses data using Zstandard algorithm
#[api_function]
pub fn decompress(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfDecompress,
) -> ClientResult<ResultOfDecompress> {
    let compressed = base64::decode(&params.compressed)
        .map_err(
            |err|
                super::errors::Error::decompression_error(format!("Unable to decode BASE64: {}", err))
        )?;

    let decompressed = decompress_zstd(compressed.as_slice())?;

    Ok(ResultOfDecompress {
        decompressed: base64::encode(&decompressed),
    })
}

/// Decompresses data using Zstandard algorithm. Useful for Rust API.
pub fn decompress_zstd(compressed: &[u8]) -> ClientResult<Vec<u8>> {
    let mut decompressed = Vec::new();
    zstd::stream::copy_decode(&mut Cursor::new(compressed), &mut decompressed)
        .map_err(|err| super::errors::Error::decompression_error(err))?;

    Ok(decompressed)
}
