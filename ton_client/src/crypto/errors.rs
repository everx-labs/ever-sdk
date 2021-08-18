use crate::error::ClientError;
use std::fmt::Display;

use super::CipherMode;

#[derive(ApiType)]
pub enum ErrorCode {
    InvalidPublicKey = 100,
    InvalidSecretKey = 101,
    InvalidKey = 102,
    InvalidFactorizeChallenge = 106,
    InvalidBigInt = 107,
    ScryptFailed = 108,
    InvalidKeySize = 109,
    NaclSecretBoxFailed = 110,
    NaclBoxFailed = 111,
    NaclSignFailed = 112,
    Bip39InvalidEntropy = 113,
    Bip39InvalidPhrase = 114,
    Bip32InvalidKey = 115,
    Bip32InvalidDerivePath = 116,
    Bip39InvalidDictionary = 117,
    Bip39InvalidWordCount = 118,
    MnemonicGenerationFailed = 119,
    MnemonicFromEntropyFailed = 120,
    SigningBoxNotRegistered = 121,
    InvalidSignature = 122,
    EncryptionBoxNotRegistered = 123,
    InvalidIvSize = 124,
    UnsupportedCipherMode = 125,
    CannotCreateCipher = 126,
    EncryptDataError = 127,
    DecryptDataError = 128,
    IvRequired = 129,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
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
        error(
            ErrorCode::InvalidBigInt,
            format!("Invalid big int [{}]", hex),
        )
    }

    pub fn scrypt_failed<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::ScryptFailed,
            format!(r#"Scrypt failed: {}"#, err),
        )
    }

    pub fn invalid_key_size(actual: usize, expected: &[usize]) -> ClientError {
        error(
            ErrorCode::InvalidKeySize,
            format!(
                "Invalid key size {}. Expected {}.",
                actual,
                expected.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(" or ")
            ),
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
        error(
            ErrorCode::Bip32InvalidKey,
            format!("Invalid bip32 key: {}", key),
        )
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
            format!("Invalid secret key [{}]: {}", key, err),
        )
    }

    pub fn invalid_public_key<E: Display>(err: E, key: &String) -> ClientError {
        error(
            ErrorCode::InvalidPublicKey,
            format!("Invalid public key [{}]: {}", key, err),
        )
    }

    pub fn invalid_signature<E: Display>(err: E, signature: &String) -> ClientError {
        error(
            ErrorCode::InvalidSignature,
            format!("Invalid signature [{}]: {}", signature, err),
        )
    }

    pub fn invalid_key<E: Display>(err: E, key: &String) -> ClientError {
        error(
            ErrorCode::InvalidKey,
            format!("Invalid key [{}]: {}", key, err),
        )
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

    pub fn signing_box_not_registered(id: u32) -> ClientError {
        error(
            ErrorCode::SigningBoxNotRegistered,
            format!("Signing box is not registered. ID {}", id),
        )
    }

    pub fn encryption_box_not_registered(id: u32) -> ClientError {
        error(
            ErrorCode::EncryptionBoxNotRegistered,
            format!("Encryption box is not registered. ID {}", id),
        )
    }

    pub fn invalid_iv_size(actual: usize, expected: usize) -> ClientError {
        error(
            ErrorCode::InvalidIvSize,
            format!("Invalid IV size {}. Expected {}.", actual, expected),
        )
    }

    pub fn unsupported_cipher_mode(mode: &str) -> ClientError {
        error(
            ErrorCode::UnsupportedCipherMode,
            format!("Unsupported cipher mode: {}", mode),
        )
    }

    pub fn cannot_create_cipher(err: impl Display) -> ClientError {
        error(
            ErrorCode::CannotCreateCipher,
            format!("Can not create cipher: {}", err),
        )
    }

    pub fn encrypt_data_error(err: impl Display) -> ClientError {
        error(
            ErrorCode::EncryptDataError,
            format!("Can not encrypt data: {}", err),
        )
    }

    pub fn decrypt_data_error(err: impl Display) -> ClientError {
        error(
            ErrorCode::DecryptDataError,
            format!("Can not decrypt data: {}", err),
        )
    }

    pub fn iv_required(mode: &CipherMode) -> ClientError {
        error(
            ErrorCode::DecryptDataError,
            format!("initialization vector is required for {:?} cipher mode", mode),
        )
    }
}
