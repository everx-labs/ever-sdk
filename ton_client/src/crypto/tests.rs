use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::client::ParamsOfAppRequest;
use crate::crypto::boxes::encryption_box::ParamsOfCreateEncryptionBox;
use crate::crypto::encscrypt::{ParamsOfScrypt, ResultOfScrypt};
use crate::crypto::hash::{ParamsOfHash, ResultOfHash};
use crate::crypto::hdkey::{
    ParamsOfHDKeyDeriveFromXPrv, ParamsOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeyPublicFromXPrv,
    ParamsOfHDKeySecretFromXPrv, ParamsOfHDKeyXPrvFromMnemonic, ResultOfHDKeyDeriveFromXPrv,
    ResultOfHDKeyDeriveFromXPrvPath, ResultOfHDKeyPublicFromXPrv, ResultOfHDKeySecretFromXPrv,
    ResultOfHDKeyXPrvFromMnemonic,
};
use crate::crypto::keys::{
    KeyPair, ParamsOfConvertPublicKeyToTonSafeFormat, ParamsOfSign, ParamsOfVerifySignature,
    ResultOfConvertPublicKeyToTonSafeFormat, ResultOfSign, ResultOfVerifySignature, strip_secret
};
use crate::crypto::math::{
    ParamsOfFactorize, ParamsOfGenerateRandomBytes, ParamsOfModularPower, ParamsOfTonCrc16,
    ResultOfFactorize, ResultOfGenerateRandomBytes, ResultOfModularPower, ResultOfTonCrc16,
};
use crate::crypto::mnemonic::{
    ParamsOfMnemonicDeriveSignKeys, ParamsOfMnemonicFromEntropy, ParamsOfMnemonicFromRandom,
    ParamsOfMnemonicVerify, ParamsOfMnemonicWords, ResultOfMnemonicFromEntropy,
    ResultOfMnemonicFromRandom, ResultOfMnemonicVerify, ResultOfMnemonicWords,
};
use crate::crypto::nacl::{
    ParamsOfNaclBox, ParamsOfNaclBoxKeyPairFromSecret, ParamsOfNaclBoxOpen, ParamsOfNaclSecretBox,
    ParamsOfNaclSecretBoxOpen, ParamsOfNaclSign, ParamsOfNaclSignKeyPairFromSecret,
    ParamsOfNaclSignOpen, ResultOfNaclBox, ResultOfNaclBoxOpen, ResultOfNaclSign,
    ResultOfNaclSignDetached, ResultOfNaclSignOpen,
};
use crate::crypto::{ParamsOfChaCha20, ResultOfChaCha20};
use crate::crypto::boxes::crypto_box::{
    BoxEncryptionAlgorithm, ChaCha20ParamsCB, CryptoBoxSecret, ParamsOfCreateCryptoBox, 
    ParamsOfGetEncryptionBoxFromCryptoBox, ParamsOfGetSigningBoxFromCryptoBox, RegisteredCryptoBox, 
    ResultOfGetCryptoBoxInfo, ResultOfGetCryptoBoxSeedPhrase,
};
use crate::crypto::boxes::encryption_box::nacl_box::NaclBoxParamsEB;
use crate::crypto::boxes::encryption_box::nacl_secret_box::NaclSecretBoxParamsEB;
use crate::json_interface::crypto::{
    ParamsOfAppPasswordProvider, ParamsOfAppSigningBox, ResultOfAppPasswordProvider,
    ResultOfAppSigningBox,
};
use crate::tests::TestClient;
use super::*;

fn base64_from_hex(hex: &str) -> String {
    base64::encode(&hex::decode(hex).unwrap())
}

fn text_from_base64(b64: &str) -> String {
    String::from_utf8(base64::decode(b64).unwrap()).unwrap()
}

#[test]
fn encryption() {
    TestClient::init_log();
    let client = TestClient::new();
    let key = "01".repeat(32);
    let nonce = "ff".repeat(12);
    let encrypted: ResultOfChaCha20 = client
        .request(
            "crypto.chacha20",
            ParamsOfChaCha20 {
                data: base64::encode("Message"),
                key: key.clone(),
                nonce: nonce.clone(),
            },
        )
        .unwrap();
    assert_eq!(encrypted.data, "w5QOGsJodQ==");
    let decrypted: ResultOfChaCha20 = client
        .request(
            "crypto.chacha20",
            ParamsOfChaCha20 {
                data: encrypted.data.clone(),
                key: key.clone(),
                nonce: nonce.clone(),
            },
        )
        .unwrap();
    assert_eq!(decrypted.data, "TWVzc2FnZQ==");
}

#[test]
fn math() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfFactorize = client
        .request(
            "crypto.factorize",
            ParamsOfFactorize {
                composite: "17ED48941A08F981".into(),
            },
        )
        .unwrap();
    assert_eq!("494C553B", result.factors[0]);
    assert_eq!("53911073", result.factors[1]);

    let result: ResultOfModularPower = client
        .request(
            "crypto.modular_power",
            ParamsOfModularPower {
                base: "0123456789ABCDEF".into(),
                exponent: "0123".into(),
                modulus: "01234567".into(),
            },
        )
        .unwrap();
    assert_eq!("63bfdf", result.modular_power);

    let result: ResultOfTonCrc16 = client
        .request(
            "crypto.ton_crc16",
            ParamsOfTonCrc16 {
                data: base64_from_hex("0123456789abcdef"),
            },
        )
        .unwrap();
    assert_eq!(result.crc, 43349);

    let result: ResultOfGenerateRandomBytes = client
        .request(
            "crypto.generate_random_bytes",
            ParamsOfGenerateRandomBytes { length: 32 },
        )
        .unwrap();
    assert_eq!(result.bytes.len(), 44);
}

