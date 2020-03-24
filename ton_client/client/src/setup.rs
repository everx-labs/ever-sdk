/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
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
#[cfg(feature = "node_interaction")]
use crate::types::ApiError;

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClientConfig, TimeoutsConfig};

pub(crate) fn register(handlers: &mut DispatchTable) {
    #[cfg(feature = "node_interaction")]
    handlers.call_no_args("uninit", |context| Ok(context.client = None));
    #[cfg(not(feature = "node_interaction"))]
    handlers.call_no_args("uninit", |_| Ok(()));

    handlers.call("setup", setup);
    handlers.call_no_args("version", |_|Ok(env!("CARGO_PKG_VERSION")));
}


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct SetupParams {
    pub base_url: Option<String>,
    pub message_retries_count: Option<u8>,
    pub message_expiration_timeout: Option<u32>,
    pub message_expiration_timeout_grow_factor: Option<f32>,
    pub message_processing_timeout: Option<u32>,
    pub message_processing_timeout_grow_factor: Option<f32>,
    pub wait_for_timeout: Option<u32>,
    pub access_key: Option<String>,
}

impl Into<TimeoutsConfig> for &SetupParams {
    fn into(self) -> TimeoutsConfig {
        let default = TimeoutsConfig::default();
        TimeoutsConfig {
            message_retries_count: self.message_retries_count.unwrap_or(default.message_retries_count),
            message_expiration_timeout: self.message_expiration_timeout.unwrap_or(default.message_expiration_timeout),
            message_expiration_timeout_grow_factor: self.message_expiration_timeout_grow_factor.unwrap_or(default.message_expiration_timeout_grow_factor),
            message_processing_timeout: self.message_processing_timeout.unwrap_or(default.message_processing_timeout),
            message_processing_timeout_grow_factor: self.message_processing_timeout_grow_factor.unwrap_or(default.message_processing_timeout_grow_factor),
            wait_for_timeout: self.wait_for_timeout.unwrap_or(default.wait_for_timeout),
        }
    }
}

#[cfg(feature = "node_interaction")]
fn setup(context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {
    // if node address is not provided don't init network connection
    if config.base_url.is_none() {
       return Ok(());
    }

    fn replace_prefix(s: &String, prefix: &str, new_prefix: &str) -> String {
        format!("{}{}", new_prefix, s[prefix.len()..].to_string())
    }

    fn resolve_url(configured: Option<&String>, default: &str) -> String {
        let url = configured.unwrap_or(&default.to_string()).to_lowercase().trim().to_string();
        if url.starts_with("http://") ||
            url.starts_with("https://") ||
            url.starts_with("ws://") ||
            url.starts_with("wss://")
        {
            url
        } else {
            format!("https://{}", url)
        }
    }

    let base_url = resolve_url(
        config.base_url.as_ref(),
        "",
    );


    let queries_url = format!("{}/graphql", base_url);

    let subscriptions_url = if queries_url.starts_with("https://") {
        replace_prefix(&queries_url, "https://", "wss://")
    } else {
        replace_prefix(&queries_url, "http://", "ws://")
    };

    let internal_config = NodeClientConfig {
        queries_server: queries_url,
        subscriptions_server: subscriptions_url,
        timeouts: Some((&config).into()),
        access_key: config.access_key
    };

    context.client = Some(ton_sdk::init(internal_config).map_err(|err|ApiError::config_init_failed(err))?);
    context.runtime = Some(tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|err| ApiError::cannot_create_runtime(err))?);

    Ok(())
}


#[cfg(not(feature = "node_interaction"))]
fn setup(_context: &mut ClientContext, _config: SetupParams) -> ApiResult<()> {
    Ok(())
}
