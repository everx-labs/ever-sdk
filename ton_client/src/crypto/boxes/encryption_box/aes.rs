/*
* Copyright 2018-2021 TON Labs LTD.
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

use std::sync::Arc;
use aes::{Aes128, Aes192, Aes256, BlockCipher, BlockDecrypt, BlockEncrypt, NewBlockCipher};
use block_modes::{BlockMode, Cbc};
use crate::ClientContext;

use crate::crypto::Error;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;
use super::{CipherMode, EncryptionBox, EncryptionBoxInfo};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct AesParams {
    pub mode: CipherMode,
    pub key: String,
    pub iv: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AesInfo {
    pub mode: CipherMode,
    pub iv: Option<String>,
}

pub(crate) struct AesEncryptionBox {
    key: Vec<u8>,
    mode: CipherMode,
    iv: Vec<u8>,
}

impl AesEncryptionBox {
    pub fn new(params: AesParams) -> ClientResult<Self> {
        let iv_required = match params.mode {
            CipherMode::CBC => true,
            _ => return Err(Error::unsupported_cipher_mode(&format!("{:?}", params.mode)))
        };
        if iv_required && params.iv.is_none() {
            return Err(Error::iv_required(&params.mode));
        }
        let key = hex_decode(&params.key)?;
        if  key.len() != 16 &&
            key.len() != 24 &&
            key.len() != 32
        {
            return Err(Error::invalid_key_size(key.len(), &[128, 192, 256]));
        }
        let iv = params.iv
            .map(|string| {
                let iv = hex_decode(&string)?;
                if iv.len() == aes::BLOCK_SIZE {
                    Ok(iv)
                } else {
                    Err(Error::invalid_iv_size(iv.len(), aes::BLOCK_SIZE))
                }
            })
            .transpose()?
            .unwrap_or_default();
        
        Ok(Self { key, iv, mode: params.mode })
    }

    fn create_block_mode<C, B>(key: &[u8], iv: &[u8]) -> ClientResult<B>
    where
        C: BlockCipher + BlockEncrypt + BlockDecrypt + NewBlockCipher,
        B: BlockMode<C, block_modes::block_padding::ZeroPadding>
    {
        B::new_from_slices(key, iv)
            .map_err(|err| Error::cannot_create_cipher(err))
    }

    fn encrypt_data<'a, C, B>(key: &[u8], iv: &[u8], data: &'a mut [u8], size: usize) -> ClientResult<&'a [u8]>
    where
        C: BlockCipher + BlockEncrypt + BlockDecrypt + NewBlockCipher,
        B: BlockMode<C, block_modes::block_padding::ZeroPadding>
    {
        Self::create_block_mode::<C, B>(key, iv)?
            .encrypt(data, size)
            .map_err(|err| Error::encrypt_data_error(format!("{:#?}", err)))
    }

    fn decrypt_data<C, B>(key: &[u8], iv: &[u8], data: &mut [u8]) -> ClientResult<()>
    where
        C: BlockCipher + BlockEncrypt + BlockDecrypt + NewBlockCipher,
        B: BlockMode<C, block_modes::block_padding::ZeroPadding>
    {
        Self::create_block_mode::<C, B>(key, iv)?
            .decrypt(data)
            .map_err(|err| Error::decrypt_data_error(format!("{:#?}", err)))
            .map(|_| ())
    }

    fn decode_base64_aligned(data: &str, align: usize) -> ClientResult<(Vec<u8>, usize)> {
        let data_size = (data.len() + 3) / 4 * 3;
        let aligned_size = (data_size + align - 1) & !(align - 1);

        let mut vec = vec![0u8; aligned_size];

        let size = base64::decode_config_slice(data, base64::STANDARD, &mut vec)
            .map_err(|err| crate::client::Error::invalid_base64(data, err))?;

        Ok((vec, size))
    }
}

#[async_trait::async_trait]
impl EncryptionBox for AesEncryptionBox {
    /// Gets encryption box information
    async fn get_info(&self, _context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo> {
        let iv = if self.iv.len() != 0 {
            Some(hex::encode(&self.iv))
        } else {
            None
        };

        let aes_info = AesInfo {
            mode: self.mode.clone(),
            iv
        };

        Ok(EncryptionBoxInfo {
            algorithm: Some("AES".to_owned()),
            hdpath: None,
            public: None,
            options: Some(json!(aes_info))
        })
    }
    /// Encrypts data
    async fn encrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        let (mut data, size) = Self::decode_base64_aligned(data, aes::BLOCK_SIZE)?;
        let result = match (self.key.len(), &self.mode) {
            (16, CipherMode::CBC) => Self::encrypt_data::<Aes128, Cbc<Aes128, _>>(&self.key, &self.iv, &mut data, size)?,
            (24, CipherMode::CBC) => Self::encrypt_data::<Aes192, Cbc<Aes192, _>>(&self.key, &self.iv, &mut data, size)?,
            (32, CipherMode::CBC) => Self::encrypt_data::<Aes256, Cbc<Aes256, _>>(&self.key, &self.iv, &mut data, size)?,
            _ => return Err(Error::unsupported_cipher_mode(&format!("{:?}", self.mode))),
        };
        Ok(base64::encode(result))
    }
    /// Decrypts data
    async fn decrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        let mut data = base64_decode(data)?;
        match (self.key.len(), &self.mode) {
            (16, CipherMode::CBC) => Self::decrypt_data::<Aes128, Cbc<Aes128, _>>(&self.key, &self.iv, &mut data)?,
            (24, CipherMode::CBC) => Self::decrypt_data::<Aes192, Cbc<Aes192, _>>(&self.key, &self.iv, &mut data)?,
            (32, CipherMode::CBC) => Self::decrypt_data::<Aes256, Cbc<Aes256, _>>(&self.key, &self.iv, &mut data)?,
            _ => return Err(Error::unsupported_cipher_mode(&format!("{:?}", self.mode))),
        }
        Ok(base64::encode(&data))
    }
}
