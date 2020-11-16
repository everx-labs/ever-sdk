use super::KeyPair;
use super::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::json_interface::request::Request;
use futures::Future;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct SigningBoxHandle(pub u32);

impl From<u32> for SigningBoxHandle {
    fn from(handle: u32) -> Self {
        Self(handle)
    }
}

#[async_trait::async_trait]
pub trait SigningBox {
    async fn get_public_key(&self) -> ClientResult<Vec<u8>>;
    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>>;
}

pub struct KeysSigningBox {
    key_pair: ed25519_dalek::Keypair
}

impl KeysSigningBox {
    pub fn new(key_pair: ed25519_dalek::Keypair) -> Self {
        Self {
            key_pair
        }
    }

    pub fn from_encoded(key_pair: KeyPair) -> ClientResult<Self> {
        key_pair.decode().map(|pair| Self::new(pair))
    }
}

#[async_trait::async_trait]
impl SigningBox for KeysSigningBox {
    async fn get_public_key(&self) -> ClientResult<Vec<u8>> {
        Ok(self.key_pair.public.to_bytes().to_vec())
    }

    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        super::internal::sign_using_keys(unsigned, &self.key_pair).map(|result| result.1)
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfGetSigningBox {
    /// Handle of the signing box.
    pub handle: SigningBoxHandle,
}

/// Gets a default signing box implementation.
#[api_function]
pub async fn get_signing_box(
    context: std::sync::Arc<ClientContext>,
    params: KeyPair,
) -> ClientResult<ResultOfGetSigningBox> {
    let id = context.get_next_id();
    let signing_box = KeysSigningBox::from_encoded(params)?;
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(ResultOfGetSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfRegisterSigningBox {
    /// Application defined reference to signing box implementation.
    pub signing_box_ref: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfRegisterSigningBox {
    /// Handle of the signing box.
    pub handle: SigningBoxHandle,
}

/// Register an application implemented signing box.
#[api_function]
pub async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRegisterSigningBox,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfRegisterSigningBox> {
    let id = context.get_next_id();
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
    let signing_box = ExternalSigningBox::new(callback);
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(ResultOfRegisterSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfSigningBoxGetPublicKey {
    /// Signing Box handle.
    pub signing_box: SigningBoxHandle,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfSigningBoxGetPublicKey {
    /// Public key of signing box. Encoded with hex
    pub pubkey: String,
}

/// Returns public key of signing key pair.
#[api_function]
pub async fn signing_box_get_public_key(
    context: Arc<ClientContext>,
    params: ParamsOfSigningBoxGetPublicKey,
) -> ClientResult<ResultOfSigningBoxGetPublicKey> {
    let signing_box = context.boxes.signing_boxes
        .get(&params.signing_box.0)
        .ok_or(Error::signing_box_not_registered(params.signing_box.0))?;

    let key = signing_box.1.get_public_key().await?;

    Ok(ResultOfSigningBoxGetPublicKey {
        pubkey: hex::encode(&key)
    })
}
    
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfSigningBoxSign {
    /// Signing Box handle.
    pub signing_box: SigningBoxHandle,
    /// Unsigned user data. Must be encoded with `base64`.
    pub unsigned: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfSigningBoxSign {
    /// Data signature. Encoded with `base64`.
    pub signature: String,
}

/// Returns signed user data.
#[api_function]
pub async fn signing_box_sign(
    context: Arc<ClientContext>,
    params: ParamsOfSigningBoxSign,
) -> ClientResult<ResultOfSigningBoxSign> {
    let signing_box = context.boxes.signing_boxes
        .get(&params.signing_box.0)
        .ok_or(Error::signing_box_not_registered(params.signing_box.0))?;

    let unsigned = crate::encoding::base64_decode(&params.unsigned)?;

    let signed = signing_box.1.sign(&unsigned).await?;

    Ok(ResultOfSigningBoxSign {
        signature: hex::encode(&signed)
    })
}

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
