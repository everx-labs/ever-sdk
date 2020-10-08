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

use crate::error::{ClientError, ClientResult};
use crate::{ResponseHandler, ResponseType, StringData};
use serde::Serialize;

pub struct Request {
    response_handler: ResponseHandler,
    request_id: u32,
}

impl Request {
    pub fn new(response_handler: ResponseHandler, request_id: u32) -> Self {
        Self {
            response_handler,
            request_id,
        }
    }

    fn call_response_handler(
        &self,
        params_json: impl Serialize,
        response_type: u32,
        finished: bool,
    ) {
        match serde_json::to_string(&params_json) {
            Ok(result) => (self.response_handler)(
                self.request_id,
                StringData::from(&result),
                response_type,
                finished,
            ),
            Err(_) => (self.response_handler)(
                self.request_id,
                StringData::from(crate::client::errors::CANNOT_SERIALIZE_RESULT),
                response_type,
                false,
            ),
        };
    }

    pub fn send_result(&self, result: ClientResult<impl Serialize>, finished: bool) {
        match result {
            Ok(result) => {
                self.call_response_handler(result, ResponseType::Success as u32, finished)
            }
            Err(err) => self.call_response_handler(err, ResponseType::Error as u32, finished),
        }
    }

    pub fn finish_with(&self, result: ClientResult<impl Serialize>) {
        self.send_result(result, true);
    }

    pub fn finish_with_error(&self, error: ClientError) {
        self.call_response_handler(error, ResponseType::Error as u32, true);
    }

    pub fn send_response(&self, result: impl Serialize, response_type: u32) {
        self.call_response_handler(result, response_type, false);
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        (self.response_handler)(self.request_id, "".into(), ResponseType::Nop as u32, true)
    }
}
