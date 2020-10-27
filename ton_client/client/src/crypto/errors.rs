use crate::error::ClientError;
use std::fmt::Display;
const CRYPTO: isize = ClientError::CRYPTO; // 100

pub enum ErrorCode {
    InvalidPublicKey = CRYPTO + 0,
    InvalidSecretKey = CRYPTO + 1,
    InvalidKey = CRYPTO + 2,
    InvalidFactorizeChallenge = CRYPTO + 6,
    InvalidBigInt = CRYPTO + 7,
    ScryptFailed = CRYPTO + 8,
    InvalidKeySize = CRYPTO + 9,
    NaclSecretBoxFailed = CRYPTO + 10,
    NaclBoxFailed = CRYPTO + 11,
    NaclSignFailed = CRYPTO + 12,
    Bip39InvalidEntropy = CRYPTO + 13,
    Bip39InvalidPhrase = CRYPTO + 14,
    Bip32InvalidKey = CRYPTO + 15,
    Bip32InvalidDerivePath = CRYPTO + 16,
    Bip39InvalidDictionary = CRYPTO + 17,
    Bip39InvalidWordCount = CRYPTO + 18,
    MnemonicGenerationFailed = CRYPTO + 19,
    MnemonicFromEntropyFailed = CRYPTO + 20,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as isize, message)
}

impl Error {
    pub fn invalid_factorize_challenge<E: Display>(hex: &String, err: E) -> ClientError {
        error(
            ErrorCode::InvalidFactorizeChallenge,
            format!(
                "Invalid factorize challenge: {}\r\nchallenge: [{}]",
                err, hex
            ),
        )
    }

    pub fn invalid_big_int(hex: &String) -> ClientError {
        error(ErrorCode::InvalidBigInt, format!("Invalid big int [{}]", hex))
    }

    pub fn scrypt_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::ScryptFailed, format!(r#"Scrypt failed: {}"#, err))
    }

    pub fn invalid_key_size(actual: usize, expected: usize) -> ClientError {
        error(
            ErrorCode::InvalidKeySize,
            format!("Invalid key size {}. Expected {}.", actual, expected),
        )
    }

    pub fn nacl_secret_box_failed<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::NaclSecretBoxFailed,
            format!("Nacl Secret Box failed: {}", err),
        )
    }

    pub fn nacl_box_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::NaclBoxFailed, format!("Box failed: {}", err))
    }

    pub fn nacl_sign_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::NaclSignFailed, format!("Sign failed: {}", err))
    }

    pub fn bip39_invalid_entropy<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::Bip39InvalidEntropy,
            format!("Invalid bip39 entropy: {}", err),
        )
    }

    pub fn bip39_invalid_phrase<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::Bip39InvalidPhrase,
            format!("Invalid bip39 phrase: {}", err),
        )
    }

    pub fn bip32_invalid_key<E: Display>(key: E) -> ClientError {
        error(ErrorCode::Bip32InvalidKey, format!("Invalid bip32 key: {}", key))
    }

    pub fn bip32_invalid_derive_path<E: Display>(path: E) -> ClientError {
        error(
            ErrorCode::Bip32InvalidDerivePath,
            format!("Invalid bip32 derive path: {}", path),
        )
    }

    pub fn bip39_invalid_dictionary(dictionary: u8) -> ClientError {
        error(
            ErrorCode::Bip39InvalidDictionary,
            format!("Invalid mnemonic dictionary: {}", dictionary),
        )
    }

    pub fn bip39_invalid_word_count(word_count: u8) -> ClientError {
        error(
            ErrorCode::Bip39InvalidWordCount,
            format!("Invalid mnemonic word count: {}", word_count),
        )
    }

    pub fn invalid_secret_key<E: Display>(err: E, key: &String) -> ClientError {
        error(
            ErrorCode::InvalidSecretKey,
            format!("Invalid secret key [{}]: {}", err, key),
        )
    }

    pub fn invalid_public_key<E: Display>(err: E, key: &String) -> ClientError {
        error(
            ErrorCode::InvalidPublicKey,
            format!("Invalid public key [{}]: {}", err, key),
        )
    }

    pub fn invalid_key<E: Display>(err: E, key: &String) -> ClientError {
        error(ErrorCode::InvalidKey, format!("Invalid key [{}]: {}", err, key))
    }

    pub fn mnemonic_generation_failed() -> ClientError {
        error(
            ErrorCode::MnemonicGenerationFailed,
            "Mnemonic generation failed".into(),
        )
    }

    pub fn mnemonic_from_entropy_failed(reason: &str) -> ClientError {
        error(
            ErrorCode::MnemonicFromEntropyFailed,
            format!("Mnemonic from entropy failed: {}", reason),
        )
    }
}
