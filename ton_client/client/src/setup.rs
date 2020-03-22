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
use ton_sdk::{NodeClientConfig};

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
    pub transaction_timeout: Option<u32>,
}

#[cfg(feature = "node_interaction")]
fn setup(context: &mut ClientContext, config: SetupParams) -> ApiResult<()> {
    debug!("-> setup({}, {})",
        config.base_url.clone().unwrap_or("".to_owned()),
        config.transaction_timeout.unwrap_or(0));
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
        transaction_timeout: config.transaction_timeout,
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
