use crate::abi::{
    encode_message, encode_message_method, Abi, CallSet, DeploySet, ParamsOfEncodeMessage,
    Signer,
};
use crate::error::ApiResult;
use crate::processing::{
    send_message, send_message_method, CallbackParams, ParamsOfSendMessage, ProcessingEvent,
};
use crate::tests::{TestClient, EVENTS};

#[tokio::test(core_threads = 2)]
async fn test_send_and_wait_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = client.generate_sign_keys();
    let abi = Abi::Serialized(events_abi.clone());

    let events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |event: ApiResult<ProcessingEvent>| {
        if let Ok(event) = event {
            events_copy.lock().unwrap().push(event);
        }
    };

    let callback_id = client.register_callback(callback);

    let encode_message = client.wrap_async(encode_message, encode_message_method);
    let send_message = client.wrap_async(send_message, send_message_method);

    let encoded = encode_message
        .call(ParamsOfEncodeMessage {
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
            signer: Signer::WithKeys(keys.clone()),
        })
        .await;

    client
        .get_grams_from_giver_async(&encoded.address, None)
        .await;

    let _result = send_message
        .call(ParamsOfSendMessage {
            message: encoded.message,
            message_expiration_time: None,
            callback: Some(CallbackParams::with_id(callback_id)),
        })
        .await;

    client.unregister_callback(callback_id);
    let events = events.lock().unwrap().clone();
    println!("{:?}", &events);
    assert_eq!(events.len(), 3);
    assert!(match events[0] {
        ProcessingEvent::WillFetchFirstBlock {} => true,
        _ => false,
    });
    assert!(match events[1] {
        ProcessingEvent::WillSend { .. } => true,
        _ => false,
    });
    assert!(match events[2] {
        ProcessingEvent::DidSend {..} => true,
        _ => false,
    });
}

