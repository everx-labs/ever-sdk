use crate::error::ClientResult;
use crate::ClientContext;

const COMPRESSION_LEVEL: i32 = 21;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfCompress {
    // Uncompressed data encoded in BASE64
    uncompressed: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfCompress {
    // Compressed data encoded in BASE64
    compressed: String,
}

#[api_function]
pub fn compress(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfCompress,
) -> ClientResult<ResultOfCompress> {
    let mut compressed = Vec::new();
    let uncompressed = base64::decode(&params.uncompressed)
        .map_err(|err| super::errors::Error::compression_error(err))?;
    zstd::stream::copy_encode(&mut uncompressed.as_slice(), &mut compressed, COMPRESSION_LEVEL)
        .map_err(|err| super::errors::Error::compression_error(err))?;

    Ok(ResultOfCompress {
        compressed: base64::encode(&compressed),
    })
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfDecompress {
    // Compressed data encoded in BASE64
    compressed: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfDecompress {
    // Decompressed data encoded in BASE64
    decompressed: String,
}

#[api_function]
pub fn decompress(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfDecompress,
) -> ClientResult<ResultOfDecompress> {
    let mut decompressed = Vec::new();
    let compressed = base64::decode(&params.compressed)
        .map_err(|err| super::errors::Error::decompression_error(err))?;
    zstd::stream::copy_decode(&mut compressed.as_slice(), &mut decompressed)
        .map_err(|err| super::errors::Error::decompression_error(err))?;

    Ok(ResultOfDecompress {
        decompressed: base64::encode(&decompressed),
    })
}