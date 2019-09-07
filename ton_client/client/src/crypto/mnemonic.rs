use bip39::{Mnemonic, MnemonicType::Words12, Language::English};
use types::{ApiResult, ApiError};
use hmac::Hmac;
use sha2::Sha512;
use pbkdf2::pbkdf2;

pub fn mnemonic_get_words() -> ApiResult<String> {
    let words = English.wordlist();
    let mut joined = String::new();
    for i in 0..2048 {
        if !joined.is_empty() {
            joined.push(' ');
        }
        joined += words.get_word(i.into());
    }
    Ok(joined)
}

pub fn mnemonic_generate_random() -> ApiResult<String> {
    let mnemonic = Mnemonic::new(Words12, English);
    Ok(mnemonic.phrase().into())
}

pub fn mnemonic_from_entropy(entropy: &[u8]) -> ApiResult<String> {
    let mnemonic = Mnemonic::from_entropy(&entropy, English)
        .map_err(|err| ApiError::crypto_bip39_invalid_entropy(err))?;
    Ok(mnemonic.phrase().into())
}

pub fn mnemonic_is_valid(phrase: &String) -> ApiResult<bool> {
    Ok(Mnemonic::validate(phrase.as_str(), English).is_ok())
}

pub fn mnemonic_seed_from_phrase_and_salt(phrase: &String, salt: &String) -> ApiResult<String> {
    let mnemonic = Mnemonic::from_phrase(phrase, English)
        .map_err(|err| ApiError::crypto_bip39_invalid_phrase(err))?;

    let salt = format!("mnemonic{}", salt);
    let mut seed = vec![0u8; 64];
    pbkdf2::<Hmac<Sha512>>(mnemonic.phrase().as_bytes(), salt.as_bytes(), 2048, &mut seed);
    Ok(hex::encode(seed))
}

pub fn mnemonic_entropy_from_phrase(phrase: &String) -> ApiResult<String> {
    let mnemonic = Mnemonic::from_phrase(phrase, English)
        .map_err(|err| ApiError::crypto_bip39_invalid_phrase(err))?;
    Ok(hex::encode(mnemonic.entropy()))
}

