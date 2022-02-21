use std::sync::Arc;

use failure::bail;
use lockfree::map::ReadGuard;
use ton_block::{Deserializable, Serializable};
use ton_types::{BuilderData, IBitstring, SliceData};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::ClientContext;
use crate::crypto::{CryptoConfig, Error};
use crate::crypto::boxes::crypto_box::encryption::{decrypt_secret, encrypt_secret};
use crate::crypto::mnemonic::mnemonics;
use crate::encoding::base64_decode;
use crate::error::ClientResult;

mod encryption;

type PasswordProvider = Arc<dyn AppPasswordProvider + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, ApiType, Default, PartialEq)]
pub struct CryptoBoxHandle(pub u32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, ApiType, Default, PartialEq, Zeroize)]
pub struct CryptoBoxDictionary(pub u8);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct RegisteredCryptoBox {
    pub handle: CryptoBoxHandle,
}

#[derive(ZeroizeOnDrop)]
pub struct ResultOfGetPassword {
    /// User's password hash.
    /// Crypto box uses this password to decrypt its secret (seed phrase).
    /// Password is encrypted with `encryption_public_key`.
    pub encrypted_password: Vec<u8>,
    /// Public key of the key pair, used for password encryption in client application.
    pub app_encryption_pubkey: sodalite::BoxPublicKey,
}

#[async_trait::async_trait]
pub trait AppPasswordProvider {
    async fn get_password(
        &self,
        encryption_public_key: &sodalite::BoxPublicKey,
    ) -> ClientResult<ResultOfGetPassword>;
}

#[derive(ZeroizeOnDrop)]
pub(crate) struct SecretString(String);

#[derive(Default, Clone, ZeroizeOnDrop)]
pub(crate) struct SecretBuf(Vec<u8>);

enum SecretInternal {
    SeedPhrase {
        phrase: SecretString,
    },
}

impl Default for SecretInternal {
    fn default() -> Self {
        SecretInternal::SeedPhrase { phrase: SecretString(String::new()) }
    }
}

const SECRET_SEED_PHRASE: u8 = 0;

