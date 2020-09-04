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
*/

use crate::dispatch::DispatchTable;

#[cfg(test)]
mod tests;

mod encode;
mod abi;
mod decode;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call("abi.encode_deploy_message", encode::encode_deploy_message);
    handlers.call("abi.encode_run_message", encode::encode_run_message);
    handlers.call("abi.encode_with_signature", encode::encode_with_signature);
}
