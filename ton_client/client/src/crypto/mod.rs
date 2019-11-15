/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

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
use crypto::keys::KeyPair;
use crypto::keys::KeyStore;
use dispatch::DispatchTable;
use client::ClientContext;

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

    // Random

    handlers.call("crypto.random.generateBytes", |_context: &mut ClientContext, params: GenerateParams| {
        params.outputEncoding.encode(api::random::generate_bytes(params.length))
    });

    // Keys

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

    handlers.spawn_no_args("crypto.mnemonic.words", |_context: &mut ClientContext|
        api::mnemonic::mnemonic_get_words(),
    );

    handlers.spawn_no_args("crypto.mnemonic.from.random", |_context: &mut ClientContext|
        api::mnemonic::mnemonic_generate_random(),
    );

    handlers.spawn("crypto.mnemonic.from.entropy", |_context: &mut ClientContext, params: MnemonicFromEntropyParams| {
        api::mnemonic::mnemonic_from_entropy(&params.entropy.decode()?)
    });

    handlers.spawn("crypto.mnemonic.verify", |_context: &mut ClientContext, params: MnemonicVerifyParams| {
        api::mnemonic::mnemonic_is_valid(&params.phrase)
    });

    handlers.spawn("crypto.mnemonic.verify", |_context: &mut ClientContext, params: MnemonicVerifyParams| {
        api::mnemonic::mnemonic_is_valid(&params.phrase)
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

