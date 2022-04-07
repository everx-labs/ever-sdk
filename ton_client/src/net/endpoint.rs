/*
 * Copyright 2018-2021 TON Labs LTD.
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
use crate::net::{Error, NetworkConfig};
use serde_json::Value;
use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};

const V_0_39_0: u32 = 39000;
const BOC_VERSION: &str = "2";

pub(crate) struct Endpoint {
    pub query_url: String,
    pub subscription_url: String,
    pub ip_address: Option<String>,
    pub server_version: AtomicU32,
    pub server_time_delta: AtomicI64,
    pub server_latency: AtomicU64,
    pub next_latency_detection_time: AtomicU64,
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        Self {
            query_url: self.query_url.clone(),
            subscription_url: self.subscription_url.clone(),
            ip_address: self.ip_address.clone(),
            server_version: AtomicU32::new(self.server_version.load(Ordering::Relaxed)),
            server_time_delta: AtomicI64::new(self.server_time_delta.load(Ordering::Relaxed)),
            server_latency: AtomicU64::new(self.server_latency.load(Ordering::Relaxed)),
            next_latency_detection_time: AtomicU64::new(
                self.next_latency_detection_time.load(Ordering::Relaxed),
            ),
        }
    }
}

const QUERY_INFO_SCHEMA: &str = "?query=%7Binfo%7Bversion%20time%7D%7D";
const QUERY_INFO_METRICS: &str = "?query=%7Binfo%7Bversion%20time%20latency%7D%7D";

const HTTP_PROTOCOL: &str = "http://";
const HTTPS_PROTOCOL: &str = "https://";

impl Endpoint {
    pub fn http_headers() -> Vec<(String, String)> {
        vec![
            ("tonclient-core-version".to_string(), core_version()),
            (
                "X-Evernode-Expected-Account-Boc-Version".to_string(),
                BOC_VERSION.to_owned(),
            ),
        ]
    }

    fn expand_address(base_url: &str) -> String {
        let url = base_url.trim_end_matches("/").to_lowercase();
        let base_url = if url.starts_with(HTTP_PROTOCOL) || url.starts_with(HTTPS_PROTOCOL) {
            base_url.to_owned()
        } else {
            let protocol = if url == "localhost" || url == "127.0.0.1" || url == "0.0.0.0" {
                HTTP_PROTOCOL
            } else {
                HTTPS_PROTOCOL
            };
            format!("{}{}", protocol, base_url)
        };

        format!("{}/graphql", base_url.trim_end_matches("/"))
    }

    async fn fetch_info_with_url(
        client_env: &ClientEnv,
        query_url: &str,
        query: &str,
        timeout: u32,
    ) -> ClientResult<(Value, String, Option<String>)> {
        let response = client_env
            .fetch(
                &format!("{}{}", query_url, query),
                FetchMethod::Get,
                None,
                None,
                timeout,
            )
            .await?;
        let query_url = response.url.trim_end_matches(query).to_owned();
        let info = response.body_as_json()?["data"]["info"].to_owned();
        Ok((info, query_url, response.remote_address))
    }

    pub async fn resolve(
        client_env: &ClientEnv,
        config: &NetworkConfig,
        address: &str,
    ) -> ClientResult<Self> {
        let address = Self::expand_address(address);
        let info_request_time = client_env.now_ms();
        let (info, query_url, ip_address) = Self::fetch_info_with_url(
            client_env,
            &address,
            QUERY_INFO_SCHEMA,
            config.query_timeout,
        )
        .await?;
        let subscription_url = query_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let endpoint = Self {
            query_url,
            subscription_url,
            ip_address,
            server_time_delta: AtomicI64::default(),
            server_version: AtomicU32::default(),
            server_latency: AtomicU64::default(),
            next_latency_detection_time: AtomicU64::default(),
        };
        endpoint.apply_server_info(client_env, config, info_request_time, &info)?;
        endpoint.refresh(client_env, config).await?;
        Ok(endpoint)
    }

    pub async fn refresh(
        &self,
        client_env: &ClientEnv,
        config: &NetworkConfig,
    ) -> ClientResult<()> {
        if self.version() >= V_0_39_0 {
            let info_request_time = client_env.now_ms();
            let (info, _, _) = Self::fetch_info_with_url(
                client_env,
                &self.query_url,
                QUERY_INFO_METRICS,
                config.query_timeout,
            )
            .await?;
            self.apply_server_info(client_env, config, info_request_time, &info)?;
        }
        Ok(())
    }

    pub fn apply_server_info(
        &self,
        client_env: &ClientEnv,
        config: &NetworkConfig,
        info_request_time: u64,
        info: &Value,
    ) -> ClientResult<()> {
        if let Some(version) = info["version"].as_str() {
            let mut parts: Vec<&str> = version.split(".").collect();
            parts.resize(3, "0");
            let parse_part = |i: usize| {
                u32::from_str_radix(parts[i], 10).map_err(|err| {
                    Error::invalid_server_response(format!(
                        "Can not parse version {}: {}",
                        version, err
                    ))
                })
            };
            self.server_version.store(
                parse_part(0)? * 1000000 + parse_part(1)? * 1000 + parse_part(2)?,
                Ordering::Relaxed,
            );
        }
        if let Some(server_time) = info["time"].as_i64() {
            let now = client_env.now_ms();
            self.server_time_delta.store(
                server_time - ((info_request_time + now) / 2) as i64,
                Ordering::Relaxed,
            );
            if let Some(latency) = info["latency"].as_i64() {
                self.server_latency
                    .store(latency.abs() as u64, Ordering::Relaxed);
                self.next_latency_detection_time.store(
                    now as u64 + config.latency_detection_interval as u64,
                    Ordering::Relaxed,
                );
            }
        }
        Ok(())
    }

    pub fn latency(&self) -> u64 {
        self.server_latency.load(Ordering::Relaxed)
    }

    pub fn version(&self) -> u32 {
        self.server_version.load(Ordering::Relaxed)
    }

    pub fn time_delta(&self) -> i64 {
        self.server_time_delta.load(Ordering::Relaxed)
    }

    pub fn next_latency_detection_time(&self) -> u64 {
        self.next_latency_detection_time.load(Ordering::Relaxed)
    }
}
