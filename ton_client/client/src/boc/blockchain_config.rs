use crate::boc::internal::deserialize_object_from_base64;
use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use ton_block::Serializable;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ParamsOfGetBlockchainConfig {
    /// Key block BOC encoded as base64
    pub block_boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ResultOfGetBlockchainConfig {
    /// Blockchain config BOC encoded as base64
    pub config_boc: String,
}

#[api_function]
pub fn get_blockchain_config(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetBlockchainConfig,
) -> ClientResult<ResultOfGetBlockchainConfig> {
    let object = deserialize_object_from_base64::<ton_block::Block>(&params.block_boc, "block")?;

    let extra = object
        .object
        .read_extra()
        .map_err(|err| Error::invalid_boc(format!("can not read `extra` from block: {}", err)))?;

    let master = extra
        .read_custom()
        .map_err(|err| Error::invalid_boc(format!("can not read `master` from block: {}", err)))?
        .ok_or(Error::inappropriate_block(
            "not a masterchain block. Only key block contains blockchain configuration",
        ))?;

    let config = master.config().ok_or(Error::inappropriate_block(
        "not a key block. Only key block contains blockchain configuration",
    ))?;

    let cell = config
        .write_to_new_cell()
        .map_err(|err| Error::serialization_error(err, "config to cells"))?;

    let bytes = ton_types::serialize_toc(&cell.into())
        .map_err(|err| Error::serialization_error(err, "config cells to bytes"))?;

    Ok(ResultOfGetBlockchainConfig {
        config_boc: base64::encode(&bytes),
    })
}
