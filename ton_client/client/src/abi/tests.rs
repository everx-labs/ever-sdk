use crate::abi::decode_message::{DecodedMessageBody, MessageBodyType, ParamsOfDecodeMessage};
use crate::abi::encode_message::{
    CallSet, DeploySet, ParamsOfAttachSignature, ParamsOfEncodeMessage, ResultOfAttachSignature,
    ResultOfEncodeMessage,
};
use crate::abi::{FunctionHeader, Signer, ParamsOfDecodeMessageBody};
use crate::crypto::KeyPair;
use crate::tests::{TestClient, EVENTS};
use crate::utils::conversion::abi_uint;

#[test]
fn encode_v2() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };
    let abi = events_abi.clone();
    let time: u64 = 1599458364291;
    let expire: u32 = 1599458404;

    let msg: ParamsOfEncodeMessage = serde_json::from_str(r#"{
        "abi": { "type": "Handle", "value": 0 },
        "signer": {
            "type": "Keys",
            "keys": {
                "public": "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499",
                "secret": "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7"
            }
        }
    }
    "#).unwrap();

    assert_eq!(msg.signer, Signer::Keys { keys: keys.clone()});

    let deploy_params = |signing: Signer| ParamsOfEncodeMessage {
        abi: abi.clone(),
        address: None,
        deploy_set: Some(DeploySet {
            workchain_id: None,
            tvc: events_tvc.clone(),
            initial_data: None,
        }),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                pubkey: signing.resolve_public_key().unwrap(),
                time: Some(time),
                expire: Some(expire),
            }),
            input: None,
        }),
        signer: signing,
        processing_try_index: None,
    };

    let unsigned: ResultOfEncodeMessage = client.request(
        "abi.encode_message",
        deploy_params(Signer::External {
            pubkey: keys.public.clone(),
        }),
    ).unwrap();
    assert_eq!(unsigned.message, "te6ccgECFwEAA2gAAqeIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEZTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gGAQEBwAICA88gBQMBAd4EAAPQIABB2mPiBH+O713GsgL3S844tQp+62YECSCD0w6eEqy4TKTMAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=");
    assert_eq!(
        unsigned.data_to_sign.as_ref().unwrap(),
        "KCGM36iTYuCYynk+Jnemis+mcwi3RFCke95i7l96s4Q="
    );
    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    assert_eq!(signature, "6272357bccb601db2b821cb0f5f564ab519212d242cf31961fe9a3c50a30b236012618296b4f769355c0e9567cd25b366f3c037435c498c82e5305622adbc70e");
    let signed: ResultOfAttachSignature = client.request(
        "abi.attach_signature",
        ParamsOfAttachSignature {
            abi: abi.clone(),
            pubkey: keys.public.clone(),
            message: unsigned.message,
            signature,
        },
    ).unwrap();
    assert_eq!(signed.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    let signed: ResultOfEncodeMessage = client.request(
        "abi.encode_message",
        deploy_params(Signer::Keys { keys: keys.clone() }),
    ).unwrap();
    assert_eq!(signed.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    let address = "0:05beb555e942fa744fd96f45a9ea9d0a8248208ca12421947c06e59bc997d309";
    let run_params = |signing: Signer| ParamsOfEncodeMessage {
        address: Some(address.into()),
        abi: abi.clone(),
        deploy_set: None,
        call_set: Some(CallSet {
            function_name: "returnValue".into(),
            header: Some(FunctionHeader {
                pubkey: None,
                time: Some(time),
                expire: Some(expire),
            }),
            input: Some(json!({
                "id": "0"
            })),
        }),
        signer: signing,
        processing_try_index: None,
    };

    let unsigned: ResultOfEncodeMessage = client.request(
        "abi.encode_message",
        run_params(Signer::External {
            pubkey: keys.public.clone(),
        }),
    ).unwrap();
    assert_eq!(unsigned.message, "te6ccgEBAgEAeAABpYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIFMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    assert_eq!(
        unsigned.data_to_sign.as_ref().unwrap(),
        "i4Hs3PB12QA9UBFbOIpkG3JerHHqjm4LgvF4MA7TDsY="
    );
    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    assert_eq!(signature, "5bbfb7f184f2cb5f019400b9cd497eeaa41f3d5885619e9f7d4fab8dd695f4b3a02159a1422996c1dd7d1be67898bc79c6adba6c65a18101ac5f0a2a2bb8910b");
    let signed: ResultOfAttachSignature = client.request(
        "abi.attach_signature",
        ParamsOfAttachSignature {
            abi: abi.clone(),
            pubkey: keys.public.clone(),
            message: unsigned.message,
            signature,
        },
    ).unwrap();
    assert_eq!(signed.message, "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==");

    let signed: ResultOfEncodeMessage = client.request(
        "abi.encode_message",
        run_params(Signer::Keys { keys: keys.clone() }),
    ).unwrap();
    assert_eq!(signed.message, "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==");

    let no_pubkey: ResultOfEncodeMessage = client.request(
        "abi.encode_message",
        run_params(Signer::None),
    ).unwrap();
    assert_eq!(no_pubkey.message, "te6ccgEBAQEAVQAApYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIAAAAC6M5Llwa+q5jIK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB");
}

#[test]
fn decode_v2() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, _events_tvc) = TestClient::package(EVENTS, Some(2));

    let decode_events = |message: &str| {
        let result: DecodedMessageBody = client.request(
            "abi.decode_message",
            ParamsOfDecodeMessage {
                abi: events_abi.clone(),
                message: message.into(),
            },
        ).unwrap();
        result
    };
    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Input,
        name: "returnValue".into(),
        value: Some(json!({
            "id": abi_uint(0, 256),
        })),
        header: Some(FunctionHeader {
            expire: Some(1599458404),
            time: Some(1599458364291),
            pubkey: Some("4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into()),
        }),
    };
    assert_eq!(expected, decode_events("te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="));

    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Event,
        name: "EventThrown".into(),
        value: Some(json!({
            "id": abi_uint(0, 256)
        })),
        header: None,
    };
    assert_eq!(expected, decode_events("te6ccgEBAQEAVQAApeACvg5/pmQpY4m61HmJ0ne+zjHJu3MNG8rJxUDLbHKBu/AAAAAAAAAMJL6z6ro48sYvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABA"));

    let result: DecodedMessageBody = client.request("abi.decode_message_body", ParamsOfDecodeMessageBody {
        abi: events_abi.clone(),
        body: "te6ccgEBAgEAlgAB4a3f2/jCeWWvgMoAXOakv3VSD56sQrDPT76n1cbrSvpZ0BCs0KEUy2Duvo3zPExePONW3TYy0MCA1i+FFRXcSIXTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGQVviwSgAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        is_internal: false,
    }).unwrap();
    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Input,
        name: "returnValue".into(),
        value: Some(json!({
            "id": abi_uint(0, 256)
        })),
        header: Some(FunctionHeader {
            expire: Some(1599458404),
            time: Some(1599458364291),
            pubkey: Some("4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into()),
        }),
    };
    assert_eq!(expected, result);

    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Output,
        name: "returnValue".into(),
        value: Some(json!({
            "value0": abi_uint(0, 256)
        })),
        header: None,
    };
    assert_eq!(expected, decode_events("te6ccgEBAQEAVQAApeACvg5/pmQpY4m61HmJ0ne+zjHJu3MNG8rJxUDLbHKBu/AAAAAAAAAMKr6z6rxK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABA"));
}
