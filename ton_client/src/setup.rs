use client::Context;
use dispatch::DispatchTable;
use error::{ClientResult, ClientError};
use ton_sdk::{NodeClientConfig, RequestsConfig, QueriesConfig};

const VERSION: &str = "0.10.0";

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call("setup", setup);
    handlers.call_no_args("version", |_|Ok(VERSION));
}


#[derive(Deserialize)]
pub(crate) struct SetupParams {
    pub defaultWorkchain: Option<i32>,
    pub baseUrl: Option<String>,
    pub requestsUrl: Option<String>,
    pub queriesUrl: Option<String>,
    pub subscriptionsUrl: Option<String>,
}


fn setup(context: &mut Context, config: SetupParams) -> ClientResult<()> {
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
        config.baseUrl.as_ref(),
        "services.tonlabs.io",
    );

    let requests_url = resolve_url(
        config.requestsUrl.as_ref(),
        &format!("{}/topics/requests", base_url),
    );


    let queries_url = resolve_url(
        config.queriesUrl.as_ref(),
        &format!("{}/graphql", base_url),
    );


    let subscriptions_url = resolve_url(
        config.subscriptionsUrl.as_ref(),
        &if queries_url.starts_with("https://") {
            replace_prefix(&queries_url, "https://", "wss://")
        } else {
            replace_prefix(&queries_url, "http://", "ws://")
        }
    );

    ton_sdk::init(config.defaultWorkchain, NodeClientConfig {
        requests_config: RequestsConfig {
            requests_server: requests_url,
        },
        queries_config: QueriesConfig {
            queries_server: queries_url,
            subscriptions_server: subscriptions_url
        }
    }).map_err(|err|ClientError::setup_failed(err))
}
