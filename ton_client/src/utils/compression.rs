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

use std::io::Cursor;

use crate::error::ClientResult;

/// Compresses data using Zstandard algorithm
pub fn compress_zstd(uncompressed: &[u8], level: Option<i32>) -> ClientResult<Vec<u8>> {
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

/// Decompresses data using Zstandard algorithm
pub fn decompress_zstd(compressed: &[u8]) -> ClientResult<Vec<u8>> {
    let mut decompressed = Vec::new();
    zstd::stream::copy_decode(&mut Cursor::new(compressed), &mut decompressed)
        .map_err(|err| super::errors::Error::decompression_error(err))?;

    Ok(decompressed)
}
