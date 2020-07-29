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

use crate::client::ClientContext;
use crate::dispatch::DispatchTable;
use crate::types::ApiResult;
use crate::types::ApiError;

use ton_sdk::{NodeClientConfig, TimeoutsConfig};

pub(crate) fn register(handlers: &mut DispatchTable) {
    #[cfg(feature = "node_interaction")]
    handlers.call_no_args("uninit", |context| Ok(context.client = None));
    #[cfg(not(feature = "node_interaction"))]
    handlers.call_no_args("uninit", |_| Ok(()));

    handlers.call("setup", setup);
    handlers.call_no_args("version", |_|Ok(env!("CARGO_PKG_VERSION")));
    handlers.call_no_args("context.get", |context|Ok(context.handle));
}


#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub(crate) struct SetupParams {
    pub base_url: Option<String>,
    pub servers: Option<Vec<String>>,
    pub message_retries_count: Option<u8>,
    pub message_expiration_timeout: Option<u32>,
    pub message_expiration_timeout_grow_factor: Option<f32>,
    pub message_processing_timeout: Option<u32>,
    pub wait_for_timeout: Option<u32>,
    pub access_key: Option<String>,
    pub out_of_sync_threshold: Option<i64>,
}

impl Into<NodeClientConfig> for SetupParams {
    fn into(self) -> NodeClientConfig {
        let default = TimeoutsConfig::default();
        NodeClientConfig {
            timeouts: Some(TimeoutsConfig {
                message_retries_count: self.message_retries_count.unwrap_or(default.message_retries_count),
                message_expiration_timeout: self.message_expiration_timeout.unwrap_or(default.message_expiration_timeout),
                message_expiration_timeout_grow_factor: self.message_expiration_timeout_grow_factor.unwrap_or(default.message_expiration_timeout_grow_factor),
                message_processing_timeout: self.message_processing_timeout.unwrap_or(default.message_processing_timeout),
                wait_for_timeout: self.wait_for_timeout.unwrap_or(default.wait_for_timeout),
                out_of_sync_threshold: self.out_of_sync_threshold.unwrap_or(default.out_of_sync_threshold),
            }),
            #[cfg(feature = "node_interaction")]
            base_url: self.base_url.or(self.servers.as_ref().unwrap_or(&vec![]).get(0).cloned()),
            #[cfg(feature = "node_interaction")]
            access_key: self.access_key,
        }
    }
}

#[cfg(feature = "node_interaction")]
fn setup(context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {

    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|err| ApiError::cannot_create_runtime(err))?;

    let client = runtime.block_on(ton_sdk::init(config.into()))
        .map_err(|err| crate::types::apierror_from_sdkerror(&err, ApiError::config_init_failed, None))?;

    context.client = Some(client);
    context.runtime = Some(runtime);

    Ok(())
}


#[cfg(not(feature = "node_interaction"))]
fn setup(context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {
    trace!("-> client.setup({:?})", config);

    context.client = Some(ton_sdk::init(config.into()).map_err(|err|ApiError::config_init_failed(err))?);
    Ok(())
}
