use std::sync::Arc;

use zeroize::ZeroizeOnDrop;

use crate::ClientContext;
use crate::crypto::{
    EncryptionBox, EncryptionBoxInfo, nacl_secret_box, nacl_secret_box_open,
    ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen,
};
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq, ZeroizeOnDrop)]
pub struct NaclSecretBoxParams {
    /// Secret key - unprefixed 0-padded to 64 symbols hex string
    pub key: String,
    /// Nonce in `hex`
    pub nonce: String,
}

#[derive(Debug)]
pub(crate) struct NaclSecretEncryptionBox {
    params: NaclSecretBoxParams,
    hdpath: Option<String>,
}

impl NaclSecretEncryptionBox {
    pub fn new(params: NaclSecretBoxParams, hdpath: Option<String>) -> Self {
        Self { params, hdpath }
    }
}

#[async_trait::async_trait]
impl EncryptionBox for NaclSecretEncryptionBox {
    async fn get_info(&self, _context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            algorithm: Some("NaclSecretBox".to_owned()),
            hdpath: self.hdpath.clone(),
            public: None,
            options: Some(json!({ "nonce": hex::encode(&self.params.nonce) })),
        })
    }

    async fn encrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        nacl_secret_box(context, ParamsOfNaclSecretBox {
            decrypted: data.clone(),
            nonce: self.params.nonce.clone(),
            key: self.params.key.clone(),
        }).map(|result| result.encrypted)
    }

    async fn decrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        nacl_secret_box_open(context, ParamsOfNaclSecretBoxOpen {
            encrypted: data.clone(),
            nonce: self.params.nonce.clone(),
            key: self.params.key.to_string(),
        }).map(|result| result.decrypted)
    }
}
