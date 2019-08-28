pub(crate) mod math;
pub(crate)mod random;
pub(crate)mod sha;
pub(crate)mod scrypt;
pub(crate)mod nacl;
pub(crate)mod keys;
pub(crate)mod ed25519;
pub(crate)mod mnemonic;
pub(crate)mod hdkey;

use crypto as api;
use types::{base64_decode, ApiError, ApiResult, hex_decode};
use crypto::keys::KeyPair;
use crypto::keys::KeyStore;
use dispatch::DispatchTable;
use InteropContext;
use client::Context;

pub(crate) struct CryptoApi;

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
pub(crate) struct MnemonicFromEntropyParams {
    pub entropy: InputMessage,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct MnemonicVerifyParams {
    pub phrase: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyFromMnemonicParams {
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
    compliant: bool
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct HDKeyDerivePathParams {
    serialized: String,
    path: String,
    #[serde(default = "default_compliant")]
    compliant: bool
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

impl CryptoApi {
    pub(crate) fn register(handlers: &mut DispatchTable) {

        // Math

        handlers.spawn("crypto.math.factorize", |context: &mut Context, hex: String| {
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
        handlers.spawn("crypto.math.modularPower", |context: &mut Context, params: ModularPowerParams| {
            api::math::modular_power(&params.base, &params.exponent, &params.modulus)
        });

        // Random

        handlers.call("crypto.random.generateBytes", |context: &mut Context, params: GenerateParams| {
            params.outputEncoding.encode(api::random::generate_bytes(params.length))
        });

        // Keys

        handlers.call_no_args("crypto.ed25519.keypair", |context: &mut Context|
            api::ed25519::generate_keypair());
        handlers.call("crypto.keystore.add", |context: &mut Context, keys: KeyPair| {
            Ok(KeyStore::add(&keys))
        });
        handlers.call("crypto.keystore.remove", |context: &mut Context, handle: String| {
            KeyStore::remove(&handle);
            Ok(())
        });
        handlers.call_no_args("crypto.keystore.clear", |context: &mut Context| {
            KeyStore::clear();
            Ok(())
        });

        // Sha

        handlers.spawn("crypto.sha256", |context: &mut Context, params: ShaParams| {
            params.outputEncoding.encode(api::sha::sha256(&params.message.decode()?))
        });

        handlers.spawn("crypto.sha512", |context: &mut Context, params: ShaParams| {
            params.outputEncoding.encode(api::sha::sha512(&params.message.decode()?))
        });

        // Scrypt

        handlers.spawn("crypto.scrypt", |context: &mut Context, params: ScryptParams| {
            params.outputEncoding.encode(api::scrypt::scrypt(
                &params.password.decode()?,
                &params.salt.decode()?,
                params.logN,
                params.r,
                params.p,
                params.dkLen
            )?)
        });

        // NaCl

        handlers.call_no_args("crypto.nacl.box.keypair", |context: &mut Context|
            api::nacl::box_keypair()
        );
        handlers.call("crypto.nacl.box.keypair.fromSecretKey", |context: &mut Context, secret: String| {
            api::nacl::box_keypair_from_secret_key(&secret)
        });
        handlers.call_no_args("crypto.nacl.sign.keypair", |context: &mut Context|
            api::nacl::sign_keypair()
        );
        handlers.call("crypto.nacl.sign.keypair.fromSecretKey", |context: &mut Context, secret: String| {
            api::nacl::sign_keypair_from_secret_key(&secret)
        });
        handlers.spawn("crypto.nacl.box", |context: &mut Context, params: NaclBoxParams| {
            params.outputEncoding.encode(api::nacl::box_(
                params.message.decode()?,
                hex_decode(&params.nonce)?,
                hex_decode(&params.theirPublicKey)?,
                KeyStore::decode_secret(&params.secretKey, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.box.open", |context: &mut Context, params: NaclBoxParams| {
            params.outputEncoding.encode(api::nacl::box_open(
                params.message.decode()?,
                hex_decode(&params.nonce)?,
                hex_decode(&params.theirPublicKey)?,
                KeyStore::decode_secret(&params.secretKey, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.secret.box", |context: &mut Context, params: NaclSecretBoxParams| {
            params.outputEncoding.encode(api::nacl::secret_box(
                params.message.decode()?,
                hex_decode(&params.nonce)?,
                KeyStore::decode_secret(&params.key, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.secret.box.open", |context: &mut Context, params: NaclSecretBoxParams| {
            params.outputEncoding.encode(api::nacl::secret_box_open(
                params.message.decode()?,
                hex_decode(&params.nonce)?,
                KeyStore::decode_secret(&params.key, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.sign", |context: &mut Context, params: NaclSignParams| {
            params.outputEncoding.encode(api::nacl::sign(
                params.message.decode()?,
                KeyStore::decode_secret(&params.key, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.sign.open", |context: &mut Context, params: NaclSignParams| {
            params.outputEncoding.encode(api::nacl::sign_open(
                params.message.decode()?,
                KeyStore::decode_secret(&params.key, &params.keystoreHandle)?
            )?)
        });
        handlers.spawn("crypto.nacl.sign.detached", |context: &mut Context, params: NaclSignParams| {
            params.outputEncoding.encode(api::nacl::sign_detached(
                params.message.decode()?,
                KeyStore::decode_secret(&params.key, &params.keystoreHandle)?
            )?)
        });

        // Mnemonic

        handlers.spawn_no_args("crypto.mnemonic.words",|context: &mut Context|
            api::mnemonic::mnemonic_get_words()
        );

        handlers.spawn_no_args("crypto.mnemonic.from.random",|context: &mut Context|
            api::mnemonic::mnemonic_generate_random()
        );

        handlers.spawn("crypto.mnemonic.from.entropy", |context: &mut Context, params: MnemonicFromEntropyParams| {
            api::mnemonic::mnemonic_from_entropy(&params.entropy.decode()?)
        });

        handlers.spawn("crypto.mnemonic.verify", |context: &mut Context, params: MnemonicVerifyParams| {
            api::mnemonic::mnemonic_is_valid(&params.phrase)
        });

        handlers.spawn("crypto.mnemonic.verify", |context: &mut Context, params: MnemonicVerifyParams| {
            api::mnemonic::mnemonic_is_valid(&params.phrase)
        });

        // HDKey

        handlers.spawn("crypto.hdkey.xprv.from.mnemonic", |context: &mut Context, params: HDKeyFromMnemonicParams| {
            api::hdkey::hdkey_xprv_from_mnemonic(&params.phrase)
        });

        handlers.spawn("crypto.hdkey.xprv.derive", |context: &mut Context, params: HDKeyDeriveParams| {
            api::hdkey::hdkey_derive_from_xprv(
                &params.serialized,
                params.index,
                params.hardened,
                params.compliant)
        });

        handlers.spawn("crypto.hdkey.xprv.derive.path", |context: &mut Context, params: HDKeyDerivePathParams| {
            api::hdkey::hdkey_derive_from_xprv_path(
                &params.serialized,
                &params.path,
                params.compliant)
        });

        handlers.spawn("crypto.hdkey.xprv.secret", |context: &mut Context, params: HDKeyGetKeyParams| {
            api::hdkey::hdkey_secret_from_xprv(&params.serialized)
        });

        handlers.spawn("crypto.hdkey.xprv.public", |context: &mut Context, params: HDKeyGetKeyParams| {
            api::hdkey::hdkey_public_from_xprv(&params.serialized)
        });
    }
}
