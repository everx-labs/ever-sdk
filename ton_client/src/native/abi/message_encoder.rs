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

use crate::native::abi::message_body_encoder::MessageBodyEncoder;
use crate::native::abi::state_init_encoder::StateInitEncoder;
use ton_block::{Grams, MsgAddressInt};

pub struct InternalMessageEncoder {
    /// Target address the message will be sent to.
    ///
    /// Must be specified in case of non-deploy message.
    pub address: Option<MsgAddressInt>,

    /// Source address of the message.
    pub src_address: Option<MsgAddressInt>,

    /// Deploy parameters.
    ///
    /// Must be specified in case of deploy message.
    pub state_init: Option<StateInitEncoder>,

    /// Function call parameters.
    ///
    /// Must be specified in case of non-deploy message.
    ///
    /// In case of deploy message it is optional and contains parameters
    /// of the functions that will to be called upon deploy transaction.
    pub body: Option<MessageBodyEncoder>,

    /// Value in nano tokens to be sent with message.
    pub value: Grams,

    /// Flag of bounceable message. Default is true.
    pub bounce: Option<bool>,

    /// Enable Instant Hypercube Routing for the message. Default is false.
    pub enable_ihr: Option<bool>,
}
