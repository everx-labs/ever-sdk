use crate::tests::TestClient;
use crate::crypto::math::{
    ResultOfFactorize, ParamsOfFactorize, ResultOfModularPower, ParamsOfModularPower,
    ResultOfTonCrc16, ParamsOfTonCrc16, ResultOfGenerateRandomBytes, ParamsOfGenerateRandomBytes,
};
use crate::crypto::hash::{ResultOfHash, ParamsOfHash};
use crate::crypto::keys::{ResultOfConvertPublicKeyToTonSafeFormat, ParamsOfConvertPublicKeyToTonSafeFormat, KeyPair, ResultOfSign, ParamsOfSign, ResultOfVerifySignature, ParamsOfVerifySignature};
use crate::crypto::scrypt::{ResultOfScrypt, ParamsOfScrypt};
use crate::crypto::nacl::{ParamsOfNaclSignKeyPairFromSecret, ParamsOfNaclSign, ResultOfNaclSign, ResultOfNaclSignOpen, ParamsOfNaclSignOpen, ResultOfNaclSignDetached, ParamsOfNaclBoxKeyPairFromSecret, ResultOfNaclBox, ParamsOfNaclBox, ResultOfNaclBoxOpen, ParamsOfNaclBoxOpen, ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen};
use crate::crypto::mnemonic::{ResultOfMnemonicWords, ParamsOfMnemonicWords, ParamsOfMnemonicFromRandom, ResultOfMnemonicFromRandom, ResultOfMnemonicFromEntropy, ParamsOfMnemonicFromEntropy, ResultOfMnemonicVerify, ParamsOfMnemonicVerify, ParamsOfMnemonicDeriveSignKeys};
use crate::crypto::hdkey::{ResultOfHDKeyXPrvFromMnemonic, ParamsOfHDKeyXPrvFromMnemonic, ResultOfHDKeySecretFromXPrv, ParamsOfHDKeySecretFromXPrv, ResultOfHDKeyPublicFromXPrv, ParamsOfHDKeyPublicFromXPrv, ResultOfHDKeyDeriveFromXPrv, ParamsOfHDKeyDeriveFromXPrv, ResultOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeyDeriveFromXPrvPath};

fn base64_from_hex(hex: &str) -> String {
    base64::encode(&hex::decode(hex).unwrap())
}

fn hex_from_base64(b64: &str) -> String {
    hex::encode(&base64::decode(b64).unwrap())
}

fn text_from_base64(b64: &str) -> String {
    String::from_utf8(base64::decode(b64).unwrap()).unwrap()
}

#[test]
fn math() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfFactorize = client.request("crypto.factorize", ParamsOfFactorize {
        composite: "17ED48941A08F981".into()
    });
    assert_eq!("494C553B", result.factors[0]);
    assert_eq!("53911073", result.factors[1]);

    let result: ResultOfModularPower = client.request("crypto.modular_power",
        ParamsOfModularPower {
            base: "0123456789ABCDEF".into(),
            exponent: "0123".into(),
            modulus: "01234567".into(),
        });
    assert_eq!("63bfdf", result.modular_power);

    let result: ResultOfTonCrc16 = client.request("crypto.ton_crc16", ParamsOfTonCrc16 {
        data: base64_from_hex("0123456789abcdef"),
    });
    assert_eq!(result.crc, 43349);

    let result: ResultOfGenerateRandomBytes = client.request("crypto.generate_random_bytes",
        ParamsOfGenerateRandomBytes {
            length: 32,
        });
    assert_eq!(result.bytes.len(), 44);
}


#[test]
fn hash() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfHash = client.request("crypto.sha512", ParamsOfHash {
        data: base64::encode("Message to hash with sha 512"),
    });
    assert_eq!("2616a44e0da827f0244e93c2b0b914223737a6129bc938b8edf2780ac9482960baa9b7c7cdb11457c1cebd5ae77e295ed94577f32d4c963dc35482991442daa5", result.hash);

    let result: ResultOfHash = client.request("crypto.sha256", ParamsOfHash {
        data: base64::encode("Message to hash with sha 256"),
    });
    assert_eq!("16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5", result.hash);

    let result: ResultOfHash = client.request("crypto.sha256", ParamsOfHash {
        data: base64_from_hex("4d65737361676520746f206861736820776974682073686120323536"),
    });
    assert_eq!("16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5", result.hash);

    let result: ResultOfHash = client.request("crypto.sha256", ParamsOfHash {
        data: "TWVzc2FnZSB0byBoYXNoIHdpdGggc2hhIDI1Ng==".into(),
    });
    assert_eq!("16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5", result.hash);

    let result: ResultOfHash = client.request("crypto.sha256", ParamsOfHash {
        data: base64::encode("Message to hash with sha 256"),
    });
    assert_eq!("16fd057308dd358d5a9b3ba2de766b2dfd5e308478fc1f7ba5988db2493852f5", result.hash);
}

