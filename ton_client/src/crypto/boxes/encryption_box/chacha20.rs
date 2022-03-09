use std::sync::Arc;
use chacha20::cipher::{NewStreamCipher, SyncStreamCipher};
use chacha20::{Key, Nonce};

use zeroize::ZeroizeOnDrop;

use crate::ClientContext;
use crate::crypto::{EncryptionBox, EncryptionBoxInfo};
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq, ZeroizeOnDrop)]
pub struct ChaCha20ParamsEB {
    /// 256-bit key. Must be encoded with `hex`.
    pub key: String,
    /// 96-bit nonce. Must be encoded with `hex`.
    pub nonce: String,
}

#[derive(Debug)]
pub(crate) struct ChaCha20EncryptionBox {
    key: Key,
    nonce: Nonce,
    hdpath: Option<String>,
}

impl ChaCha20EncryptionBox {
    pub fn new(params: ChaCha20ParamsEB, hdpath: Option<String>) -> ClientResult<Self> {
        let key = Key::clone_from_slice(&hex_decode(&params.key)?);
        let nonce = Nonce::clone_from_slice(&hex_decode(&params.nonce)?);

        Ok(Self { key, nonce, hdpath })
    }

    fn chacha20(&self, data: &str) -> ClientResult<String> {
        let mut cipher = chacha20::ChaCha20::new(&self.key, &self.nonce);
        let mut data = base64_decode(data)?;
        cipher.apply_keystream(&mut data);

        Ok(base64::encode(&data))
    }
}

#[async_trait::async_trait]
impl EncryptionBox for ChaCha20EncryptionBox {
    async fn get_info(&self, _context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            algorithm: Some("ChaCha20".to_owned()),
            hdpath: self.hdpath.clone(),
            public: None,
            options: Some(json!({ "nonce": hex::encode(&self.nonce) }))
        })
    }

    async fn encrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        self.chacha20(data)
    }

    async fn decrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        self.chacha20(data)
    }
}
