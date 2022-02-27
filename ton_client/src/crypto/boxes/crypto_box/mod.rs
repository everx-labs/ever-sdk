use std::future::Future;
use std::sync::Arc;

use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use lockfree::map::ReadGuard;
use tokio::sync::RwLock;
use zeroize::ZeroizeOnDrop;

use crate::ClientContext;
use crate::crypto::{CryptoConfig, Error, register_signing_box, RegisteredSigningBox, SigningBox};
use crate::crypto::boxes::crypto_box::encryption::{decrypt_secret, encrypt_secret};
use crate::crypto::boxes::signing_box::KeysSigningBox;
use crate::crypto::mnemonic::mnemonics;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;

mod encryption;

type PasswordProvider = Arc<dyn AppPasswordProvider + Send + Sync + 'static>;

const DEFAULT_DICTIONARY: u8 = 0;
const DEFAULT_WORDCOUNT: u8 = 12;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, ApiType, Default, PartialEq)]
pub struct CryptoBoxHandle(pub u32);

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
        dictionary: u8,
        wordcount: u8,
    },
}

impl Default for SecretInternal {
    fn default() -> Self {
        SecretInternal::SeedPhrase {
            phrase: SecretString(String::new()),
            dictionary: DEFAULT_DICTIONARY,
            wordcount: DEFAULT_WORDCOUNT,
        }
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
        dictionary: u8,
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
        dictionary: u8,
        wordcount: u8,
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
        Self::PredefinedSeedPhrase {
            phrase: Default::default(),
            dictionary: DEFAULT_DICTIONARY,
            wordcount: DEFAULT_WORDCOUNT,
        }
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
            let config = CryptoConfig::default();
            let phrase = {
                let mnemonics = mnemonics(&config, Some(*dictionary), Some(*wordcount))?;
                SecretInternal::SeedPhrase {
                    phrase: SecretString(mnemonics.generate_random_phrase()?),
                    dictionary: *dictionary,
                    wordcount: *wordcount,
                }
            };
            encrypt_secret(
                &phrase,
                &password_provider,
                &params.secret_encryption_salt,
            ).await?
        },

