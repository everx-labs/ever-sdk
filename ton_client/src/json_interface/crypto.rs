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

 use crate::client::{AppObject, ClientContext, Error};
 use crate::error::ClientResult;
 use crate::crypto::{ResultOfRegisterSigningBox, SigningBox};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ParamsOfAppSigningBox {
    GetPublicKey,
    Sign {
        unsigned: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ResultOfAppSigningBox {
    GetPublicKey {
        public_key: String,
    },
    Sign {
        signature: String,
    },
}

struct ExternalSigningBox {
    app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>,
}

impl ExternalSigningBox {
    pub fn new(app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>) -> Self {
        Self { app_object }
    }
}

#[async_trait::async_trait]
impl SigningBox for ExternalSigningBox {
    async fn get_public_key(&self) -> ClientResult<Vec<u8>> {
        let response = self.app_object.call(ParamsOfAppSigningBox::GetPublicKey).await?;

        match response {
            ResultOfAppSigningBox::GetPublicKey { public_key } => {
               crate::encoding::hex_decode(&public_key)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxGetPublicKey", &response))
        }
    }

    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        let response = self.app_object.call(ParamsOfAppSigningBox::Sign { 
            unsigned: base64::encode(unsigned)
        }).await?;

        match response {
            ResultOfAppSigningBox::Sign { signature: signed } => {
               crate::encoding::hex_decode(&signed)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxSign", &response))
        }
    }
}

 /// Register an application implemented signing box.
#[api_function]
pub(crate) async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>,
) -> ClientResult<ResultOfRegisterSigningBox> {
    crate::crypto::register_signing_box(context, ExternalSigningBox::new(app_object)).await
}
