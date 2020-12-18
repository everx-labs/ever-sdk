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

use std::sync::Arc;
use crate::client::{ClientEnv, FetchMethod};
use crate::error::ClientResult;
use crate::net::Error;

pub(crate) struct ServerVersion {
    pub version: u64,
    pub supports_time: bool,
}

impl ServerVersion {
    pub fn from_version(version: &str) -> ton_types::Result<Self> {
        let mut vec: Vec<&str> = version.split(".").collect();
        vec.resize(3, "0");
        let version = u64::from_str_radix(vec[0], 10)? * 1000000
            + u64::from_str_radix(vec[1], 10)? * 1000
            + u64::from_str_radix(vec[2], 10)?;

        Ok(ServerVersion {
            version,
            supports_time: version >= 26003,
        })
    }
}

pub(crate) struct ServerInfo {
    pub query_url: String,
    pub subscription_url: String,
    pub server_version: ServerVersion,
}

impl ServerInfo {
    pub fn expand_address(base_url: &str) -> String {
        let base_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
            base_url.to_owned()
        } else {
            format!("https://{}", base_url)
        };

        format!("{}/graphql", base_url)
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

        let server_version = ServerVersion::from_version(version).map_err(|err| {
            Error::invalid_server_response(format!("Can not parse version {}: {}", version, err))
        })?;

        let query_url = response
            .url
            .trim_end_matches("?query=%7Binfo%7Bversion%7D%7D")
            .to_owned();
        let subscription_url = query_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        Ok(Self {
            query_url,
            subscription_url,
            server_version,
        })
    }
}

