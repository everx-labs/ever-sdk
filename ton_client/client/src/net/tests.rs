use crate::abi::{Abi, MessageSigning, ParamsOfEncodeMessage, DeploySet, CallSet};
use crate::crypto::KeyPair;
use crate::error::ApiResult;
use crate::net::{
    CallbackParams, MessageProcessingEvent, MessageSource, ParamsOfProcessMessage,
    ResultOfProcessMessage,
};
use crate::tests::{TestClient, EVENTS};

#[tokio::test(core_threads = 2)]
async fn process_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };
    let abi = Abi::Value(events_abi.clone());

    let events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |event: ApiResult<MessageProcessingEvent>| {
        if let Ok(event) = event {
            events_copy.lock().unwrap().push(event);
        }
    };

    let callback_id = client.register_callback(callback);

    let result: ResultOfProcessMessage = client.request_future(
        "net.process_message",
        ParamsOfProcessMessage {
            message: MessageSource::EncodingParams(ParamsOfEncodeMessage {
                abi: abi.clone(),
                address: None,
                deploy_set: Some(DeploySet {
                    workchain_id: None,
                    tvc: events_tvc.clone(),
                    initial_data: None,
                }),
                call_set: Some(CallSet {
                    function_name: "constructor".into(),
                    header: Some(json!({
                        "pubkey": keys.public.clone(),
                    })),
                    input: None,
                }),
                signing: MessageSigning::Keys(keys),
            }),
            context: None,
            callback: Some(CallbackParams {
                id: callback_id,
                unregister: true,
            }),
        },
    ).await;

    println!("{:?}", result.transaction);
    // give some time for subscription to receive all data
    // std::thread::sleep(std::time::Duration::from_millis(5000));

    client.unregister_callback(callback_id);
    let events = events.lock().unwrap().clone();
    println!("{:?}", &events);
    assert_eq!(events.len(), 2);
}
