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

use crate::client::{ClientContext};
use crate::error::ClientResult;
use crate::processing::{ParamsOfProcessMessage, ProcessingEvent, ProcessingResponseType, ResultOfProcessMessage, ParamsOfSendMessage, ResultOfSendMessage, ParamsOfWaitForTransaction};
use std::sync::Arc;
use super::request::Request;

/// Sends message to the network and monitors network for a result of
/// message processing.
#[api_function]
pub(crate) async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    request: std::sync::Arc<Request>,
) -> ClientResult<ResultOfProcessMessage> {
    let callback = move |event: ProcessingEvent| {
        request.send_response(event, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };
    crate::processing::process_message(context, params, callback).await
}


#[api_function]
pub(crate) async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfSendMessage> {
    let callback = move |result: ProcessingEvent| {
        callback.send_response(result, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };

    crate::processing::send_message::send_message(context, params, callback).await
}

/// Performs monitoring of the network for a results of the external
/// inbound message processing.
///
/// Note that presence of the `abi` parameter is critical for ABI
/// compliant contracts. Message processing uses drastically
/// different strategy for processing message with an ABI expiration
/// replay protection.
///
/// When the ABI header `expire` is present, the processing uses
/// `message expiration` strategy:
/// - The maximum block gen time is set to
///   `message_expiration_time + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `MessageExpired` error.
///
/// When the ABI header `expire` isn't present or `abi` parameter
/// isn't specified, the processing uses `transaction waiting`
/// strategy:
/// - The maximum block gen time is set to
///   `now() + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `Incomplete` result.
#[api_function]
pub(crate) async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfProcessMessage> {
    let callback = move |result: ProcessingEvent| {
        callback.send_response(result, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };
    crate::processing::wait_for_transaction(context, params, callback).await
}
