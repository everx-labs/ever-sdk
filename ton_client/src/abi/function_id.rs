use crate::client::ClientContext;
use crate::error::ClientResult;
use std::sync::Arc;

use super::{Abi, Error};

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfCalcFunctionId {
    /// Contract ABI.
    pub abi: Abi,
    /// Contract function name
    pub function_name: String,
    /// If set to `true` output function ID will be returned which is used in contract response.
    /// Default is `false`
    pub output: Option<bool>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfCalcFunctionId {
    /// Contract function ID
    pub function_id: u32,
}

/// Calculates contract function ID by contract ABI
#[api_function]
pub fn calc_function_id(
    _context: Arc<ClientContext>,
    params: ParamsOfCalcFunctionId,
) -> ClientResult<ResultOfCalcFunctionId> {
    let abi = params.abi.abi()?;
    let function = abi.function(&params.function_name)
        .map_err(|_| Error::invalid_function_name(&params.function_name))?;

    let function_id = if params.output.unwrap_or_default() {
        function.get_output_id()
    } else {
        function.get_input_id()
    };

    Ok(ResultOfCalcFunctionId { function_id })
}
