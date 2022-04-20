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

use ed25519_dalek::PublicKey;
use serde_json::Value;

pub struct FunctionHeader {
    pub expire: Option<u32>,
    pub time: Option<u64>,
    pub pubkey: Option<PublicKey>,
}

pub struct MessageBodyEncoder {
    /// Function name that is being called.
    /// Or function id encoded as string in hex (starting with 0x).
    pub function_name: String,

    /// Function header.
    ///
    /// If an application omits some header parameters required by the
    /// contract's ABI, the library will set the default values for
    /// them.
    pub header: Option<FunctionHeader>,

    /// Function input parameters according to ABI.
    pub input: Option<Value>,
}
