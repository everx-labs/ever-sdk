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

 use super::request::Request;
 use crate::client::ClientContext;
 use crate::error::ClientResult;
 use crate::crypto::{Error, ParamsOfRegisterSigningBox, ResultOfRegisterSigningBox, SigningBox};
 use futures::Future;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum SigningBoxAppRequest {
    GetPublicKey,
    Sign {
        unsigned: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum SigningBoxAppResponse {
    SigningBoxGetPublicKey {
        public_key: String,
    },
    SigningBoxSign {
        signature: String,
    },
}

pub struct ExternalSigningBox<F, Fut>
where
    F: Fn(SigningBoxAppRequest) -> Fut + Send + Sync,
    Fut: Future<Output=ClientResult<SigningBoxAppResponse>> + Send + Sync + 'static,
{
    callback: F,
}

impl<F, Fut> ExternalSigningBox<F, Fut>
where
    F: Fn(SigningBoxAppRequest) -> Fut + Send + Sync,
    Fut: Future<Output=ClientResult<SigningBoxAppResponse>> + Send + Sync + 'static,
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait::async_trait]
impl<F, Fut> SigningBox for ExternalSigningBox<F, Fut>
where
    F: Fn(SigningBoxAppRequest) -> Fut + Send + Sync,
    Fut: Future<Output=ClientResult<SigningBoxAppResponse>> + Send + Sync + 'static,
{
    async fn get_public_key(&self) -> ClientResult<Vec<u8>> {
        let response = (self.callback)(SigningBoxAppRequest::GetPublicKey).await?;

        match response {
            SigningBoxAppResponse::SigningBoxGetPublicKey { public_key } => {
               crate::encoding::hex_decode(&public_key)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxResponse::SigningBoxGetPublicKey", &response))
        }
    }

    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        let response = (self.callback)(SigningBoxAppRequest::Sign { 
            unsigned: base64::encode(unsigned)
        }).await?;

        match response {
            SigningBoxAppResponse::SigningBoxSign { signature: signed } => {
               crate::encoding::hex_decode(&signed)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxResponse::SigningBoxSign", &response))
        }
    }
}

 /// Register an application implemented signing box.
#[api_function]
pub(crate) async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRegisterSigningBox,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfRegisterSigningBox> {
    let context_copy = context.clone();
    let object_ref = params.signing_box_ref;
    let callback = move |request| {
        let callback = callback.clone();
        let context = context_copy.clone();
        let object_ref = object_ref.clone();
        async move {
            context.app_request(&callback, object_ref, request).await
        }
    };
    
    crate::crypto::register_signing_box(context, ExternalSigningBox::new(callback)).await
}
