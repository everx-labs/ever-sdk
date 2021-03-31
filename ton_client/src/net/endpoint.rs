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
 *
 */

use crate::client::{core_version, ClientEnv, FetchMethod};
use crate::error::ClientResult;
use crate::net::Error;
use std::sync::Arc;

pub(crate) struct Endpoint {
    pub query_url: String,
    pub subscription_url: String,
    pub server_version: u64,
    pub server_supports_time: bool,
    pub server_supports_endpoints: bool,
}

impl Endpoint {
    pub fn http_headers() -> Vec<(String, String)> {
        vec![("tonclient-core-version".to_string(), core_version())]
    }

    pub fn expand_address(base_url: &str) -> String {
        let base_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
            base_url.to_owned()
        } else {
            format!("https://{}", base_url)
        };

        format!("{}/graphql", base_url.trim_end_matches("/"))
    }

    pub async fn fetch(client_env: Arc<ClientEnv>, address: &str) -> ClientResult<Self> {
        let response = client_env
            .fetch(
                &format!("{}?query=%7Binfo%7Bversion%7D%7D", address),
                FetchMethod::Get,
                None,
                None,
                None,
            )
            .await?;
        let response_body = response.body_as_json()?;

        let version = response_body["data"]["info"]["version"].as_str().ok_or(
            Error::invalid_server_response(format!("No version in response: {}", response_body)),
        )?;

        let query_url = response
            .url
            .trim_end_matches("?query=%7Binfo%7Bversion%7D%7D")
            .to_owned();
        let subscription_url = query_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        let mut parts: Vec<&str> = version.split(".").collect();
        parts.resize(3, "0");
        let parse_part = |i: usize| {
            u64::from_str_radix(parts[i], 10).map_err(|err| {
                Error::invalid_server_response(format!(
                    "Can not parse version {}: {}",
                    version, err
                ))
            })
        };
        let version = parse_part(0)? * 1000000 + parse_part(1)? * 1000 + parse_part(2)?;

        Ok(Self {
            query_url,
            subscription_url,
            server_version: version,
            server_supports_time: version >= 26003,
            server_supports_endpoints: version >= 30000,
        })
    }
}