impl Serializable for SecretInternal {
    fn write_to(&self, cell: &mut BuilderData) -> ton_types::Result<()> {
        match self {
            SecretInternal::SeedPhrase { phrase } => {
                cell.append_u8(SECRET_SEED_PHRASE)?;
                cell.append_u16(phrase.0.len() as u16)?;
                cell.append_bitstring(phrase.0.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl Deserializable for SecretInternal {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let secret_type = slice.get_next_byte()?;
        match secret_type {
            SECRET_SEED_PHRASE => {
                let len = slice.get_next_u16()?;
                let phrase = SecretString(
                    String::from_utf8(
                        slice.get_next_bytes(len as usize)?
                    )?
                );
                *self = SecretInternal::SeedPhrase { phrase };
            }

            _ => bail!("Unsupported secret_type: {}", secret_type),
        }
        Ok(())
    }
}

pub(crate) struct CryptoBox {
    pub password_provider: PasswordProvider,
    pub secret_encryption_salt: SecretString,
    pub encrypted_secret: SecretBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq, ZeroizeOnDrop)]
#[serde(tag="type")]
pub enum CryptoBoxSecret {
    /// Generates new random seed phrase and wraps it into crypto box.
    /// This option can be used if the developer doesn't want the seed phrase to leave
    /// client's library memory, where it is immediately encrypted with salt and password, received
    /// from `AppPasswordProvider`.
    RandomSeedPhrase {
        dictionary: CryptoBoxDictionary,
        wordcount: u8,
    },

    /// Restores crypto box instance from an existing seed phrase.
    /// Takes the phrase, encrypts it with salt and password, received from `AppPasswordProvider`.
    /// After this save `encrypted_secret` and login via `encrypted_secret` the next time.
    PredefinedSeedPhrase {
        phrase: String,
    },

    /// Use specified encrypted secret. This type is used when user already logged in before.
    /// Can be only retrieved via `get_crypto_box_info`.
    EncryptedSecret {
        /// It is an object, containing seed phrase or private key (now we support only seed phrase).
        encrypted_secret: String,
    },
}

impl Default for CryptoBoxSecret {
    fn default() -> Self {
        Self::PredefinedSeedPhrase { phrase: Default::default() }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, ApiType, PartialEq, ZeroizeOnDrop)]
pub struct ParamsOfCreateCryptoBox {
    /// Salt used for secret encryption.
    /// Crypto box stores all secret information in encrypted form.
    /// For example, a mobile device can use device ID as salt.
    pub secret_encryption_salt: String,

    /// Secret.
    pub secret: CryptoBoxSecret,
}

/// Create Crypto Box
pub async fn create_crypto_box(
    context: Arc<ClientContext>,
    params: ParamsOfCreateCryptoBox,
    password_provider: PasswordProvider,
) -> ClientResult<RegisteredCryptoBox> {
    let encrypted_secret = match &params.secret {
        CryptoBoxSecret::RandomSeedPhrase { dictionary, wordcount } => {
            let config = CryptoConfig {
                mnemonic_dictionary: dictionary.0,
                mnemonic_word_count: *wordcount,
                ..Default::default()
            };
            let phrase = {
                let mnemonics = mnemonics(&config, Some(dictionary.0), Some(*wordcount))?;
                SecretString(mnemonics.generate_random_phrase()?)
            };
            encrypt_secret(
                phrase.0.as_bytes(),
                &password_provider,
                &params.secret_encryption_salt,
            ).await?
        },

        CryptoBoxSecret::PredefinedSeedPhrase { phrase } => encrypt_secret(
            phrase.as_bytes(),
            &password_provider,
            &params.secret_encryption_salt,
        ).await?,

        CryptoBoxSecret::EncryptedSecret { encrypted_secret } =>
            SecretBuf(base64_decode(&encrypted_secret)?),
    };

    let crypto_box = CryptoBox {
        password_provider,
        secret_encryption_salt: SecretString(params.secret_encryption_salt.clone()),
        encrypted_secret,
    };
    let id = context.get_next_id();
    assert!(context.boxes.crypto_boxes.insert(id, crypto_box).is_none());

    Ok(RegisteredCryptoBox { handle: CryptoBoxHandle(id) })
}

/// Remove Crypto Box
#[api_function]
pub async fn remove_crypto_box(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<()> {
    context.boxes.crypto_boxes.remove(&params.handle.0);
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq, ZeroizeOnDrop)]
pub struct ResultOfGetCryptoBoxSeedPhrase {
    pub phrase: String,
}

/// Get Crypto Box Seed Phrase
#[api_function]
pub async fn get_crypto_box_seed_phrase(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<ResultOfGetCryptoBoxSeedPhrase> {
    let SecretInternal::SeedPhrase { phrase } = {
        let guard = get_crypto_box(&context, &params.handle)?;

        SecretInternal::construct_from_bytes(
        &decrypt_secret(
                &guard.val().encrypted_secret.0,
                &guard.val().password_provider,
                &guard.val().secret_encryption_salt.0,
            ).await?.0
        ).map_err(|err| Error::crypto_box_secret_deserialization_error(err))?
    };

    Ok(ResultOfGetCryptoBoxSeedPhrase { phrase: phrase.0.clone() })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfGetCryptoBoxInfo {
    pub encrypted_secret: String,
}

/// Get Crypto Box Info
#[api_function]
pub async fn get_crypto_box_info(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<ResultOfGetCryptoBoxInfo> {
    Ok(ResultOfGetCryptoBoxInfo {
        encrypted_secret: base64::encode(&get_crypto_box(&context, &params.handle)?.val().encrypted_secret.0),
    })
}

fn get_crypto_box<'context>(context: &'context Arc<ClientContext>, handle: &CryptoBoxHandle)
    -> ClientResult<ReadGuard<'context, u32, CryptoBox>>
{
    context.boxes.crypto_boxes.get(&handle.0)
        .ok_or_else(|| Error::crypto_box_not_registered(handle.0))
}
