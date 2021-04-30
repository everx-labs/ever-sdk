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

#[derive(Clone)]
pub(crate) struct Endpoint {
    pub query_url: String,
    pub subscription_url: String,
    pub server_version: u64,
    pub server_time_delta: i64,
}

impl Endpoint {
    pub fn http_headers() -> Vec<(String, String)> {
        vec![("tonclient-core-version".to_string(), core_version())]
    }

    fn expand_address(base_url: &str) -> String {
        let base_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
            base_url.to_owned()
        } else {
            format!("https://{}", base_url)
        };

        format!("{}/graphql", base_url.trim_end_matches("/"))
    }

    pub async fn resolve(client_env: Arc<ClientEnv>, address: &str) -> ClientResult<Self> {
        let address = Self::expand_address(address);
        let start = client_env.now_ms() as i64;
        let query = "?query=%7Binfo%7Bversion%20time%7D%7D";
        let response = client_env
            .fetch(
                &format!("{}{}", address, query),
                FetchMethod::Get,
                None,
                None,
                None,
            )
            .await?;
        let response_body = response.body_as_json()?;
        let end = client_env.now_ms() as i64;

        let server_version = response_body["data"]["info"]["version"].as_str().ok_or(
            Error::invalid_server_response(format!("No version in response: {}", response_body)),
        )?;
        let server_time = response_body["data"]["info"]["time"].as_i64().ok_or(
            Error::invalid_server_response(format!("No time in response: {}", response_body)),
        )?;

        let server_time_delta = server_time - (start + (end - start) / 2);

        let query_url = response
            .url
            .trim_end_matches(query)
            .to_owned();
        let subscription_url = query_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        let mut parts: Vec<&str> = server_version.split(".").collect();
        parts.resize(3, "0");
        let parse_part = |i: usize| {
            u64::from_str_radix(parts[i], 10).map_err(|err| {
                Error::invalid_server_response(format!(
                    "Can not parse version {}: {}",
                    server_version, err
                ))
            })
        };
        let server_version = parse_part(0)? * 1000000 + parse_part(1)? * 1000 + parse_part(2)?;

        Ok(Self {
            query_url,
            subscription_url,
            server_time_delta,
            server_version,
        })
    }
}
