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
*/

use super::Error;
use crate::error::{ClientError, ClientResult};
use futures::{Sink, Stream};
use std::collections::HashMap;
use std::pin::Pin;

pub(crate) struct WebSocket {
    pub sender: Pin<Box<dyn Sink<String, Error = ClientError> + Send>>,
    pub receiver: Pin<Box<dyn Stream<Item = ClientResult<String>> + Send>>,
}

#[derive(Debug, Clone)]
pub(crate) struct FetchResult {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String,
    pub remote_address: Option<String>,
}

impl FetchResult {
    pub fn body_as_text(&self) -> ClientResult<&str> {
        Ok(&self.body)
    }

    pub fn body_as_json(&self) -> ClientResult<serde_json::Value> {
        self.check_success()?;
        let text = self.body_as_text()?;
        serde_json::from_str(text).map_err(|err| {
            Error::http_request_parse_error(format!("Body is not a valid JSON: {}\n{}", err, text))
        })
    }

    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn check_success(&self) -> ClientResult<()> {
        if self.is_success() {
            Ok(())
        } else {
            log::debug!(
                "Server responded with code {}. Body \n{}",
                self.status,
                self.body
            );
            if let Ok(json) = serde_json::from_str(&self.body) {
                if let Some(err) = crate::net::Error::try_extract_graphql_error(&json) {
                    return Err(err);
                }
            }
            Err(Error::http_request_send_error(format!(
                "Server responded with code {}: {}",
                self.status, self.body
            )))
        }
    }
}

#[allow(dead_code)]
pub(crate) enum FetchMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl FetchMethod {
    pub fn as_str(&self) -> &str {
        match self {
            FetchMethod::Get => "GET",
            FetchMethod::Post => "POST",
            FetchMethod::Put => "PUT",
            FetchMethod::Delete => "DELETE",
            FetchMethod::Head => "HEAD",
            FetchMethod::Options => "OPTIONS",
            FetchMethod::Connect => "CONNECT",
            FetchMethod::Patch => "PATCH",
            FetchMethod::Trace => "TRACE",
        }
    }
}