#[test]
fn hash() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfHash = client
        .request(
            "crypto.sha512",
            ParamsOfHash {
                data: base64::encode("Message to hash with sha 512"),
            },
        )
        .unwrap();
    assert_eq!("2616a44e0da827f0244e93c2b0b914223737a6129bc938b8edf2780ac9482960baa9b7c7cdb11457c1cebd5ae77e295ed94577f32d4c963dc35482991442daa5", result.hash);

    let result: ResultOfHash = client
        .request(
            "crypto.sha256",
            ParamsOfHash {
                data: base64::encode("Message to hash with sha 256"),
            },
        )
        .unwrap();
    assert_eq!(
        "16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5",
        result.hash
    );

    let result: ResultOfHash = client
        .request(
            "crypto.sha256",
            ParamsOfHash {
                data: base64_from_hex("4d65737361676520746f206861736820776974682073686120323536"),
            },
        )
        .unwrap();
    assert_eq!(
        "16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5",
        result.hash
    );

    let result: ResultOfHash = client
        .request(
            "crypto.sha256",
            ParamsOfHash {
                data: "TWVzc2FnZSB0byBoYXNoIHdpdGggc2hhIDI1Ng==".into(),
            },
        )
        .unwrap();
    assert_eq!(
        "16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5",
        result.hash
    );

    let result: ResultOfHash = client
        .request(
            "crypto.sha256",
            ParamsOfHash {
                data: base64::encode("Message to hash with sha 256"),
            },
        )
        .unwrap();
    assert_eq!(
        "16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5",
        result.hash
    );
}

#[test]
fn keys() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfConvertPublicKeyToTonSafeFormat = client
        .request(
            "crypto.convert_public_key_to_ton_safe_format",
            ParamsOfConvertPublicKeyToTonSafeFormat {
                public_key: "06117f59ade83e097e0fb33e5d29e8735bda82b3bf78a015542aaa853bb69600"
                    .into(),
            },
        )
        .unwrap();
    assert_eq!(
        "PuYGEX9Zreg-CX4Psz5dKehzW9qCs794oBVUKqqFO7aWAOTD",
        result.ton_public_key
    );

    let result: KeyPair = client
        .request_no_params("crypto.generate_random_sign_keys")
        .unwrap();
    assert_eq!(result.public.len(), 64);
    assert_eq!(result.secret.len(), 64);
    assert_ne!(result.secret, result.public);

    let result: ResultOfSign = client
        .request(
            "crypto.sign",
            ParamsOfSign {
                unsigned: base64::encode("Test Message"),
                keys: KeyPair {
                    public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e"
                        .into(),
                    secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f"
                        .into(),
                },
            },
        )
        .unwrap();
    assert_eq!(result.signed, "+wz+QO6l1slgZS5s65BNqKcu4vz24FCJz4NSAxef9lu0jFfs8x3PzSZRC+pn5k8+aJi3xYMA3BQzglQmjK3hA1Rlc3QgTWVzc2FnZQ==");
    assert_eq!(result.signature, "fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade103");

    let result: ResultOfVerifySignature = client.request("crypto.verify_signature", ParamsOfVerifySignature {
        signed: base64_from_hex("fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade10354657374204d657373616765"),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(text_from_base64(&result.unsigned), "Test Message");
}

#[test]
fn scrypt() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfScrypt = client
        .request(
            "crypto.scrypt",
            ParamsOfScrypt {
                password: base64::encode("Test Password"),
                salt: base64::encode("Test Salt"),
                log_n: 10,
                r: 8,
                p: 16,
                dk_len: 64,
            },
        )
        .unwrap();
    assert_eq!(result.key, "52e7fcf91356eca55fc5d52f16f5d777e3521f54e3c570c9bbb7df58fc15add73994e5db42be368de7ebed93c9d4f21f9be7cc453358d734b04a057d0ed3626d");
}

