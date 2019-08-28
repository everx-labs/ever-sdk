use dispatch::DispatchTable;
use client::Context;
use error::ClientResult;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call("contracts.abi.deploy", deploy);
}


#[derive(Deserialize)]
struct AbiDeployParams {
}

#[derive(Serialize)]
struct AbiDeployResult {
}


fn abi_deploy(context: &mut Context, config: AbiDeployParams) -> ClientResult<AbiDeployResult> {

}

