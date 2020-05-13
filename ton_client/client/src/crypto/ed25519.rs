/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::crypto::keys::KeyPair;
use crate::types::ApiResult;

pub fn generate_keypair() -> ApiResult<KeyPair> {
    let mut csprng = rand::thread_rng();
    let keypair = ed25519_dalek::Keypair::generate(&mut csprng);
    Ok(KeyPair::new(
        hex::encode(keypair.public.to_bytes()),
        hex::encode(keypair.secret.to_bytes()),
    ))
}