#[test]
fn keys() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfConvertPublicKeyToTonSafeFormat = client.request("crypto.convert_public_key_to_ton_safe_format", ParamsOfConvertPublicKeyToTonSafeFormat {
        public_key: "06117f59ade83e097e0fb33e5d29e8735bda82b3bf78a015542aaa853bb69600".into(),
    });
    assert_eq!("PuYGEX9Zreg-CX4Psz5dKehzW9qCs794oBVUKqqFO7aWAOTD", result.ton_public_key);

    let result: KeyPair = client.request_no_params("crypto.generate_random_sign_keys");
    assert_eq!(result.public.len(), 64);
    assert_eq!(result.secret.len(), 64);
    assert_ne!(result.secret, result.public);

    let result: ResultOfSign = client.request("crypto.sign", ParamsOfSign {
        unsigned: base64::encode("Test Message"),
        keys: KeyPair {
            public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
            secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f".into(),
        },
    });
    assert_eq!(result.signed, "+wz+QO6l1slgZS5s65BNqKcu4vz24FCJz4NSAxef9lu0jFfs8x3PzSZRC+pn5k8+aJi3xYMA3BQzglQmjK3hA1Rlc3QgTWVzc2FnZQ==");
    assert_eq!(result.signature, "fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade103");

    let result: ResultOfVerifySignature = client.request("crypto.verify_signature", ParamsOfVerifySignature {
        signed: base64_from_hex("fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade10354657374204d657373616765"),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    });
    assert_eq!(text_from_base64(&result.unsigned), "Test Message");
}

#[test]
fn scrypt() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfScrypt = client.request("crypto.scrypt", ParamsOfScrypt {
        password: base64::encode("Test Password"),
        salt: base64::encode("Test Salt"),
        log_n: 10,
        r: 8,
        p: 16,
        dk_len: 64,
    });
    assert_eq!(result.key, "52e7fcf91356eca55fc5d52f16f5d777e3521f54e3c570c9bbb7df58fc15add73994e5db42be368de7ebed93c9d4f21f9be7cc453358d734b04a057d0ed3626d");
}

