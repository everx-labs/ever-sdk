use std::sync::Arc;

use lockfree::map::ReadGuard;
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

#[derive(Serialize, Deserialize, Debug, PartialEq, ZeroizeOnDrop)]
pub(crate) struct SecretString(pub String);

#[derive(Debug, Default, Clone, ZeroizeOnDrop)]
pub(crate) struct SecretBuf(pub Vec<u8>);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, ZeroizeOnDrop)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum SecretInternal {
    SeedPhrase {
        phrase: SecretString,
    },
}

impl Default for SecretInternal {
    fn default() -> Self {
        SecretInternal::SeedPhrase { phrase: SecretString(String::new()) }
    }
}

pub(crate) struct CryptoBox {
    pub password_provider: PasswordProvider,
    pub secret_encryption_salt: SecretString,
    pub encrypted_secret: SecretBuf,
}

/// Crypto Box Secret.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq, ZeroizeOnDrop)]
#[serde(tag="type")]
pub enum CryptoBoxSecret {
    /// Creates Crypto Box from a random seed phrase.
    /// This option can be used if a developer doesn't want the seed phrase to leave the core
    /// library's memory, where it is stored encrypted.
    ///
    /// This type should be used upon the first wallet initialization, all further initializations
    /// should use `EncryptedSecret` type instead.
    ///
    /// Get `encrypted_secret` with `get_crypto_box_info` function and store it on your side.
    RandomSeedPhrase {
        dictionary: CryptoBoxDictionary,
        wordcount: u8,
    },

    /// Restores crypto box instance from an existing seed phrase.
    /// This type should be used when Crypto Box is initialized from a seed phrase, entered by a user.
    ///
    /// This type should be used only upon the first wallet initialization, all further
    /// initializations should use `EncryptedSecret` type instead.
    ///
    /// Get `encrypted_secret` with `get_crypto_box_info` function and store it on your side.
    PredefinedSeedPhrase {
        phrase: String,
    },

    /// Use this type for wallet reinitializations, when you already have `encrypted_secret` on hands.
    /// To get `encrypted_secret`, use `get_crypto_box_info` function after you initialized your
    /// crypto box for the first time.
    ///
    /// It is an object, containing seed phrase or private key, encrypted with
    /// `secret_encryption_salt` and `password_provider`.
    ///
    /// Note that if you want to change salt or password provider, then you need to reinitialize
    /// the wallet with `PredefinedSeedPhrase`, then get `EncryptedSecret` via `get_crypto_box_info`,
    /// store it somewhere, and only after that initialize the wallet with `EncryptedSecret` type.
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

/// Creates Crypto Box.
///
/// Crypto Box is a root crypto object, that encapsulates some secret (seed phrase usually)
/// in encrypted form and acts as a factory for all crypto primitives used in SDK:
/// keys for signing and encryption, derived from this secret.
///
/// Crypto Box encrypts original Seed Phrase with salt and some secret that is retrieved
/// in runtime via `password_provider` callback, implemented on Application side.
///
/// When used, decrypted secret has shown up in core library's memory for a very short period
/// of time and then is immediately overwritten with zeroes.
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
                SecretInternal::SeedPhrase { phrase: SecretString(mnemonics.generate_random_phrase()?) }
            };
            encrypt_secret(
                &phrase,
                &password_provider,
                &params.secret_encryption_salt,
            ).await?
        },

        CryptoBoxSecret::PredefinedSeedPhrase { phrase } => {
            encrypt_secret(
                &SecretInternal::SeedPhrase { phrase: SecretString(phrase.clone()) },
                &password_provider,
                &params.secret_encryption_salt,
            ).await?
        },

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

/// Remove Crypto Box.
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

/// Get Crypto Box Seed Phrase.
///
/// Store this data in your application for a very short time and overwrite it with zeroes ASAP.
#[api_function]
pub async fn get_crypto_box_seed_phrase(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<ResultOfGetCryptoBoxSeedPhrase> {
    let SecretInternal::SeedPhrase { phrase } = {
        let guard = get_crypto_box(&context, &params.handle)?;
        let crypto_box = guard.val();
        decrypt_secret(
            &crypto_box.encrypted_secret.0,
            &crypto_box.password_provider,
            &crypto_box.secret_encryption_salt.0,
        ).await?
    };

    Ok(ResultOfGetCryptoBoxSeedPhrase { phrase: phrase.0.clone() })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfGetCryptoBoxInfo {
    pub encrypted_secret: String,
}

/// Get Crypto Box Info.
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
