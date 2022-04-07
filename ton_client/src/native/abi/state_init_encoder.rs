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
use serde_json::Value;
use ton_types::Cell;

pub struct StateInitEncoder {
    /// Content of TVC file.
    pub tvc: Cell,

    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,

    /// List of initial values for contract's public variables.
    pub initial_data: Option<Value>,

    /// Optional public key that can be provided in deploy set in order to substitute one
    /// in TVM file or provided by Signer.
    ///
    /// Public key resolving priority:
    /// 1. Public key from deploy set.
    /// 2. Public key, specified in TVM file.
    /// 3. Public key, provided by Signer.
    pub initial_pubkey: Option<String>,
}