#[test]
fn nacl() {
    TestClient::init_log();
    let client = TestClient::new();

    // Sign

    let result: KeyPair = client.request("crypto.nacl_sign_keypair_from_secret", ParamsOfNaclSignKeyPairFromSecret {
        secret: "8fb4f2d256e57138fb310b0a6dac5bbc4bee09eb4821223a720e5b8e1f3dd674".into(),
    });
    assert_eq!(result.public, "aa5533618573860a7e1bf19f34bd292871710ed5b2eafa0dcdbb33405f2231c6");

    let result: ResultOfNaclSign = client.request("crypto.nacl_sign", ParamsOfNaclSign {
        unsigned: base64::encode("Test Message"),
        secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    });
    assert_eq!(result.signed, "+wz+QO6l1slgZS5s65BNqKcu4vz24FCJz4NSAxef9lu0jFfs8x3PzSZRC+pn5k8+aJi3xYMA3BQzglQmjK3hA1Rlc3QgTWVzc2FnZQ==");

    let result: ResultOfNaclSignOpen = client.request("crypto.nacl_sign_open", ParamsOfNaclSignOpen {
        signed: base64_from_hex("fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade10354657374204d657373616765"),
        public: "1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    });
    assert_eq!(text_from_base64(&result.unsigned), "Test Message");

    let result: ResultOfNaclSignDetached = client.request("crypto.nacl_sign_detached", ParamsOfNaclSign {
        unsigned: base64::encode("Test Message"),
        secret: "56b6a77093d6fdf14e593f36275d872d75de5b341942376b2a08759f3cbae78f1869b7ef29d58026217e9cf163cbfbd0de889bdf1bf4daebf5433a312f5b8d6e".into(),
    });
    assert_eq!(result.signature, "fb0cfe40eea5d6c960652e6ceb904da8a72ee2fcf6e05089cf835203179ff65bb48c57ecf31dcfcd26510bea67e64f3e6898b7c58300dc14338254268cade103");

    // Box

    let result: KeyPair = client.request_no_params("crypto.nacl_box_keypair");
    assert_eq!(result.public.len(), 64);
    assert_eq!(result.secret.len(), 64);
    assert_ne!(result.public, result.secret);

    let result: KeyPair = client.request("crypto.nacl_box_keypair_from_secret", ParamsOfNaclBoxKeyPairFromSecret {
        secret: "e207b5966fb2c5be1b71ed94ea813202706ab84253bdf4dc55232f82a1caf0d4".into(),
    });
    assert_eq!(result.public, "a53b003d3ffc1e159355cb37332d67fc235a7feb6381e36c803274074dc3933a");

    let result: ResultOfNaclBox = client.request("crypto.nacl_box", ParamsOfNaclBox {
        decrypted: base64::encode("Test Message"),
        nonce: "cd7f99924bf422544046e83595dd5803f17536f5c9a11746".into(),
        their_public: "c4e2d9fe6a6baf8d1812b799856ef2a306291be7a7024837ad33a8530db79c6b".into(),
        secret: "d9b9dc5033fb416134e5d2107fdbacab5aadb297cb82dbdcd137d663bac59f7f".into(),
    });
    assert_eq!(result.encrypted, "li4XED4kx/pjQ2qdP0eR2d/K30uN94voNADxwA==");

    let result: ResultOfNaclBoxOpen = client.request("crypto.nacl_box_open", ParamsOfNaclBoxOpen {
        encrypted: base64_from_hex("962e17103e24c7fa63436a9d3f4791d9dfcadf4b8df78be83400f1c0"),
        nonce: "cd7f99924bf422544046e83595dd5803f17536f5c9a11746".into(),
        their_public: "c4e2d9fe6a6baf8d1812b799856ef2a306291be7a7024837ad33a8530db79c6b".into(),
        secret: "d9b9dc5033fb416134e5d2107fdbacab5aadb297cb82dbdcd137d663bac59f7f".into(),
    });
    assert_eq!(text_from_base64(&result.decrypted), "Test Message");

    // Secret box

    let result: ResultOfNaclBox = client.request("crypto.nacl_secret_box", ParamsOfNaclSecretBox {
        decrypted: base64::encode("Test Message"),
        nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
        key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
    });
    assert_eq!(result.encrypted, "JL7ejKWe2KXmrsns41yfXoQF0t/C1Q8RGyzQ2A==");

    let result: ResultOfNaclBoxOpen = client.request("crypto.nacl_secret_box_open", ParamsOfNaclSecretBoxOpen {
        encrypted: base64_from_hex("24bede8ca59ed8a5e6aec9ece35c9f5e8405d2dfc2d50f111b2cd0d8"),
        nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
        key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
    });
    assert_eq!(text_from_base64(&result.decrypted), "Test Message");

    let e: ResultOfNaclBox = client.request("crypto.nacl_secret_box", ParamsOfNaclSecretBox {
        decrypted: base64::encode("Text with \' and \" and : {}"),
        nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
        key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
    });
    let d: ResultOfNaclBoxOpen = client.request("crypto.nacl_secret_box_open", ParamsOfNaclSecretBoxOpen {
        encrypted: e.encrypted,
        nonce: "2a33564717595ebe53d91a785b9e068aba625c8453a76e45".into(),
        key: "8f68445b4e78c000fe4d6b7fc826879c1e63e3118379219a754ae66327764bd8".into(),
    });
    assert_eq!(text_from_base64(&d.decrypted), "Text with \' and \" and : {}");


}

#[test]
fn mnemonic() {
    TestClient::init_log();
    let client = TestClient::new();

    let result: ResultOfMnemonicWords = client.request("crypto.mnemonic_words", ParamsOfMnemonicWords {
        dictionary: None,
    });
    assert_eq!(result.words.split(" ").count(), 2048);


    for dictionary in 1..9 {
        for word_count in &[12u8, 15, 18, 21, 24] {
            let result: ResultOfMnemonicFromRandom = client.request("crypto.mnemonic_from_random", ParamsOfMnemonicFromRandom {
                dictionary: Some(dictionary),
                word_count: Some(*word_count),
            });
            assert_eq!(result.phrase.split(" ").count(), *word_count as usize);
        }
    }

    let result: ResultOfMnemonicFromEntropy = client.request("crypto.mnemonic_from_entropy", ParamsOfMnemonicFromEntropy {
        entropy: "00112233445566778899AABBCCDDEEFF".into(),
        dictionary: Some(1),
        word_count: Some(12),
    });
    assert_eq!(result.phrase, "abandon math mimic master filter design carbon crystal rookie group knife young");

    for dictionary in 1..9 {
        for word_count in &[12u8, 15, 18, 21, 24] {
            let result: ResultOfMnemonicFromRandom = client.request("crypto.mnemonic_from_random", ParamsOfMnemonicFromRandom {
                dictionary: Some(dictionary),
                word_count: Some(*word_count),
            });
            let verify_result: ResultOfMnemonicVerify = client.request("crypto.mnemonic_verify", ParamsOfMnemonicVerify {
                phrase: result.phrase,
                dictionary: Some(dictionary),
                word_count: Some(*word_count),
            });
            assert_eq!(verify_result.valid, true);
        }
    }

    let result: ResultOfMnemonicVerify = client.request("crypto.mnemonic_verify", ParamsOfMnemonicVerify {
        phrase: "one two".into(),
        dictionary: None,
        word_count: None,
    });
    assert_eq!(result.valid, false);

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle".into(),
        path: None,
        dictionary: Some(0),
        word_count: Some(24),
    });
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client.request("crypto.convert_public_key_to_ton_safe_format", ParamsOfConvertPublicKeyToTonSafeFormat {
        public_key: result.public,
    });
    assert_eq!(result.ton_public_key, "PuYTvCuf__YXhp-4jv3TXTHL0iK65ImwxG0RGrYc1sP3H4KS");

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle".into(),
        path: Some("m".into()),
        dictionary: Some(0),
        word_count: Some(24),
    });
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client.request("crypto.convert_public_key_to_ton_safe_format", ParamsOfConvertPublicKeyToTonSafeFormat {
        public_key: result.public,
    });
    assert_eq!(result.ton_public_key, "PubDdJkMyss2qHywFuVP1vzww0TpsLxnRNnbifTCcu-XEgW0");

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: "abandon math mimic master filter design carbon crystal rookie group knife young".into(),
        path: None,
        dictionary: None,
        word_count: None,
    });
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client.request("crypto.convert_public_key_to_ton_safe_format", ParamsOfConvertPublicKeyToTonSafeFormat {
        public_key: result.public,
    });
    assert_eq!(result.ton_public_key, "PuZhw8W5ejPJwKA68RL7sn4_RNmeH4BIU_mEK7em5d4_-cIx");

    let result: ResultOfMnemonicFromRandom = client.request("crypto.mnemonic_from_random", ParamsOfMnemonicFromRandom {
        dictionary: None,
        word_count: None,
    });
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromRandom = client.request("crypto.mnemonic_from_random", ParamsOfMnemonicFromRandom {
        dictionary: Some(0),
        word_count: Some(12),
    });
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromRandom = client.request("crypto.mnemonic_from_random", ParamsOfMnemonicFromRandom {
        dictionary: Some(1),
        word_count: Some(12),
    });
    assert_eq!(result.phrase.split(" ").count(), 12);

    let result: ResultOfMnemonicFromEntropy = client.request("crypto.mnemonic_from_entropy", ParamsOfMnemonicFromEntropy {
        entropy: "2199ebe996f14d9e4e2595113ad1e627".into(),
        dictionary: None,
        word_count: None,
    });

    let result: KeyPair = client.request("crypto.mnemonic_derive_sign_keys", ParamsOfMnemonicDeriveSignKeys {
        phrase: result.phrase,
        path: None,
        dictionary: None,
        word_count: None,
    });
    let result: ResultOfConvertPublicKeyToTonSafeFormat = client.request("crypto.convert_public_key_to_ton_safe_format", ParamsOfConvertPublicKeyToTonSafeFormat {
        public_key: result.public,
    });
    assert_eq!(result.ton_public_key, "PuZdw_KyXIzo8IksTrERN3_WoAoYTyK7OvM-yaLk711sUIB3");
}

