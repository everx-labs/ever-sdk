use crate::tests::{TestClient, EVENTS};
use crate::abi::encode::{
    ParamsOfDeployMessage, ResultOfEncodeMessage, ResultOfEncodeWithSignature,
    ParamsOfEncodeWithSignature,
};
use crate::abi::abi::Abi;
use crate::crypto::boxes::Signing;
use crate::crypto::keys::KeyPair;

#[test]
fn encode() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, 2);
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };

    let deploy_params = |public_key: Option<String>, signing: Option<Signing>| ParamsOfDeployMessage {
        abi: Abi::Value(events_abi.clone()),
        tvc: events_tvc.clone(),
        initial_data: None,
        function_name: None,
        public_key,
        header: Some(json!({
            "pubkey": keys.public,
            "time": 1,
            "expire": 2,
        })),
        input: None,
        signing,
        workchain_id: None,
    };


    let unsigned: ResultOfEncodeMessage = client.request(
        "abi.encode_deploy_message",
        deploy_params(Some(keys.public.clone()), None),
    );
    println!(">>> {} {:?}", unsigned.message, unsigned.data_to_sign);
    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    let signed: ResultOfEncodeWithSignature = client.request("abi.encode_with_signature", ParamsOfEncodeWithSignature {
        message: unsigned.message.clone(),
        signature,
    });
    let message: ResultOfEncodeMessage = client.request(
        "abi.encode_deploy_message",
        deploy_params(None, Some(Signing::Keys(keys.clone()))),
    );
    assert_eq!(signed.message, message.message);

    /*
        const message = await contracts.createDeployMessage(deployParams);
        expect(signed.message.messageBodyBase64)
            .toEqual(message.message.messageBodyBase64);


        const messageParams = {
            address: message.address,
            abi: eventsPackage.abi,
            functionName: 'returnValue',
            header: {
                pubkey: keys.public,
                time: Date.now(),
            },
            input: { id: '0' },
            keyPair: keys,
        };

        const unsignedRunMessage = await contracts.createUnsignedRunMessage(messageParams);
        signBytesBase64 = await crypto.naclSignDetached({
            base64: unsignedRunMessage.signParams.bytesToSignBase64,
        }, signKey.secret, TONOutputEncoding.Base64);
        const signedRunMessage = await contracts.createSignedRunMessage({
            unsignedMessage: unsignedRunMessage,
            signBytesBase64,
        });
        const runMessage = await contracts.createRunMessage(messageParams);

        expect(signedRunMessage).toEqual(runMessage);
    */
}
