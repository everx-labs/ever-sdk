use crypto::keys::KeyPair;
use types::ApiResult;

pub fn generate_keypair() -> ApiResult<KeyPair> {
    let mut csprng = rand::rngs::OsRng::new().unwrap();
    let keypair = ed25519_dalek::Keypair::generate::<sha2::Sha512, _>(&mut csprng);
    Ok(KeyPair::new(
        hex::encode(keypair.public.to_bytes()),
        hex::encode(keypair.secret.to_bytes()),
    ))
}