#[test]
fn hdkey() {
    TestClient::init_log();
    let client = TestClient::new();

    let master: ResultOfHDKeyXPrvFromMnemonic = client.request("crypto.hdkey_xprv_from_mnemonic", ParamsOfHDKeyXPrvFromMnemonic {
        phrase: "abuse boss fly battle rubber wasp afraid hamster guide essence vibrant tattoo".into(),
    });
    assert_eq!(master.xprv, "xprv9s21ZrQH143K25JhKqEwvJW7QAiVvkmi4WRenBZanA6kxHKtKAQQKwZG65kCyW5jWJ8NY9e3GkRoistUjjcpHNsGBUv94istDPXvqGNuWpC");

    let result: ResultOfHDKeySecretFromXPrv = client.request("crypto.hdkey_secret_from_xprv", ParamsOfHDKeySecretFromXPrv {
        xprv: master.xprv.clone(),
    });
    assert_eq!(result.secret, "0c91e53128fa4d67589d63a6c44049c1068ec28a63069a55ca3de30c57f8b365");

    let result: ResultOfHDKeyPublicFromXPrv = client.request("crypto.hdkey_public_from_xprv", ParamsOfHDKeyPublicFromXPrv {
        xprv: master.xprv.clone(),
    });
    assert_eq!(result.public, "02a8eb63085f73c33fa31b4d1134259406347284f8dab6fc68f4bf8c96f6c39b75");

    let child: ResultOfHDKeyDeriveFromXPrv = client.request("crypto.hdkey_derive_from_xprv", ParamsOfHDKeyDeriveFromXPrv {
        xprv: master.xprv.clone(),
        child_index: 0,
        hardened: false,
    });
    assert_eq!(child.xprv, "xprv9uZwtSeoKf1swgAkVVCEUmC2at6t7MCJoHnBbn1MWJZyxQ4cySkVXPyNh7zjf9VjsP4vEHDDD2a6R35cHubg4WpzXRzniYiy8aJh1gNnBKv");

    let result: ResultOfHDKeySecretFromXPrv = client.request("crypto.hdkey_secret_from_xprv", ParamsOfHDKeySecretFromXPrv {
        xprv: child.xprv.clone(),
    });
    assert_eq!(result.secret, "518afc6489b61d4b738ee9ad9092815fa014ffa6e9a280fa17f84d95f31adb91");

    let result: ResultOfHDKeyPublicFromXPrv = client.request("crypto.hdkey_public_from_xprv", ParamsOfHDKeyPublicFromXPrv {
        xprv: child.xprv.clone(),
    });
    assert_eq!(result.public, "027a598c7572dbb4fbb9663a0c805576babf7faa173a4288a48a52f6f427e12be1");


    let second: ResultOfHDKeyDeriveFromXPrvPath = client.request("crypto.hdkey_derive_from_xprv_path", ParamsOfHDKeyDeriveFromXPrvPath {
        xprv: master.xprv.clone(),
        path: "m/44'/60'/0'/0'".into(),
    });
    assert_eq!(second.xprv, "xprvA1KNMo63UcGjmDF1bX39Cw2BXGUwrwMjeD5qvQ3tA3qS3mZQkGtpf4DHq8FDLKAvAjXsYGLHDP2dVzLu9ycta8PXLuSYib2T3vzLf3brVgZ");

    let result: ResultOfHDKeySecretFromXPrv = client.request("crypto.hdkey_secret_from_xprv", ParamsOfHDKeySecretFromXPrv {
        xprv: second.xprv.clone(),
    });
    assert_eq!(result.secret, "1c566ade41169763b155761406d3cef08b29b31cf8014f51be08c0cb4e67c5e1");

    let result: ResultOfHDKeyPublicFromXPrv = client.request("crypto.hdkey_public_from_xprv", ParamsOfHDKeyPublicFromXPrv {
        xprv: second.xprv.clone(),
    });
    assert_eq!(result.public, "02a87d9764eedaacee45b0f777b5a242939b05fa06873bf511ca9a59cb46a5f526");
}