#[test]
fn nacl() {
    TestClient::init_log();
    let client = TestClient::new();

    // Sign

    let result: KeyPair = client
        .request(
            "crypto.nacl_sign_keypair_from_secret_key",
            ParamsOfNaclSignKeyPairFromSecret {
                secret: "8fb4f2d256e57138fb310b0a6dac5bbc4bee09eb4821223a720e5b8e1f3dd674".into(),
            },
        )
        .unwrap();
    assert_eq!(
        result.public,
        "aa5533618573860a7e1bf19f34bd292871710ed5b2eafa0dcdbb33405f2231c6"
    );

    let result: ResultOfNaclSign = client.request("crypto.nacl_sign", ParamsOfNaclSign {
        unsigned: base64::encode("Test Message"),
        secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(result.signed, "+wz+QO6l1slgZS5s65BNqKcu4vz24FCJz4NSAxef9lu0jFfs8x3PzSZRC+pn5k8+aJi3xYMA3BQzglQmjK3hA1Rlc3QgTWVzc2FnZQ==");

    let result: ResultOfNaclSignOpen = client.request("crypto.nacl_sign_open", ParamsOfNaclSignOpen {
        signed: base64_from_hex("fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade10354657374204d657373616765"),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(text_from_base64(&result.unsigned), "Test Message");

    let result: ResultOfNaclSignDetached = client.request("crypto.nacl_sign_detached", ParamsOfNaclSign {
        unsigned: base64::encode("Test Message"),
        secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(result.signature, "fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade103");
    let signature = result.signature;
    let result: ResultOfNaclSignDetachedVerify = client.request("crypto.nacl_sign_detached_verify", ParamsOfNaclSignDetachedVerify {
        unsigned: base64::encode("Test Message"),
        signature: signature.clone(),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(result.succeeded, true);

    let result: ResultOfNaclSignDetachedVerify = client.request("crypto.nacl_sign_detached_verify", ParamsOfNaclSignDetachedVerify {
        unsigned: base64::encode("Test Message 1"),
        signature: signature.clone(),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    }).unwrap();
    assert_eq!(result.succeeded, false);

    // Box

    let result: KeyPair = client.request_no_params("crypto.nacl_box_keypair").unwrap();
    assert_eq!(result.public.len(), 64);
    assert_eq!(result.secret.len(), 64);
    assert_ne!(result.public, result.secret);

    let result: KeyPair = client
        .request(
            "crypto.nacl_box_keypair_from_secret_key",
            ParamsOfNaclBoxKeyPairFromSecret {
                secret: "e207b5966fb2c5be1b71ed94ea813202706ab84253bdf4dc55232f82a1caf0d4".into(),
            },
        )
        .unwrap();
    assert_eq!(
        result.public,
        "a53b003d3ffc1e159355cb37332d67fc235a7feb6381e36c803274074dc3933a"
    );

    let result: ResultOfNaclBox = client
        .request(
            "crypto.nacl_box",
            ParamsOfNaclBox {
                decrypted: base64::encode("Test Message"),
                nonce: "cd7f99924bf422544046e83595dd5803f17536f5c9a11746".into(),
                their_public: "c4e2d9fe6a6baf8d1812b799856ef2a306291be7a7024837ad33a8530db79c6b"
                    .into(),
                secret: "d9b9dc5033fb416134e5d2107fdbacab5aadb297cb82dbdcd137d663bac59f7f".into(),
            },
        )
        .unwrap();
    assert_eq!(result.encrypted, "li4XED4kx/pjQ2qdP0eR2d/K30uN94voNADxwA==");

    let result: ResultOfNaclBoxOpen = client
        .request(
            "crypto.nacl_box_open",
            ParamsOfNaclBoxOpen {
                encrypted: base64_from_hex(
                    "962e17103e24c7fa63436a9d3f4791d9dfcadf4b8df78be83400f1c0",
                ),
                nonce: "cd7f99924bf422544046e83595dd5803f17536f5c9a11746".into(),
                their_public: "c4e2d9fe6a6baf8d1812b799856ef2a306291be7a7024837ad33a8530db79c6b"
                    .into(),
                secret: "d9b9dc5033fb416134e5d2107fdbacab5aadb297cb82dbdcd137d663bac59f7f".into(),
            },
        )
        .unwrap();
    assert_eq!(text_from_base64(&result.decrypted), "Test Message");

    // Secret box

    let result: ResultOfNaclBox = client
        .request(
            "crypto.nacl_secret_box",
            ParamsOfNaclSecretBox {
                decrypted: base64::encode("Test Message"),
                nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
                key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
            },
        )
        .unwrap();
    assert_eq!(result.encrypted, "JL7ejKWe2KXmrsns41yfXoQF0t/C1Q8RGyzQ2A==");

    let result: ResultOfNaclBoxOpen = client
        .request(
            "crypto.nacl_secret_box_open",
            ParamsOfNaclSecretBoxOpen {
                encrypted: base64_from_hex(
                    "24bede8ca59ed8a5e6aec9ece35c9f5e8405d2dfc2d50f111b2cd0d8",
                ),
                nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
                key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
            },
        )
        .unwrap();
    assert_eq!(text_from_base64(&result.decrypted), "Test Message");

    let e: ResultOfNaclBox = client
        .request(
            "crypto.nacl_secret_box",
            ParamsOfNaclSecretBox {
                decrypted: base64::encode("Text with \' and \" and : {}"),
                nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
                key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
            },
        )
        .unwrap();
    let d: ResultOfNaclBoxOpen = client
        .request(
            "crypto.nacl_secret_box_open",
            ParamsOfNaclSecretBoxOpen {
                encrypted: e.encrypted,
                nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
                key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
            },
        )
        .unwrap();
    assert_eq!(
        text_from_base64(&d.decrypted),
        "Text with \' and \" and : {}"
    );
}

#[test]
fn mnemonic() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfMnemonicWords = client
        .request(
            "crypto.mnemonic_words",
            ParamsOfMnemonicWords { dictionary: None },
        )
        .unwrap();
    assert_eq!(result.words.split(" ").count(), 2048);

    for dictionary in 1..9 {
        for word_count in &[12u8, 15, 18, 21, 24] {
            let result: ResultOfMnemonicFromRandom = client
                .request(
                    "crypto.mnemonic_from_random",
                    ParamsOfMnemonicFromRandom {
                        dictionary: Some(dictionary),
                        word_count: Some(*word_count),
                    },
                )
                .unwrap();
            assert_eq!(result.phrase.split(" ").count(), *word_count as usize);
        }
    }

    let result: ResultOfMnemonicFromEntropy = client
        .request(
            "crypto.mnemonic_from_entropy",
            ParamsOfMnemonicFromEntropy {
                entropy: "00112233445566778899AABBCCDDEEFF".into(),
                dictionary: Some(1),
                word_count: Some(12),
            },
        )
        .unwrap();
    assert_eq!(
        result.phrase,
        "abandon math mimic master filter design carbon crystal rookie group knife young"
    );

    for dictionary in 1..9 {
        for word_count in &[12u8, 15, 18, 21, 24] {
            let result: ResultOfMnemonicFromRandom = client
                .request(
                    "crypto.mnemonic_from_random",
                    ParamsOfMnemonicFromRandom {
                        dictionary: Some(dictionary),
                        word_count: Some(*word_count),
                    },
                )
                .unwrap();
            let verify_result: ResultOfMnemonicVerify = client
                .request(
                    "crypto.mnemonic_verify",
                    ParamsOfMnemonicVerify {
                        phrase: result.phrase,
                        dictionary: Some(dictionary),
                        word_count: Some(*word_count),
                    },
                )
                .unwrap();
            assert_eq!(verify_result.valid, true);
        }
    }

    let result: ResultOfMnemonicVerify = client
        .request(
            "crypto.mnemonic_verify",
            ParamsOfMnemonicVerify {
                phrase: "one two".into(),
                dictionary: None,
                word_count: None,
            },
        )
        .unwrap();
    assert_eq!(result.valid, false);

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle".into(),
        path: None,
        dictionary: Some(0),
        word_count: Some(24),
    }).unwrap();
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client
        .request(
            "crypto.convert_public_key_to_ton_safe_format",
            ParamsOfConvertPublicKeyToTonSafeFormat {
                public_key: result.public,
            },
        )
        .unwrap();
    assert_eq!(
        result.ton_public_key,
        "PuYTvCuf__YXhp-4jv3TXTHL0iK65ImwxG0RGrYc1sP3H4KS"
    );

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle".into(),
        path: Some("m".into()),
        dictionary: Some(0),
        word_count: Some(24),
    }).unwrap();
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client
        .request(
            "crypto.convert_public_key_to_ton_safe_format",
            ParamsOfConvertPublicKeyToTonSafeFormat {
                public_key: result.public,
            },
        )
        .unwrap();
    assert_eq!(
        result.ton_public_key,
        "PubDdJkMyss2qHywFuVP1vzww0TpsLxnRNnbifTCcu-XEgW0"
    );

    let result: KeyPair = client.request(
        "crypto.mnemonic_derive_sign_keys",
        ParamsOfMnemonicDeriveSignKeys {
            phrase:
                "abandon math mimic master filter design carbon crystal rookie group knife young"
                    .into(),
            path: None,
            dictionary: None,
            word_count: None,
        },
    ).unwrap();
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client
        .request(
            "crypto.convert_public_key_to_ton_safe_format",
            ParamsOfConvertPublicKeyToTonSafeFormat {
                public_key: result.public,
            },
        )
        .unwrap();
    assert_eq!(
        result.ton_public_key,
        "PuZhw8W5ejPJwKA68RL7sn4_RNmeH4BIU_mEK7em5d4_-cIx"
    );

    let result: ResultOfMnemonicFromRandom = client
        .request(
            "crypto.mnemonic_from_random",
            ParamsOfMnemonicFromRandom {
                dictionary: None,
                word_count: None,
            },
        )
        .unwrap();
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromRandom = client
        .request(
            "crypto.mnemonic_from_random",
            ParamsOfMnemonicFromRandom {
                dictionary: Some(0),
                word_count: Some(12),
            },
        )
        .unwrap();
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromRandom = client
        .request(
            "crypto.mnemonic_from_random",
            ParamsOfMnemonicFromRandom {
                dictionary: Some(1),
                word_count: Some(12),
            },
        )
        .unwrap();
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromEntropy = client
        .request(
            "crypto.mnemonic_from_entropy",
            ParamsOfMnemonicFromEntropy {
                entropy: "2199ebe996f14d9e4e2595113ad1e627".into(),
                dictionary: None,
                word_count: None,
            },
        )
        .unwrap();

    let result: KeyPair = client
        .request(
            "crypto.mnemonic_derive_sign_keys",
            ParamsOfMnemonicDeriveSignKeys {
                phrase: result.phrase,
                path: None,
                dictionary: None,
                word_count: None,
            },
        )
        .unwrap();
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client
        .request(
            "crypto.convert_public_key_to_ton_safe_format",
            ParamsOfConvertPublicKeyToTonSafeFormat {
                public_key: result.public,
            },
        )
        .unwrap();
    assert_eq!(
        result.ton_public_key,
        "PuZdw_KyXIzo8IksTrERN3_WoAoYTyK7OvM-yaLk711sUIB3"
    );
}

#[test]
fn hdkey() {
    TestClient::init_log();
    let client = TestClient::new();

    let master: ResultOfHDKeyXPrvFromMnemonic = client
        .request(
            "crypto.hdkey_xprv_from_mnemonic",
            ParamsOfHDKeyXPrvFromMnemonic {
                dictionary: None,
                word_count: None,
                phrase:
                    "abuse boss fly battle rubber wasp afraid hamster guide essence vibrant tattoo"
                        .into(),
            },
        )
        .unwrap();
    assert_eq!(master.xprv, "xprv9s21ZrQH143K25JhKqEwvJW7QAiVvkmi4WRenBZanA6kxHKtKAQQKwZG65kCyW5jWJ8NY9e3GkRoistUjjcpHNsGBUv94istDPXvqGNuWpC");

    let result: ResultOfHDKeySecretFromXPrv = client
        .request(
            "crypto.hdkey_secret_from_xprv",
            ParamsOfHDKeySecretFromXPrv {
                xprv: master.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.secret,
        "0c91e53128fa4d67589d63a6c44049c1068ec28a63069a55ca3de30c57f8b365"
    );

    let result: ResultOfHDKeyPublicFromXPrv = client
        .request(
            "crypto.hdkey_public_from_xprv",
            ParamsOfHDKeyPublicFromXPrv {
                xprv: master.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.public,
        "7b70008d0c40992283d488b1046739cf827afeabf647a5f07c4ad1e7e45a6f89"
    );

    let child: ResultOfHDKeyDeriveFromXPrv = client
        .request(
            "crypto.hdkey_derive_from_xprv",
            ParamsOfHDKeyDeriveFromXPrv {
                xprv: master.xprv.clone(),
                child_index: 0,
                hardened: false,
            },
        )
        .unwrap();
    assert_eq!(child.xprv, "xprv9uZwtSeoKf1swgAkVVCEUmC2at6t7MCJoHnBbn1MWJZyxQ4cySkVXPyNh7zjf9VjsP4vEHDDD2a6R35cHubg4WpzXRzniYiy8aJh1gNnBKv");

    let result: ResultOfHDKeySecretFromXPrv = client
        .request(
            "crypto.hdkey_secret_from_xprv",
            ParamsOfHDKeySecretFromXPrv {
                xprv: child.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.secret,
        "518afc6489b61d4b738ee9ad9092815fa014ffa6e9a280fa17f84d95f31adb91"
    );

    let result: ResultOfHDKeyPublicFromXPrv = client
        .request(
            "crypto.hdkey_public_from_xprv",
            ParamsOfHDKeyPublicFromXPrv {
                xprv: child.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.public,
        "b45e1297a5e767341a6eaaac9e20f8ccd7556a0106298316f1272e461b6fbe98"
    );

    let second: ResultOfHDKeyDeriveFromXPrvPath = client
        .request(
            "crypto.hdkey_derive_from_xprv_path",
            ParamsOfHDKeyDeriveFromXPrvPath {
                xprv: master.xprv.clone(),
                path: "m/44'/60'/0'/0'".into(),
            },
        )
        .unwrap();
    assert_eq!(second.xprv, "xprvA1KNMo63UcGjmDF1bX39Cw2BXGUwrwMjeD5qvQ3tA3qS3mZQkGtpf4DHq8FDLKAvAjXsYGLHDP2dVzLu9ycta8PXLuSYib2T3vzLf3brVgZ");

    let result: ResultOfHDKeySecretFromXPrv = client
        .request(
            "crypto.hdkey_secret_from_xprv",
            ParamsOfHDKeySecretFromXPrv {
                xprv: second.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.secret,
        "1c566ade41169763b155761406d3cef08b29b31cf8014f51be08c0cb4e67c5e1"
    );

    let result: ResultOfHDKeyPublicFromXPrv = client
        .request(
            "crypto.hdkey_public_from_xprv",
            ParamsOfHDKeyPublicFromXPrv {
                xprv: second.xprv.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        result.public,
        "302a832bad9e5c9906422a82c28b39ae465dcd60178480f7309e183ee34b5e83"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_signing_box() {
    let client = std::sync::Arc::new(TestClient::new());
    let client_copy = client.clone();

    let keys = client.generate_sign_keys();

    let keys_box_handle = client
        .request_async::<_, RegisteredSigningBox>(
            "crypto.get_signing_box",
            keys.clone(),
        )
        .await
        .unwrap()
        .handle
        .0;

    // external signing box uses keys box inside
    let callback = move |request: crate::client::ParamsOfAppRequest, _: u32| {
        let client = client_copy.clone();
        tokio::spawn(async move {
            match serde_json::from_value(request.request_data).unwrap() {
                ParamsOfAppSigningBox::GetPublicKey => {
                    let result: ResultOfSigningBoxGetPublicKey = client
                        .request_async(
                            "crypto.signing_box_get_public_key",
                            RegisteredSigningBox {
                                handle: keys_box_handle.into()
                            },
                        ).await.unwrap();
                    client.resolve_app_request(
                        request.app_request_id,
                        ResultOfAppSigningBox::GetPublicKey { public_key: result.pubkey }
                    ).await;
                },
                ParamsOfAppSigningBox::Sign { unsigned } => {
                    let result: ResultOfSigningBoxSign = client
                        .request_async(
                            "crypto.signing_box_sign",
                            ParamsOfSigningBoxSign {
                                signing_box: keys_box_handle.into(),
                                unsigned,
                            },
                        ).await.unwrap();
                    client.resolve_app_request(
                        request.app_request_id,
                        ResultOfAppSigningBox::Sign { signature: result.signature }
                    ).await;
                }
            }
        });
        futures::future::ready(())
    };

    let external_box: RegisteredSigningBox = client.request_async_callback(
        "crypto.register_signing_box",
        (),
        callback
    ).await.unwrap();


    let box_pubkey: ResultOfSigningBoxGetPublicKey = client
        .request_async(
            "crypto.signing_box_get_public_key",
            RegisteredSigningBox {
                handle: external_box.handle.clone(),
            },
        ).await.unwrap();

    assert_eq!(box_pubkey.pubkey, keys.public);

    let unsigned = base64::encode("Test Message");
    let box_sign: ResultOfSigningBoxSign = client
        .request_async(
            "crypto.signing_box_sign",
            ParamsOfSigningBoxSign {
                signing_box: external_box.handle.clone(),
                unsigned: unsigned.clone(),
            },
        ).await.unwrap();

    let keys_sign: ResultOfSign = client
        .request(
            "crypto.sign",
            ParamsOfSign {
                unsigned,
                keys,
            },
        ).unwrap();

    assert_eq!(box_sign.signature, keys_sign.signature);

    let _: () = client
        .request_async(
            "crypto.remove_signing_box",
            RegisteredSigningBox {
                handle: external_box.handle,
        },
    ).await.unwrap();

    let _: () = client
        .request_async(
            "crypto.remove_signing_box",
            RegisteredSigningBox {
                handle: keys_box_handle.into(),
        },
    ).await.unwrap();
}

#[test]
fn test_strip_secret() {
    assert_eq!(strip_secret(""), r#""""#);
    assert_eq!(strip_secret("0123456"), r#""0123456""#);
    assert_eq!(strip_secret("01234567"), r#""01234567""#);
    assert_eq!(strip_secret("012345678"), r#""01234567..." (9 chars)"#);
    assert_eq!(strip_secret("0123456789"), r#""01234567..." (10 chars)"#);
    assert_eq!(
        strip_secret("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        r#""01234567..." (64 chars)"#
    );
}

#[test]
fn test_debug_keypair_secret_stripped() {
    let keypair = KeyPair::new(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        "9123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()
    );

    assert_eq!(
        format!("{:?}", keypair),
        "KeyPair { \
            public: \"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\", \
            secret: \"91234567...\" (64 chars) \
        }"
    )
}

async fn test_aes_params(key: &str, data: &str, encrypted: &str) {
    let client = std::sync::Arc::new(TestClient::new());

    let iv = hex::encode(&std::fs::read("src/crypto/test_data/aes.iv.bin").unwrap());
    let key = hex::encode(&std::fs::read(key).unwrap());
    let data = std::fs::read(data).unwrap();
    let encrypted = base64::encode(&std::fs::read(encrypted).unwrap());

    let box_handle = client
        .request_async::<_, RegisteredEncryptionBox>(
            "crypto.create_encryption_box",
            ParamsOfCreateEncryptionBox {
                algorithm: EncryptionAlgorithm::AES(AesParamsEB {
                    key: key.clone(),
                    iv: Some(iv.clone()),
                    mode: CipherMode::CBC,
                })
            },
        )
        .await
        .unwrap()
        .handle;

    let result: ResultOfEncryptionBoxEncrypt = client
        .request_async(
            "crypto.encryption_box_encrypt",
            ParamsOfEncryptionBoxEncrypt {
                encryption_box: box_handle.clone(),
                data: base64::encode(&data.clone()),
            },
        ).await.unwrap();

    assert_eq!(result.data, encrypted);

    let result: ResultOfEncryptionBoxDecrypt = client
        .request_async(
            "crypto.encryption_box_decrypt",
            ParamsOfEncryptionBoxDecrypt {
                encryption_box: box_handle.clone(),
                data: encrypted,
            },
        ).await.unwrap();

    assert_eq!(
        base64::decode(&result.data).unwrap()[..data.len()],
        data
    );

    let _: () = client
        .request_async(
            "crypto.remove_encryption_box",
            RegisteredEncryptionBox {
                handle: box_handle,
        },
    ).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_aes_encryption_box() {
    test_aes_params(
        "src/crypto/test_data/aes128.key.bin",
        "src/crypto/test_data/aes.plaintext.bin",
        "src/crypto/test_data/cbc-aes128.ciphertext.bin"
    ).await;

    test_aes_params(
        "src/crypto/test_data/aes256.key.bin",
        "src/crypto/test_data/aes.plaintext.for.padding.bin",
        "src/crypto/test_data/cbc-aes256.ciphertext.padded.bin"
    ).await;
}

#[tokio::test]
async fn test_chacha20_encryption_box() {
    let client = Arc::new(TestClient::new());

    let key = "01".repeat(32);
    let nonce = "ff".repeat(12);

    let box_handle = client
        .request_async::<_, RegisteredEncryptionBox>(
            "crypto.create_encryption_box",
            ParamsOfCreateEncryptionBox {
                algorithm: EncryptionAlgorithm::ChaCha20(
                    super::boxes::encryption_box::chacha20::ChaCha20ParamsEB {
                        key: key.clone(),
                        nonce: nonce.clone(),
                    }
                )
            },
        )
        .await
        .unwrap()
        .handle;

    let decrypted: String = base64::encode("Message");

    let result: ResultOfEncryptionBoxEncrypt = client
        .request_async(
            "crypto.encryption_box_encrypt",
            ParamsOfEncryptionBoxEncrypt {
                encryption_box: box_handle.clone(),
                data: decrypted.clone(),
            },
        ).await.unwrap();

    assert_eq!(result.data, "w5QOGsJodQ==");

    let result: ResultOfEncryptionBoxDecrypt = client
        .request_async(
            "crypto.encryption_box_decrypt",
            ParamsOfEncryptionBoxDecrypt {
                encryption_box: box_handle.clone(),
                data: result.data,
            },
        ).await.unwrap();

    assert_eq!(result.data, decrypted);

    let result: ResultOfEncryptionBoxGetInfo = client
        .request_async(
            "crypto.encryption_box_get_info",
            ParamsOfEncryptionBoxGetInfo {
                encryption_box: box_handle,
            },
        ).await.unwrap();

    assert_eq!(
        result.info,
        EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("ChaCha20".to_string()),
            options: Some(json!({
                "nonce": nonce,
            })),
            public: None,
        },
    );
}

#[tokio::test]
async fn test_nacl_encryption_box() {
    let client = Arc::new(TestClient::new());

    const THEIR_PUBLIC: &str = "c4e2d9fe6a6baf8d1812b799856ef2a306291be7a7024837ad33a8530db79c6b";
    const NONCE: &str = "cd7f99924bf422544046e83595dd5803f17536f5c9a11746";
    const SECRET: &str = "d9b9dc5033fb416134e5d2107fdbacab5aadb297cb82dbdcd137d663bac59f7f";

    let box_handle = client
        .request_async::<_, RegisteredEncryptionBox>(
            "crypto.create_encryption_box",
            ParamsOfCreateEncryptionBox {
                algorithm: EncryptionAlgorithm::NaclBox(
                    NaclBoxParamsEB {
                        their_public: THEIR_PUBLIC.to_string(),
                        secret: SECRET.to_string(),
                        nonce: NONCE.to_string(),
                    }
                )
            },
        )
        .await
        .unwrap()
        .handle;

    let decrypted: String = base64::encode("Test Message");

    let result: ResultOfEncryptionBoxEncrypt = client
        .request_async(
            "crypto.encryption_box_encrypt",
            ParamsOfEncryptionBoxEncrypt {
                encryption_box: box_handle.clone(),
                data: decrypted.clone(),
            },
        ).await.unwrap();

    assert_eq!(result.data, "li4XED4kx/pjQ2qdP0eR2d/K30uN94voNADxwA==");

    let result: ResultOfEncryptionBoxDecrypt = client
        .request_async(
            "crypto.encryption_box_decrypt",
            ParamsOfEncryptionBoxDecrypt {
                encryption_box: box_handle.clone(),
                data: result.data,
            },
        ).await.unwrap();

    assert_eq!(result.data, decrypted);

    let result: ResultOfEncryptionBoxGetInfo = client
        .request_async(
            "crypto.encryption_box_get_info",
            ParamsOfEncryptionBoxGetInfo {
                encryption_box: box_handle,
            },
        ).await.unwrap();

    assert_eq!(
        result.info,
        EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("NaclBox".to_string()),
            options: Some(json!({
                "their_public": THEIR_PUBLIC,
                "nonce": NONCE,
            })),
            public: None,
        },
    );
}

#[tokio::test]
async fn test_nacl_secret_encryption_box() {
    let client = Arc::new(TestClient::new());

    const KEY: &str = "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8";
    const NONCE: &str = "2a33564717595ebe53d91a785b9e068aba625c8453a76e45";

    let box_handle = client
        .request_async::<_, RegisteredEncryptionBox>(
            "crypto.create_encryption_box",
            ParamsOfCreateEncryptionBox {
                algorithm: EncryptionAlgorithm::NaclSecretBox(
                    NaclSecretBoxParamsEB {
                        key: KEY.to_string(),
                        nonce: NONCE.to_string(),
                    }
                )
            },
        )
        .await
        .unwrap()
        .handle;

    let decrypted: String = base64::encode("Test Message");

    let result: ResultOfEncryptionBoxEncrypt = client
        .request_async(
            "crypto.encryption_box_encrypt",
            ParamsOfEncryptionBoxEncrypt {
                encryption_box: box_handle.clone(),
                data: decrypted.clone(),
            },
        ).await.unwrap();

    assert_eq!(result.data, "JL7ejKWe2KXmrsns41yfXoQF0t/C1Q8RGyzQ2A==");

    let result: ResultOfEncryptionBoxDecrypt = client
        .request_async(
            "crypto.encryption_box_decrypt",
            ParamsOfEncryptionBoxDecrypt {
                encryption_box: box_handle.clone(),
                data: result.data,
            },
        ).await.unwrap();

    assert_eq!(result.data, decrypted);

    let result: ResultOfEncryptionBoxGetInfo = client
        .request_async(
            "crypto.encryption_box_get_info",
            ParamsOfEncryptionBoxGetInfo {
                encryption_box: box_handle,
            },
        ).await.unwrap();

    assert_eq!(
        result.info,
        EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("NaclSecretBox".to_string()),
            options: Some(json!({
                "nonce": NONCE,
            })),
            public: None,
        },
    );
}

fn get_callback(
    client: &Arc<TestClient>,
    password_hash: &Arc<String>,
    on_callback: impl Fn() + Send + Sync + 'static,
) -> impl Fn(ParamsOfAppRequest, u32) -> futures::future::Ready<()> {
    let client = Arc::clone(client);
    let password_hash = Arc::clone(password_hash);
    let on_callback = Arc::new(on_callback);
    move |request: ParamsOfAppRequest, _: u32| {
        let client = Arc::clone(&client);
        let password_hash = Arc::clone(&password_hash);
        let on_callback = Arc::clone(&on_callback);
        tokio::spawn(async move {
            on_callback();
            let ParamsOfAppPasswordProvider::GetPassword { encryption_public_key } =
                serde_json::from_value(request.request_data).unwrap();

            let KeyPair { public, secret } =
                client.request_no_params("crypto.nacl_box_keypair").unwrap();

            let ResultOfNaclBox { encrypted } =
                client.request_async(
                    "crypto.nacl_box",
                    ParamsOfNaclBox {
                        decrypted: base64::encode(&hex::decode(password_hash.as_ref()).unwrap()),
                        nonce: encryption_public_key[..48].to_string(),
                        their_public: encryption_public_key,
                        secret,
                    },
                ).await
                    .unwrap();

            client.resolve_app_request(
                request.app_request_id,
                ResultOfAppPasswordProvider::GetPassword {
                    encrypted_password: encrypted,
                    app_encryption_pubkey: public,
                },
            ).await;
        });
        futures::future::ready(())
    }
}

#[tokio::test]
async fn test_crypto_boxes() -> ton_types::Result<()> {
    let client = Arc::new(TestClient::new());
    let password_hash = Arc::new(
        "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".to_string()
    );
    let salt = "123123123";

    let RegisteredCryptoBox { handle } = client
        .request_async_callback(
            "crypto.create_crypto_box",
            ParamsOfCreateCryptoBox {
                secret_encryption_salt: salt.to_string(),
                secret: CryptoBoxSecret::RandomSeedPhrase {
                    dictionary: Default::default(),
                    wordcount: 12,
                }
            },
            get_callback(&client, &password_hash, || ()),
        ).await?;

    let seed_phrase: ResultOfGetCryptoBoxSeedPhrase = client.request_async(
        "crypto.get_crypto_box_seed_phrase",
        RegisteredCryptoBox { handle },
    ).await?;

    let verify_result: ResultOfMnemonicVerify = client
        .request(
            "crypto.mnemonic_verify",
            ParamsOfMnemonicVerify {
                phrase: seed_phrase.phrase.clone(),
                dictionary: Some(0),
                word_count: Some(12),
            },
        )
        .unwrap();

    assert!(verify_result.valid);

    let crypto_box_info: ResultOfGetCryptoBoxInfo = client.request_async(
        "crypto.get_crypto_box_info",
        RegisteredCryptoBox { handle },
    ).await?;

    let RegisteredCryptoBox { handle } = client
        .request_async_callback(
            "crypto.create_crypto_box",
            ParamsOfCreateCryptoBox {
                secret_encryption_salt: salt.to_string(),
                secret: CryptoBoxSecret::EncryptedSecret {
                    encrypted_secret: crypto_box_info.encrypted_secret.clone(),
                }
            },
            get_callback(&client, &password_hash, || ()),
        ).await?;

    let seed_phrase2: ResultOfGetCryptoBoxSeedPhrase = client.request_async(
        "crypto.get_crypto_box_seed_phrase",
        RegisteredCryptoBox { handle },
    ).await?;

    assert_eq!(seed_phrase.phrase, seed_phrase2.phrase);

    let RegisteredCryptoBox { handle } = client
        .request_async_callback(
            "crypto.create_crypto_box",
            ParamsOfCreateCryptoBox {
                secret_encryption_salt: salt.to_string(),
                secret: CryptoBoxSecret::PredefinedSeedPhrase {
                    phrase: seed_phrase.phrase.clone(),
                    dictionary: 0,
                    wordcount: 12,
                }
            },
            get_callback(&client, &password_hash, || ()),
        ).await?;

    let seed_phrase3: ResultOfGetCryptoBoxSeedPhrase = client.request_async(
        "crypto.get_crypto_box_seed_phrase",
        RegisteredCryptoBox { handle },
    ).await?;

    assert_eq!(seed_phrase.phrase, seed_phrase3.phrase);

    Ok(())
}

#[tokio::test]
async fn test_crypto_box_signing_boxes() -> ton_types::Result<()> {
    let client = Arc::new(TestClient::new());
    let password_hash = Arc::new(
        "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".to_string()
    );
    let salt = "123123123";
    let callback_calls_counter = Arc::new(AtomicUsize::new(0));
    let callback_calls_counter_copy = Arc::clone(&callback_calls_counter);

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 0);

    let crypto_box: RegisteredCryptoBox = client.request_async_callback(
        "crypto.create_crypto_box",
        ParamsOfCreateCryptoBox {
            secret_encryption_salt: salt.to_string(),
            secret: CryptoBoxSecret::RandomSeedPhrase {
                dictionary: Default::default(),
                wordcount: 12,
            }
        },
        get_callback(&client, &password_hash, move || {
            callback_calls_counter_copy.fetch_add(1, Ordering::Relaxed);
        }),
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 1);

    let signing_box: RegisteredSigningBox = client.request_async(
        "crypto.get_signing_box_from_crypto_box",
        ParamsOfGetSigningBoxFromCryptoBox {
            handle: crypto_box.handle.0,
            hdpath: None,
            secret_lifetime: None,
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 1);

    let ResultOfSigningBoxGetPublicKey { pubkey } = client.request_async(
        "crypto.signing_box_get_public_key",
        RegisteredSigningBox {
            handle: signing_box.handle.clone(),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 2);
    assert_eq!(pubkey.len(), 64);

    let _: ResultOfSigningBoxGetPublicKey = client.request_async(
        "crypto.signing_box_get_public_key",
        RegisteredSigningBox {
            handle: signing_box.handle.clone(),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 3);

    let signing_box: RegisteredSigningBox = client.request_async(
        "crypto.get_signing_box_from_crypto_box",
        ParamsOfGetSigningBoxFromCryptoBox {
            handle: crypto_box.handle.0,
            hdpath: None,
            secret_lifetime: Some(u32::MAX),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 3);

    for _ in 0..3 {
        let _: ResultOfSigningBoxGetPublicKey = client.request_async(
            "crypto.signing_box_get_public_key",
            RegisteredSigningBox {
                handle: signing_box.handle.clone(),
            }
        ).await?;

        assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 4);
    }

    client.request_async(
        "crypto.clear_crypto_box_secret_cache",
        RegisteredCryptoBox {
            handle: crypto_box.handle.clone(),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 4);

    for _ in 0..3 {
        let _: ResultOfSigningBoxGetPublicKey = client.request_async(
            "crypto.signing_box_get_public_key",
            RegisteredSigningBox {
                handle: signing_box.handle.clone(),
            }
        ).await?;

        assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 5);
    }

    Ok(())
}

#[tokio::test]
async fn test_crypto_box_encryption_boxes() -> ton_types::Result<()> {
    let client = Arc::new(TestClient::new());
    let password_hash = Arc::new(
        "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".to_string()
    );
    let nonce = "ff".repeat(12);
    let salt = "123123123";
    let callback_calls_counter = Arc::new(AtomicUsize::new(0));
    let callback_calls_counter_copy = Arc::clone(&callback_calls_counter);

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 0);

    let crypto_box: RegisteredCryptoBox = client.request_async_callback(
        "crypto.create_crypto_box",
        ParamsOfCreateCryptoBox {
            secret_encryption_salt: salt.to_string(),
            secret: CryptoBoxSecret::RandomSeedPhrase {
                dictionary: Default::default(),
                wordcount: 12,
            }
        },
        get_callback(&client, &password_hash, move || {
            callback_calls_counter_copy.fetch_add(1, Ordering::Relaxed);
        }),
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 1);

    let encryption_box: RegisteredEncryptionBox = client.request_async(
        "crypto.get_encryption_box_from_crypto_box",
        ParamsOfGetEncryptionBoxFromCryptoBox {
            handle: crypto_box.handle.0,
            hdpath: None,
            algorithm: BoxEncryptionAlgorithm::ChaCha20(ChaCha20ParamsCB {
                nonce: nonce.clone(),
            }),
            secret_lifetime: None,
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 1);

    let result: ResultOfEncryptionBoxGetInfo = client
        .request_async(
            "crypto.encryption_box_get_info",
            ParamsOfEncryptionBoxGetInfo {
                encryption_box: encryption_box.handle.clone(),
            },
        ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 2);

    assert_eq!(
        result.info,
        EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("ChaCha20".to_string()),
            options: Some(json!({
                "nonce": nonce,
            })),
            public: None,
        },
    );

    let _: ResultOfEncryptionBoxGetInfo = client
        .request_async(
            "crypto.encryption_box_get_info",
            ParamsOfEncryptionBoxGetInfo {
                encryption_box: encryption_box.handle.clone(),
            },
        ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 3);

    let encryption_box: RegisteredEncryptionBox = client.request_async(
        "crypto.get_encryption_box_from_crypto_box",
        ParamsOfGetEncryptionBoxFromCryptoBox {
            handle: crypto_box.handle.0,
            hdpath: None,
            algorithm: BoxEncryptionAlgorithm::ChaCha20(ChaCha20ParamsCB {
                nonce: nonce.clone(),
            }),
            secret_lifetime: Some(u32::MAX),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 3);

    for _ in 0..3 {
        let _: ResultOfEncryptionBoxGetInfo = client
            .request_async(
                "crypto.encryption_box_get_info",
                ParamsOfEncryptionBoxGetInfo {
                    encryption_box: encryption_box.handle.clone(),
                },
            ).await?;

        assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 4);
    }

    client.request_async(
        "crypto.clear_crypto_box_secret_cache",
        RegisteredCryptoBox {
            handle: crypto_box.handle.clone(),
        }
    ).await?;

    assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 4);

    for _ in 0..3 {
        let _: ResultOfEncryptionBoxGetInfo = client
            .request_async(
                "crypto.encryption_box_get_info",
                ParamsOfEncryptionBoxGetInfo {
                    encryption_box: encryption_box.handle.clone(),
                },
            ).await?;

        assert_eq!(callback_calls_counter.load(Ordering::Relaxed), 5);
    }

    Ok(())
}
