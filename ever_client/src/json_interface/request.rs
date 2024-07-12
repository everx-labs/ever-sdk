/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

use crate::error::{ClientError, ClientResult};
use crate::{
    CResponseHandler, CResponseHandlerPtr, ResponseHandler, ResponseHandlerPtr, ResponseType,
    StringData,
};
use serde::Serialize;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

enum ResponseHandlerImpl {
    Rust(u32, ResponseHandler),
    C(u32, CResponseHandler),
    RustPtr(usize, ResponseHandlerPtr),
    CPtr(usize, CResponseHandlerPtr),
}

pub struct Request {
    response_handler: ResponseHandlerImpl,
    finished: AtomicBool,
}

impl Request {
    pub(crate) fn new(request_id: u32, response_handler: ResponseHandler) -> Self {
        Self {
            response_handler: ResponseHandlerImpl::Rust(request_id, response_handler),
            finished: AtomicBool::new(false),
        }
    }

    pub(crate) fn new_with_c_handler(request_id: u32, response_handler: CResponseHandler) -> Self {
        Self {
            response_handler: ResponseHandlerImpl::C(request_id, response_handler),
            finished: AtomicBool::new(false),
        }
    }

    pub(crate) fn new_with_ptr(
        request_ptr: *const (),
        response_handler: ResponseHandlerPtr,
    ) -> Self {
        Self {
            response_handler: ResponseHandlerImpl::RustPtr(request_ptr as usize, response_handler),
            finished: AtomicBool::new(false),
        }
    }

    pub(crate) fn new_with_c_handler_ptr(
        request_ptr: *const c_void,
        response_handler: CResponseHandlerPtr,
    ) -> Self {
        Self {
            response_handler: ResponseHandlerImpl::CPtr(request_ptr as usize, response_handler),
            finished: AtomicBool::new(false),
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

    fn set_finished(&self, finished: bool) -> bool {
        // We must not change finished flag if it is already finished.
        if self.finished.load(Ordering::Relaxed) {
            return true;
        }
        // We can change flag only `false` -> `true`
        if finished {
            self.finished.store(finished, Ordering::Relaxed);
        }
        return false;
    }

    fn call_response_handler(&self, params_json: String, response_type: u32, finished: bool) {
        let was_finished = self.set_finished(finished);
        if was_finished {
            return;
        }
        match self.response_handler {
            ResponseHandlerImpl::Rust(id, handler) => {
                handler(id, params_json, response_type, finished)
            }
            ResponseHandlerImpl::C(id, handler) => {
                handler(id, StringData::new(&params_json), response_type, finished)
            }
            ResponseHandlerImpl::RustPtr(ptr, handler) => {
                handler(ptr as *const (), params_json, response_type, finished)
            }
            ResponseHandlerImpl::CPtr(ptr, handler) => handler(
                ptr as *const c_void,
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
