use crate::tests::{TestClient, EVENTS};
use crate::abi::encode::{
    ParamsOfDeployMessage, ResultOfEncodeMessage, ResultOfEncodeWithSignature,
    ParamsOfEncodeWithSignature,
};
use crate::abi::abi::Abi;
use serde_json::{Value, Map};
use crate::crypto::boxes::Signing;

#[test]
fn encode() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, 2);
    let keys = client.generate_sign_keys();

    let deploy_params = |signing: Option<Signing>| ParamsOfDeployMessage {
        abi: Abi::Value(events_abi.clone()),
        tvc: events_tvc.clone(),
        data: None,
        function_name: None,
        header: Some(json!({
                "pubkey": keys.public,
                "time": 1,
                "expire": 2,
            })),
        input: Value::Object(Map::new()),
        signing,
        workchain_id: None,
    };


    let unsigned: ResultOfEncodeMessage = client.request(
        "abi.encode_deploy_message",
        deploy_params(None),
    );
    let signature = client.sign_detached(&unsigned.bytes_to_sign.unwrap(), &keys);
    let signed: ResultOfEncodeWithSignature = client.request("abi.encode_with_signature", ParamsOfEncodeWithSignature {
        message: unsigned.message.clone(),
        signature,
    });
    let message: ResultOfEncodeMessage = client.request(
        "abi.encode_deploy_message",
        deploy_params(Some(Signing::Keys(keys.clone()))),
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
