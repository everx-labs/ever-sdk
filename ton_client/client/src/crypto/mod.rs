pub(crate) mod math;
pub(crate) mod random;
pub(crate) mod sha;
pub(crate) mod scrypt;
pub(crate) mod nacl;
pub(crate) mod keys;
pub(crate) mod ed25519;
pub(crate) mod mnemonic;
pub(crate) mod hdkey;

use crypto as api;
use types::{base64_decode, ApiError, ApiResult, hex_decode};
use crypto::keys::{KeyPair, hmac_sha512, pbkdf2_hmac_sha512, key_to_ton_string};
use crypto::keys::KeyStore;
use dispatch::DispatchTable;
use client::ClientContext;
use crypto::math::ton_crc16;
use crypto::mnemonic::{CryptoMnemonic, TonMnemonic, Bip39Mnemonic};
use bip39::{MnemonicType, Language};

#[derive(Serialize, Deserialize)]
pub(crate) struct FactorizeResult {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize)]
pub(crate) struct ModularPowerParams {
    pub base: String,
    pub exponent: String,
    pub modulus: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct InputMessage {
    pub text: Option<String>,
    pub hex: Option<String>,
    pub base64: Option<String>,
}

#[derive(Deserialize)]
pub(crate) enum OutputEncoding {
    Text,
    Hex,
    HexUppercase,
    Base64,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct GenerateParams {
    pub length: usize,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ShaParams {
    pub message: InputMessage,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ScryptParams {
    pub password: InputMessage,
    pub salt: InputMessage,
    pub logN: u8,
    pub r: u32,
    pub p: u32,
    pub dkLen: usize,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct NaclBoxParams {
    pub message: InputMessage,
    pub nonce: String,
    pub theirPublicKey: String,
    pub secretKey: Option<String>,
    pub keystoreHandle: Option<String>,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct NaclSecretBoxParams {
    pub message: InputMessage,
    pub nonce: String,
    pub key: Option<String>,
    pub keystoreHandle: Option<String>,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct NaclSignParams {
    pub message: InputMessage,
    pub key: Option<String>,
    pub keystoreHandle: Option<String>,
    #[serde(default = "default_result_encoding_hex")]
    pub outputEncoding: OutputEncoding,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct MnemonicWordsParams {
    pub dictionary: Option<u8>,
    pub wordCount: Option<u8>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct MnemonicGenerateParams {
    pub dictionary: Option<u8>,
    pub wordCount: Option<u8>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct MnemonicFromEntropyParams {
    pub dictionary: Option<u8>,
    pub wordCount: Option<u8>,
    pub entropy: InputMessage,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct MnemonicVerifyParams {
    pub dictionary: Option<u8>,
    pub wordCount: Option<u8>,
    pub phrase: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyFromMnemonicParams {
    pub dictionary: Option<u8>,
    pub wordCount: Option<u8>,
    pub phrase: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyDeriveParams {
    serialized: String,
    index: u32,
    #[serde(default = "default_hardened")]
    hardened: bool,
    #[serde(default = "default_compliant")]
    compliant: bool,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyDerivePathParams {
    serialized: String,
    path: String,
    #[serde(default = "default_compliant")]
    compliant: bool,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyGetKeyParams {
    serialized: String,
}

fn default_hardened() -> bool {
    false
}

fn default_compliant() -> bool {
    true
}

fn default_result_encoding_hex() -> OutputEncoding {
    OutputEncoding::Hex
}

impl InputMessage {
    pub(crate) fn decode(&self) -> ApiResult<Vec<u8>> {
        if let Some(ref text) = self.text {
            Ok(text.as_bytes().to_vec())
        } else if let Some(ref hex) = self.hex {
            hex_decode(hex)
        } else if let Some(ref base64) = self.base64 {
            base64_decode(base64)
        } else {
            Err(ApiError::crypto_convert_input_data_missing())
        }
    }
}

impl OutputEncoding {
    pub(crate) fn encode(&self, output: Vec<u8>) -> ApiResult<String> {
        match self {
            OutputEncoding::Text => Ok(String::from_utf8(output)
                .map_err(|err| ApiError::crypto_convert_output_can_not_be_encoded_to_utf8(err))?),
            OutputEncoding::Hex => Ok(hex::encode(output)),
            OutputEncoding::HexUppercase => Ok(hex::encode_upper(output)),
            OutputEncoding::Base64 => Ok(base64::encode(&output))
        }
    }
}

const TON_DICTIONARY: u8 = 0;
const ENGLISH_DICTIONARY: u8 = 1;
const CHINESE_SIMPLIFIED_DICTIONARY: u8 = 2;
const CHINESE_TRADITIONAL_DICTIONARY: u8 = 3;
const FRENCH_DICTIONARY: u8 = 4;
const ITALIAN_DICTIONARY: u8 = 5;
const JAPANESE_DICTIONARY: u8 = 6;
const KOREAN_DICTIONARY: u8 = 7;
const SPANISH_DICTIONARY: u8 = 8;

fn mnemonics(dictionary: Option<u8>, word_count: Option<u8>) -> ApiResult<Box<dyn CryptoMnemonic>> {
    let dictionary = dictionary.unwrap_or(TON_DICTIONARY);
    let word_count = word_count.unwrap_or(24);
    if dictionary == TON_DICTIONARY {
        return Ok(Box::new(TonMnemonic::new(word_count)));
    }
    let mnemonic_type = match word_count {
        12 => MnemonicType::Words12,
        15 => MnemonicType::Words15,
        18 => MnemonicType::Words18,
        21 => MnemonicType::Words21,
        24 => MnemonicType::Words24,
        _ => return Err(ApiError::crypto_bip39_invalid_word_count(word_count)),
    };
    let language = match dictionary {
        ENGLISH_DICTIONARY => Language::English,
        CHINESE_SIMPLIFIED_DICTIONARY => Language::ChineseSimplified,
        CHINESE_TRADITIONAL_DICTIONARY => Language::ChineseTraditional,
        FRENCH_DICTIONARY => Language::French,
        ITALIAN_DICTIONARY => Language::Italian,
        JAPANESE_DICTIONARY => Language::Japanese,
        KOREAN_DICTIONARY => Language::Korean,
        SPANISH_DICTIONARY => Language::Spanish,
        _ => return Err(ApiError::crypto_bip39_invalid_dictionary(dictionary))
    };
    Ok(Box::new(Bip39Mnemonic::new(mnemonic_type, language)))
}

pub(crate) fn register(handlers: &mut DispatchTable) {

    // Math

    handlers.spawn("crypto.math.factorize", |_context: &mut ClientContext, hex: String| {
        let challenge = u64::from_str_radix(hex.as_str(), 16).
            map_err(|err| ApiError::crypto_invalid_factorize_challenge(&hex, err))?;
        if challenge == 0 {
            return Err(ApiError::crypto_invalid_factorize_challenge(&hex, "Challenge can not be zero"));
        }
        let answer = api::math::factorize(challenge);
        if answer.len() != 2 {
            return Err(ApiError::crypto_invalid_factorize_challenge(&hex, "Challenge can not be factorized"));
        }
        Ok(FactorizeResult {
            a: format!("{:X}", answer[0]),
            b: format!("{:X}", answer[1]),
        })
    });
    handlers.spawn("crypto.math.modularPower", |_context: &mut ClientContext, params: ModularPowerParams| {
        api::math::modular_power(&params.base, &params.exponent, &params.modulus)
    });

    handlers.spawn("crypto.ton_crc16", |_context: &mut ClientContext, params: InputMessage| {
        let bytes = params.decode()?;
        Ok(ton_crc16(&bytes))
    });

    // Random

    handlers.call("crypto.random.generateBytes", |_context: &mut ClientContext, params: GenerateParams| {
        params.outputEncoding.encode(api::random::generate_bytes(params.length))
    });

    // Keys

    handlers.spawn("crypto.sign_keys_from_ton_mnemonic", |_context: &mut ClientContext, params: String| {
        let entropy = hmac_sha512(params.as_bytes(), &[]);
        let seed = pbkdf2_hmac_sha512(&entropy, "TON default seed".as_bytes(), 100_000);
        let secret = ed25519_dalek::SecretKey::from_bytes(&seed[..32]).unwrap();
        let public = ed25519_dalek::PublicKey::from_secret::<sha2::Sha512>(&secret);
        Ok(KeyPair::new(
            hex::encode(public.as_bytes()),
            hex::encode(secret.as_bytes()),
        ))
    });

    handlers.spawn("crypto.ton_public_key_string", |_context: &mut ClientContext, params: String| {
        Ok(key_to_ton_string(&hex_decode(&params)?))
    });

    handlers.call_no_args("crypto.ed25519.keypair", |_context: &mut ClientContext|
        api::ed25519::generate_keypair());
    handlers.call("crypto.keystore.add", |_context: &mut ClientContext, keys: KeyPair| {
        Ok(KeyStore::add(&keys))
    });
    handlers.call("crypto.keystore.remove", |_context: &mut ClientContext, handle: String| {
        KeyStore::remove(&handle);
        Ok(())
    });
    handlers.call_no_args("crypto.keystore.clear", |_context: &mut ClientContext| {
        KeyStore::clear();
        Ok(())
    });

    // Sha

    handlers.spawn("crypto.sha256", |_context: &mut ClientContext, params: ShaParams| {
        params.outputEncoding.encode(api::sha::sha256(&params.message.decode()?))
    });

    handlers.spawn("crypto.sha512", |_context: &mut ClientContext, params: ShaParams| {
        params.outputEncoding.encode(api::sha::sha512(&params.message.decode()?))
    });

    // Scrypt

    handlers.spawn("crypto.scrypt", |_context: &mut ClientContext, params: ScryptParams| {
        params.outputEncoding.encode(api::scrypt::scrypt(
            &params.password.decode()?,
            &params.salt.decode()?,
            params.logN,
            params.r,
            params.p,
            params.dkLen,
        )?)
    });

    // NaCl

    handlers.call_no_args("crypto.nacl.box.keypair", |_context: &mut ClientContext|
        api::nacl::box_keypair(),
    );
    handlers.call("crypto.nacl.box.keypair.fromSecretKey", |_context: &mut ClientContext, secret: String| {
        api::nacl::box_keypair_from_secret_key(&secret)
    });
    handlers.call_no_args("crypto.nacl.sign.keypair", |_context: &mut ClientContext|
        api::nacl::sign_keypair(),
    );
    handlers.call("crypto.nacl.sign.keypair.fromSecretKey", |_context: &mut ClientContext, secret: String| {
        api::nacl::sign_keypair_from_secret_key(&secret)
    });
    handlers.spawn("crypto.nacl.box", |_context: &mut ClientContext, params: NaclBoxParams| {
        params.outputEncoding.encode(api::nacl::box_(
            params.message.decode()?,
            hex_decode(&params.nonce)?,
            hex_decode(&params.theirPublicKey)?,
            KeyStore::decode_secret(&params.secretKey, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.box.open", |_context: &mut ClientContext, params: NaclBoxParams| {
        params.outputEncoding.encode(api::nacl::box_open(
            params.message.decode()?,
            hex_decode(&params.nonce)?,
            hex_decode(&params.theirPublicKey)?,
            KeyStore::decode_secret(&params.secretKey, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.secret.box", |_context: &mut ClientContext, params: NaclSecretBoxParams| {
        params.outputEncoding.encode(api::nacl::secret_box(
            params.message.decode()?,
            hex_decode(&params.nonce)?,
            KeyStore::decode_secret(&params.key, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.secret.box.open", |_context: &mut ClientContext, params: NaclSecretBoxParams| {
        params.outputEncoding.encode(api::nacl::secret_box_open(
            params.message.decode()?,
            hex_decode(&params.nonce)?,
            KeyStore::decode_secret(&params.key, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.sign", |_context: &mut ClientContext, params: NaclSignParams| {
        params.outputEncoding.encode(api::nacl::sign(
            params.message.decode()?,
            KeyStore::decode_secret(&params.key, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.sign.open", |_context: &mut ClientContext, params: NaclSignParams| {
        params.outputEncoding.encode(api::nacl::sign_open(
            params.message.decode()?,
            KeyStore::decode_secret(&params.key, &params.keystoreHandle)?,
        )?)
    });
    handlers.spawn("crypto.nacl.sign.detached", |_context: &mut ClientContext, params: NaclSignParams| {
        params.outputEncoding.encode(api::nacl::sign_detached(
            params.message.decode()?,
            KeyStore::decode_secret(&params.key, &params.keystoreHandle)?,
        )?)
    });

    // Mnemonic

    handlers.spawn("crypto.mnemonic.words", |_context: &mut ClientContext, params: MnemonicWordsParams|
        mnemonics(params.dictionary, params.wordCount)?.get_words(),
    );

    handlers.spawn("crypto.mnemonic.from.random", |_context: &mut ClientContext, params: MnemonicGenerateParams|
        mnemonics(params.dictionary, params.wordCount)?.generate_random_phrase()
    );

    handlers.spawn("crypto.mnemonic.from.entropy", |_context: &mut ClientContext, params: MnemonicFromEntropyParams| {
        mnemonics(params.dictionary, params.wordCount)?.phrase_from_entropy(&params.entropy.decode()?)
    });

    handlers.spawn("crypto.mnemonic.verify", |_context: &mut ClientContext, params: MnemonicVerifyParams| {
        mnemonics(params.dictionary, params.wordCount)?.is_phrase_valid(&params.phrase)
    });

    // HDKey

    handlers.spawn("crypto.hdkey.xprv.from.mnemonic", |_context: &mut ClientContext, params: HDKeyFromMnemonicParams| {
        api::hdkey::hdkey_xprv_from_mnemonic(&params.phrase)
    });

    handlers.spawn("crypto.hdkey.xprv.derive", |_context: &mut ClientContext, params: HDKeyDeriveParams| {
        api::hdkey::hdkey_derive_from_xprv(
            &params.serialized,
            params.index,
            params.hardened,
            params.compliant)
    });

    handlers.spawn("crypto.hdkey.xprv.derive.path", |_context: &mut ClientContext, params: HDKeyDerivePathParams| {
        api::hdkey::hdkey_derive_from_xprv_path(
            &params.serialized,
            &params.path,
            params.compliant)
    });

    handlers.spawn("crypto.hdkey.xprv.secret", |_context: &mut ClientContext, params: HDKeyGetKeyParams| {
        api::hdkey::hdkey_secret_from_xprv(&params.serialized)
    });

    handlers.spawn("crypto.hdkey.xprv.public", |_context: &mut ClientContext, params: HDKeyGetKeyParams| {
        api::hdkey::hdkey_public_from_xprv(&params.serialized)
    });
}

