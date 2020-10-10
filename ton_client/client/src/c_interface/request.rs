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
use crate::{CResponseHandler, StringData, ResponseHandler, ResponseType};
use serde::Serialize;

enum ResponseHandlerImpl {
    Rust(ResponseHandler),
    C(CResponseHandler),
}

pub struct Request {
    request_id: u32,
    response_handler: ResponseHandlerImpl,
}

impl Request {
    pub(crate) fn new(request_id: u32, response_handler: ResponseHandler) -> Self {
        Self {
            request_id,
            response_handler: ResponseHandlerImpl::Rust(response_handler),
        }
    }

    pub(crate) fn new_with_c_handler(request_id: u32, response_handler: CResponseHandler) -> Self {
        Self {
            request_id,
            response_handler: ResponseHandlerImpl::C(response_handler),
        }
    }

    pub fn response(&self, params: impl Serialize, response_type: u32) {
        self.response_serialize(params, response_type, false);
    }

    pub fn response_result(&self, result: ClientResult<impl Serialize>) {
        self.response_result_with_finished(result, false)
    }

    pub fn finish_with_result(&self, result: ClientResult<impl Serialize>) {
        self.response_result_with_finished(result, true);
    }

    pub fn finish_with_error(&self, error: ClientError) {
        self.response_serialize(error, ResponseType::Error as u32, true);
    }

    fn response_result_with_finished(&self, result: ClientResult<impl Serialize>, finished: bool) {
        match result {
            Ok(success) => self.response_serialize(success, ResponseType::Success as u32, finished),
            Err(error) => self.response_serialize(error, ResponseType::Error as u32, finished),
        }
    }

    fn response_serialize(&self, params: impl Serialize, response_type: u32, finished: bool) {
        match serde_json::to_string(&params) {
            Ok(result) => self.call_response_handler(result, response_type, finished),
            Err(_) => self.call_response_handler(
                crate::client::errors::CANNOT_SERIALIZE_RESULT.into(),
                response_type,
                false,
            ),
        };
    }

    fn call_response_handler(&self, params_json: String, response_type: u32, finished: bool) {
        match self.response_handler {
            ResponseHandlerImpl::Rust(handler) => {
                handler(self.request_id, params_json, response_type, finished)
            }
            ResponseHandlerImpl::C(handler) => handler(
                self.request_id,
                StringData::new(&params_json),
                response_type,
                finished,
            ),
        }
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        self.call_response_handler("".into(), ResponseType::Nop as u32, true)
    }
}
