use chacha20::cipher::{NewStreamCipher, SyncStreamCipher};
use rand::RngCore;
use sodalite::{BOX_NONCE_LEN, BOX_PUBLIC_KEY_LEN, BOX_SECRET_KEY_LEN};
use zeroize::Zeroize;

use crate::crypto::nacl::nacl_box_open_internal;
use crate::crypto::{boxes::crypto_box::SecretInternal, internal::SecretBuf};
use crate::error::ClientResult;

use super::{Error, PasswordProvider};

const NONCE_LEN: usize = 12;

#[derive(Default, Zeroize, ZeroizeOnDrop)]
struct SecretKey(sodalite::BoxSecretKey);

fn generate_nonce() -> SecretBuf {
    let mut rng = rand::thread_rng();
    let mut nonce = SecretBuf(vec![0; NONCE_LEN]);
    rng.fill_bytes(&mut nonce.0);

    nonce
}

fn derive_key(password: &[u8], salt: &str) -> ClientResult<SecretBuf> {
    let scrypt_params = scrypt::Params::new(14, 8, 1).expect("Scrypt params setup failed");
    let mut key = SecretBuf(vec![0; 32]);
    scrypt::scrypt(password, salt.as_bytes(), &scrypt_params, &mut key.0)
        .map_err(|err| Error::scrypt_failed(err))?;

    Ok(key)
}

async fn apply_chacha20(
    secret: &[u8],
    password_provider: &PasswordProvider,
    salt: &str,
    nonce: &[u8],
) -> ClientResult<SecretBuf> {
    let password = get_password(password_provider).await?;
    let key = derive_key(&password.0, salt)?;
    let mut cipher = chacha20::ChaCha20::new(
        chacha20::Key::from_slice(&key.0),
        chacha20::Nonce::from_slice(nonce),
    );
    let mut output = SecretBuf(secret.into());
    cipher.apply_keystream(&mut output.0);

    Ok(output)
}

pub(crate) async fn encrypt_secret(
    secret: &SecretInternal,
    password_provider: &PasswordProvider,
    salt: &str,
) -> ClientResult<SecretBuf> {
    let mut result = generate_nonce();
    let serialized = SecretBuf(
        bincode::serialize(secret)
            .map_err(|err| Error::crypto_box_secret_serialization_error(err))?,
    );
    apply_chacha20(&serialized.0, password_provider, salt, &result.0)
        .await
        .map(|mut output| {
            result.0.append(&mut output.0);
            result
        })
}

pub(crate) async fn decrypt_secret(
    encrypted_secret: &[u8],
    password_provider: &PasswordProvider,
    salt: &str,
) -> ClientResult<SecretInternal> {
    let (nonce, encrypted_secret) = encrypted_secret.split_at(NONCE_LEN);
    let data = apply_chacha20(encrypted_secret, password_provider, salt, nonce).await?;
    bincode::deserialize(&data.0).map_err(|err| Error::crypto_box_secret_deserialization_error(err))
}

async fn get_password(password_provider: &PasswordProvider) -> ClientResult<SecretBuf> {
    let (secret_key, public_key) = gen_nacl_box_keypair();

    let password_data = password_provider.get_password(&public_key).await?;

    Ok(SecretBuf(nacl_box_open_internal(
        &password_data.encrypted_password,
        &public_key[..BOX_NONCE_LEN],
        &password_data.app_encryption_pubkey,
        &secret_key.0,
    )?))
}

fn gen_nacl_box_keypair() -> (SecretKey, sodalite::BoxPublicKey) {
    let mut secret_key = SecretKey([0; BOX_SECRET_KEY_LEN]);
    let mut public_key = [0; BOX_PUBLIC_KEY_LEN];
    sodalite::box_keypair(&mut public_key, &mut secret_key.0);

    (secret_key, public_key)
}