        CryptoBoxSecret::PredefinedSeedPhrase { phrase, dictionary, wordcount } => {
            encrypt_secret(
                &SecretInternal::SeedPhrase {
                    phrase: SecretString(phrase.clone()),
                    dictionary: *dictionary,
                    wordcount: *wordcount,
                },
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
    pub dictionary: u8,
    pub wordcount: u8,
}

/// Get Crypto Box Seed Phrase.
///
/// Store this data in your application for a very short time and overwrite it with zeroes ASAP.
#[api_function]
pub async fn get_crypto_box_seed_phrase(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<ResultOfGetCryptoBoxSeedPhrase> {
    let SecretInternal::SeedPhrase { phrase, dictionary, wordcount } = {
        let guard = get_crypto_box(&context, &params.handle)?;
        let crypto_box = guard.val();
        decrypt_secret(
            &crypto_box.encrypted_secret.0,
            &crypto_box.password_provider,
            &crypto_box.secret_encryption_salt.0,
        ).await?
    };

    Ok(ResultOfGetCryptoBoxSeedPhrase { phrase: phrase.0.clone(), dictionary, wordcount })
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfGetSigningBoxFromCryptoBox {
    /// Crypto Box Handle.
    pub handle: u32,
    /// HD key derivation path. By default, Everscale HD path is used.
    pub hdpath: Option<String>,
    /// Store derived secret for encryption algorithm for this lifetime (in ms).
    /// The timer starts after each signing box operation.
    /// Secrets will be deleted after each signing box operation, if this value is not set.
    pub secret_lifetime: Option<u32>,
}

/// Get Signing Box from Crypto Box.
#[api_function]
pub async fn get_signing_box_from_crypto_box(
    context: Arc<ClientContext>,
    params: ParamsOfGetSigningBoxFromCryptoBox,
) -> ClientResult<RegisteredSigningBox> {
    register_signing_box(
        context,
        SigningBoxFromCryptoBoxLifeCycleManager {
            params,
            internal_signing_box: Default::default(),
        }
    ).await
}

struct SigningBoxFromCryptoBoxLifeCycleManager {
    params: ParamsOfGetSigningBoxFromCryptoBox,
    internal_signing_box: Arc<RwLock<Option<Arc<KeysSigningBox>>>>,
}

impl SigningBoxFromCryptoBoxLifeCycleManager {
    async fn with_internal_signing_box<Cb, Fut, Ret>(
        &self,
        context: Arc<ClientContext>,
        callback: Cb,
    ) -> ClientResult<Ret>
    where
        Cb: Fn(Arc<KeysSigningBox>) -> Fut,
        Fut: Future<Output=ClientResult<Ret>>,
    {
        loop {
            if let Some(signing_box) = self.internal_signing_box.read().await.as_ref() {
                return callback(Arc::clone(signing_box)).await;
            }
            let mut write_guard = self.internal_signing_box.write().await;
            if let Some(signing_box) = write_guard.as_ref() {
                return callback(Arc::clone(signing_box)).await;
            }

            let seed_phrase = get_crypto_box_seed_phrase(
                Arc::clone(&context),
                RegisteredCryptoBox { handle: CryptoBoxHandle(self.params.handle) },
            ).await?;

            let mnemonic = mnemonics(
                &context.config.crypto,
                Some(seed_phrase.dictionary),
                Some(seed_phrase.wordcount),
            )?;
            let hdpath = self.params.hdpath.as_ref()
                .unwrap_or(&context.config.crypto.hdkey_derivation_path);
            let keypair = mnemonic.derive_ed25519_keys_from_phrase(&context.config.crypto, &seed_phrase.phrase, hdpath)
                .map::<ClientResult<Keypair>, _>(|keypair| Ok(Keypair {
                    public: PublicKey::from_bytes(&hex_decode(&keypair.public)?)
                        .map_err(|err| Error::invalid_public_key(err, &keypair.public))?,
                    secret: SecretKey::from_bytes(&hex_decode(&keypair.secret)?)
                        .map_err(|err| Error::invalid_secret_key(err, &keypair.secret))?,
                }))??;

            *write_guard = Some(Arc::new(KeysSigningBox::new(keypair)));

            let lifetime = self.params.secret_lifetime.unwrap_or(0) as u64;
            let context_copy = Arc::clone(&context);
            let internal_signing_box = Arc::clone(&self.internal_signing_box);
            context.env.spawn(async move {
                if lifetime > 0 {
                    context_copy.env.set_timer(lifetime).await.ok();
                }
                Self::drop_secret(internal_signing_box).await;
            });
        }
    }

    async fn drop_secret<T>(internal_signing_box: Arc<RwLock<Option<T>>>) {
        *internal_signing_box.write().await = None;
    }
}

#[async_trait::async_trait]
impl SigningBox for SigningBoxFromCryptoBoxLifeCycleManager {
    async fn get_public_key(&self, context: Arc<ClientContext>) -> ClientResult<Vec<u8>> {
        self.with_internal_signing_box(Arc::clone(&context), move |signing_box| {
            let context = Arc::clone(&context);
            async move {
                signing_box.get_public_key(Arc::clone(&context)).await
            }
        }).await
    }

    async fn sign(&self, context: Arc<ClientContext>, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        self.with_internal_signing_box(Arc::clone(&context), move |signing_box| {
            let context = Arc::clone(&context);
            async move {
                signing_box.sign(Arc::clone(&context), unsigned).await
            }
        }).await
    }
}

/// Remove all cached secrets from signing boxes, derived from selected crypto box.
#[api_function]
pub async fn clear_crypto_box_secret_cache(
    context: Arc<ClientContext>,
    params: RegisteredCryptoBox,
) -> ClientResult<()> {
    for item in context.boxes.signing_boxes.iter() {
        let signing_box_opt: Option<&SigningBoxFromCryptoBoxLifeCycleManager> =
            item.val().downcast_ref();
        if let Some(signing_box) = signing_box_opt {
            if signing_box.params.handle == params.handle.0 {
                SigningBoxFromCryptoBoxLifeCycleManager::drop_secret(
                    Arc::clone(&signing_box.internal_signing_box),
                ).await;
            }
        }
    }

    // TODO: Add support for ecnryption boxes created from crypto boxes.

    Ok(())
}
