use crate::abi::decode_message::DataLayout;
use crate::abi::encode_message::{
    CallSet, DeploySet, ParamsOfAttachSignature, ParamsOfEncodeInternalMessage,
    ParamsOfEncodeMessage, ResultOfAttachSignature, ResultOfEncodeInternalMessage,
    ResultOfEncodeMessage,
};
use crate::abi::internal::{create_tvc_image, is_empty_pubkey, resolve_pubkey};
use crate::abi::{FunctionHeader, ParamsOfDecodeMessageBody, Signer};
use crate::boc::internal::{
    deserialize_object_from_base64, deserialize_object_from_cell, get_boc_hash,
    serialize_cell_to_base64, serialize_object_to_base64,
};
use crate::boc::{
    parse_message, ParamsOfDecodeStateInit, ParamsOfGetCodeFromTvc, ParamsOfParse,
    ResultOfDecodeStateInit, ResultOfGetCodeFromTvc,
};
use crate::crypto::KeyPair;
use crate::encoding::account_decode;
use crate::tests::{TestClient, EVENTS, EVENTS_OLD, HELLO, T24_INIT_DATA};
use crate::utils::conversion::abi_uint;
use crate::{
    abi::decode_message::{DecodedMessageBody, MessageBodyType, ParamsOfDecodeMessage},
    boc::ResultOfParse,
};
use std::future::Future;

use crate::boc::tvc::resolve_state_init_cell;
use crate::boc::tvc_serialization::{Metadata, SmallStr, TvcFrst, TvmSmc, Version, TVC};
use serde_json::Value;
use std::io::Cursor;
use ton_abi::Contract;
use ton_block::{
    CurrencyCollection, Deserializable, InternalMessageHeader, Message, Serializable, StateInit,
};
use ton_sdk::ContractImage;
use ton_types::{BuilderData, IBitstring, Result};

use super::*;

struct EncodeCheckEthalons {
    pub deploy_unsigned_message: &'static str,
    pub deploy_unsigned_data_to_sign: &'static str,
    pub deploy_signature: &'static str,
    pub deploy_signed_message: &'static str,
    pub deploy_without_sign_message: &'static str,
    pub run_unsigned_message: &'static str,
    pub run_unsigned_data_to_sign: &'static str,
    pub run_signature: &'static str,
    pub run_signed_message: &'static str,
    pub run_without_sign_message: &'static str,
}

#[test]
fn encode_v2() {
    let ethalons = EncodeCheckEthalons {
        deploy_unsigned_message: "te6ccgECFwEAA2gAAqeIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEZTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gGAQEBwAICA88gBQMBAd4EAAPQIABB2mPiBH+O713GsgL3S844tQp+62YECSCD0w6eEqy4TKTMAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=",
        deploy_unsigned_data_to_sign: "KCGM36iTYuCYynk+Jnemis+mcwi3RFCke95i7l96s4Q=",
        deploy_signature: "6272357bccb601db2b821cb0f5f564ab519212d242cf31961fe9a3c50a30b236012618296b4f769355c0e9567cd25b366f3c037435c498c82e5305622adbc70e",
        deploy_signed_message: "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==",
        deploy_without_sign_message: "te6ccgECFwEAA2gAAqeIAQlSohYE8AjiqWNwltuoi4JpOxqFrrRd2cD25VrcnUJsEYpj4gR/ju9dxrIC90vOOLUKfutmBAkgg9MOnhKsuEykyAAAC6M5Llwa+q5jI0Wq+fwGAQEBwAICA88gBQMBAd4EAAPQIABB2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=",
        run_unsigned_message: "te6ccgEBAgEAeAABpYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIFMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        run_unsigned_data_to_sign: "i4Hs3PB12QA9UBFbOIpkG3JerHHqjm4LgvF4MA7TDsY=",
        run_signature: "5bbfb7f184f2cb5f019400b9cd497eeaa41f3d5885619e9f7d4fab8dd695f4b3a02159a1422996c1dd7d1be67898bc79c6adba6c65a18101ac5f0a2a2bb8910b",
        run_signed_message: "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        run_without_sign_message: "te6ccgEBAQEAVQAApYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIAAAAC6M5Llwa+q5jIK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB",
    };

    test_encode_v2_params(ethalons, None);
}

#[test]
fn encode_v2_with_signature_id() {
    let ethalons = EncodeCheckEthalons {
        deploy_unsigned_message: "te6ccgECFwEAA2gAAqeIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEZTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gGAQEBwAICA88gBQMBAd4EAAPQIABB2mPiBH+O713GsgL3S844tQp+62YECSCD0w6eEqy4TKTMAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=",
        deploy_unsigned_data_to_sign: "AAAAeyghjN+ok2LgmMp5PiZ3porPpnMIt0RQpHveYu5ferOE",
        deploy_signature: "5ced0e939e98198a5a649be3863367af3aa679f26d80a8a2d01fe292de5498175867b1dbecb32b1c175e0b4701fa17a094e6ad8f74328fb6d5695f034d66f70c",
        deploy_signed_message: "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4a52h0nPTAzFLTJN8cMZs9edUzz5NsBUUWgP8UlvKkwLrDPY7fZZlY4LrwWjgP0L0EpzVse6GUfbarSvgaaze4ZTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==",
        deploy_without_sign_message: "te6ccgECFwEAA2gAAqeIAQlSohYE8AjiqWNwltuoi4JpOxqFrrRd2cD25VrcnUJsEYpj4gR/ju9dxrIC90vOOLUKfutmBAkgg9MOnhKsuEykyAAAC6M5Llwa+q5jI0Wq+fwGAQEBwAICA88gBQMBAd4EAAPQIABB2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=",
        run_unsigned_message: "te6ccgEBAgEAeAABpYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIFMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        run_unsigned_data_to_sign: "AAAAe4uB7NzwddkAPVARWziKZBtyXqxx6o5uC4LxeDAO0w7G",
        run_signature: "e0a552ac1263c3fea301157f7d2f1f0f9e75259db1cdc6439c7458000618f6f8748a77a076bd6689446522eed92e95ca9a139c6134cb62d4f2ff786b9f52b106",
        run_signed_message: "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHh8FKpVgkx4f9RgIq/vpePh886ks7Y5uMhzjosAAMMe3w6RTvQO16zRKIykXdsl0rlTQnOMJplsWp5f7w1z6lYg1MfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        run_without_sign_message: "te6ccgEBAQEAVQAApYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIAAAAC6M5Llwa+q5jIK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB",
    };

    test_encode_v2_params(ethalons, Some(123));
}

fn test_encode_v2_params(ethalons: EncodeCheckEthalons, signature_id: Option<i32>) {
    TestClient::init_log();
    let client = TestClient::new_with_config(serde_json::json!({
        "network": {
            "signature_id": signature_id
        }
    }));
    let (events_abi, events_tvc) = TestClient::package(EVENTS_OLD, Some(2));
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };
    let abi = events_abi.clone();
    let time: u64 = 1599458364291;
    let expire: u32 = 1599458404;

    let signing_box: crate::crypto::RegisteredSigningBox = client
        .request("crypto.get_signing_box", keys.clone())
        .unwrap();

    let msg: ParamsOfEncodeMessage = serde_json::from_str(
        r#"{
        "abi": { "type": "Handle", "value": 0 },
        "signer": {
            "type": "Keys",
            "keys": {
                "public": "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499",
                "secret": "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7"
            }
        }
    }
    "#,
    )
    .unwrap();

    assert_eq!(msg.signer, Signer::Keys { keys: keys.clone() });

    // check deploy params

    let deploy_params = |signing: Signer| ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: Some(DeploySet {
            tvc: events_tvc.clone(),
            ..Default::default()
        }),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                pubkey: Some(keys.public.clone()),
                time: Some(time),
                expire: Some(expire),
            }),
            input: None,
        }),
        signer: signing,
        ..Default::default()
    };

    let unsigned: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::External {
                public_key: keys.public.clone(),
            }),
        )
        .unwrap();
    assert_eq!(unsigned.message, ethalons.deploy_unsigned_message);
    assert_eq!(
        unsigned.data_to_sign.as_ref().unwrap(),
        ethalons.deploy_unsigned_data_to_sign
    );
    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    assert_eq!(signature, ethalons.deploy_signature);
    let signed: ResultOfAttachSignature = client
        .request(
            "abi.attach_signature",
            ParamsOfAttachSignature {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned.message,
                signature,
            },
        )
        .unwrap();
    assert_eq!(signed.message, ethalons.deploy_signed_message);

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::Keys { keys: keys.clone() }),
        )
        .unwrap();
    assert_eq!(signed.message, ethalons.deploy_signed_message);

    let signed_with_box: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::SigningBox {
                handle: signing_box.handle.clone(),
            }),
        )
        .unwrap();
    assert_eq!(signed_with_box.message, ethalons.deploy_signed_message);

    let without_sign: ResultOfEncodeMessage = client
        .request("abi.encode_message", deploy_params(Signer::None))
        .unwrap();
    assert_eq!(without_sign.message, ethalons.deploy_without_sign_message);

    // check run params

    let address = "0:05beb555e942fa744fd96f45a9ea9d0a8248208ca12421947c06e59bc997d309";
    let run_params = |signing: Signer| ParamsOfEncodeMessage {
        address: Some(address.into()),
        abi: abi.clone(),
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
        ..Default::default()
    };
    let body_params = |run_params: ParamsOfEncodeMessage| ParamsOfEncodeMessageBody {
        abi: run_params.abi,
        call_set: run_params.call_set.unwrap(),
        is_internal: false,
        processing_try_index: run_params.processing_try_index,
        signer: run_params.signer,
        address: Some(address.into()),
        ..Default::default()
    };
    let extract_body = |message| {
        let unsigned_parsed: ResultOfParse = client
            .request("boc.parse_message", ParamsOfParse { boc: message })
            .unwrap();
        unsigned_parsed.parsed["body"].as_str().unwrap().to_owned()
    };

    // encoding unsigned and attaching the signature

    let unsigned: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::External {
                public_key: keys.public.clone(),
            }),
        )
        .unwrap();
    assert_eq!(unsigned.message, ethalons.run_unsigned_message);
    assert_eq!(
        unsigned.data_to_sign.clone().unwrap(),
        ethalons.run_unsigned_data_to_sign
    );

    let unsigned_body = extract_body(unsigned.message.clone());

    let unsigned_body_encoded: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::External {
                public_key: keys.public.clone(),
            })),
        )
        .unwrap();
    assert_eq!(unsigned_body_encoded.body, unsigned_body);
    assert_eq!(unsigned_body_encoded.data_to_sign, unsigned.data_to_sign);

    let signature = client.sign_detached(unsigned.data_to_sign.as_deref().unwrap(), &keys);
    assert_eq!(signature, ethalons.run_signature);
    let signed: ResultOfAttachSignature = client
        .request(
            "abi.attach_signature",
            ParamsOfAttachSignature {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned.message,
                signature: signature.clone(),
            },
        )
        .unwrap();
    assert_eq!(signed.message, ethalons.run_signed_message);

    let sign: ResultOfGetSignatureData = client
        .request(
            "abi.get_signature_data",
            ParamsOfGetSignatureData {
                abi: abi.clone(),
                message: signed.message.clone(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(sign.signature, ethalons.run_signature);
    assert_eq!(sign.unsigned, unsigned.data_to_sign.as_deref().unwrap());

    let signed_body = extract_body(signed.message);

    let signed: ResultOfAttachSignatureToMessageBody = client
        .request(
            "abi.attach_signature_to_message_body",
            ParamsOfAttachSignatureToMessageBody {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned_body_encoded.body,
                signature,
            },
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    // encoding signed

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::Keys { keys: keys.clone() }),
        )
        .unwrap();
    assert_eq!(signed.message, ethalons.run_signed_message);

    let signed: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::Keys { keys: keys.clone() })),
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::SigningBox {
                handle: signing_box.handle.clone(),
            }),
        )
        .unwrap();
    assert_eq!(signed.message, ethalons.run_signed_message);

    let signed: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::SigningBox {
                handle: signing_box.handle.clone(),
            })),
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    let no_pubkey: ResultOfEncodeMessage = client
        .request("abi.encode_message", run_params(Signer::None))
        .unwrap();
    assert_eq!(no_pubkey.message, ethalons.run_without_sign_message);

    let no_pubkey_body: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::None)),
        )
        .unwrap();
    assert_eq!(no_pubkey_body.body, extract_body(no_pubkey.message));
}

#[test]
fn decode_v2() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, _events_tvc) = TestClient::package(EVENTS_OLD, Some(2));

    let decode_events = |message: &str| {
        let result: DecodedMessageBody = client
            .request(
                "abi.decode_message",
                ParamsOfDecodeMessage {
                    abi: events_abi.clone(),
                    message: message.into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let parsed: ResultOfParse = client
            .request(
                "boc.parse_message",
                ParamsOfParse {
                    boc: message.into(),
                },
            )
            .unwrap();
        let body = parsed.parsed["body"].as_str().unwrap().to_owned();
        let result_body: DecodedMessageBody = client
            .request(
                "abi.decode_message_body",
                ParamsOfDecodeMessageBody {
                    abi: events_abi.clone(),
                    body,
                    is_internal: parsed.parsed["msg_type_name"] == "Internal",
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(result, result_body);
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
        ..Default::default()
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

#[test]
fn test_is_empty_pubkey() -> Result<()> {
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;

    assert!(is_empty_pubkey(&pubkey));

    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    assert!(!is_empty_pubkey(&pubkey));

    let mut array = [0; 32];
    array[0] = 1;
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&array)?;
    assert!(!is_empty_pubkey(&pubkey));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_resolve_pubkey() -> Result<()> {
    let context = crate::ClientContext::new(crate::ClientConfig::default()).unwrap();
    let tvc = base64::encode(include_bytes!("../tests/contracts/abi_v2/Hello.tvc"));
    let mut deploy_set = DeploySet {
        tvc: Some(tvc.clone()),
        ..Default::default()
    };
    let mut image = create_tvc_image("", None, resolve_state_init_cell(&context, &tvc).await?)?;
    assert!(resolve_pubkey(&deploy_set, &image, &None)?.is_none());

    let external_pub_key =
        Some("1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".to_owned());
    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;
    assert_eq!(resolved, external_pub_key);

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, external_pub_key);

    let tvc_pubkey_empty = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;
    image.set_public_key(&tvc_pubkey_empty)?;

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, external_pub_key);

    let tvc_pubkey_1 = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    image.set_public_key(&tvc_pubkey_1)?;

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, Some(hex::encode(tvc_pubkey_1.as_bytes())));

    let initial_pub_key =
        Some("1234567890123456789012345678901234567890123456789012345678901234".to_owned());
    deploy_set.initial_pubkey = initial_pub_key.clone();

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, initial_pub_key);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_encode_message_pubkey() -> Result<()> {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package(HELLO, None);

    let initial_pubkey = Some(gen_pubkey());
    let tvc_pubkey = Some(gen_pubkey());
    let signer_pubkey = Some(gen_pubkey());

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &None,
        &None,
        &signer_pubkey,
        &signer_pubkey,
    )
    .await?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &None,
        &tvc_pubkey,
        &signer_pubkey,
        &tvc_pubkey,
    )
    .await?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &None,
        &signer_pubkey,
        &initial_pubkey,
    )
    .await?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &tvc_pubkey,
        &signer_pubkey,
        &initial_pubkey,
    )
    .await?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &tvc_pubkey,
        &None,
        &initial_pubkey,
    )
    .await?;

    Ok(())
}

async fn test_encode_message_pubkey_internal(
    client: &TestClient,
    abi: &Abi,
    tvc: &Option<String>,
    initial_pubkey: &Option<ed25519_dalek::PublicKey>,
    tvc_pubkey: &Option<ed25519_dalek::PublicKey>,
    signer_pubkey: &Option<ed25519_dalek::PublicKey>,
    expected_pubkey: &Option<ed25519_dalek::PublicKey>,
) -> Result<()> {
    let context = crate::ClientContext::new(crate::ClientConfig::default()).unwrap();
    let mut image = create_tvc_image(
        &abi.json_string()?,
        None,
        resolve_state_init_cell(&context, tvc.as_ref().unwrap()).await?,
    )?;
    if let Some(tvc_pubkey) = tvc_pubkey {
        image.set_public_key(tvc_pubkey)?;
    }

    let tvc = base64::encode(&image.serialize()?);

    let deploy_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: Some(DeploySet {
            tvc: Some(tvc),
            initial_pubkey: initial_pubkey.map(|key| hex::encode(key.as_bytes())),
            ..Default::default()
        }),
        signer: if let Some(key) = signer_pubkey {
            Signer::External {
                public_key: hex::encode(key.as_bytes()),
            }
        } else {
            Signer::None
        },
        call_set: CallSet::some_with_function("constructor"),
        ..Default::default()
    };

    let result: ResultOfEncodeMessage = client
        .request_async("abi.encode_message", deploy_params)
        .await?;

    let message = Message::construct_from_base64(&result.message)?;
    let state_init = message
        .state_init()
        .expect("Expected State Init")
        .write_to_bytes()?;
    let image = ContractImage::from_state_init(&mut Cursor::new(state_init))?;
    let public_key = image.get_public_key()?;

    assert_eq!(&public_key, expected_pubkey);

    Ok(())
}

fn gen_pubkey() -> ed25519_dalek::PublicKey {
    ed25519_dalek::Keypair::generate(&mut rand::thread_rng()).public
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_encode_internal_message() -> Result<()> {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package(HELLO, None);
    let contract = Contract::load(abi.json_string().unwrap().as_bytes()).unwrap();
    let func_id = contract.function("sayHello").unwrap().get_input_id();
    let context = crate::ClientContext::new(crate::ClientConfig::default()).unwrap();
    let image = create_tvc_image(
        &abi.json_string()?,
        None,
        resolve_state_init_cell(&context, tvc.as_ref().unwrap()).await?,
    )?;
    let address =
        String::from("0:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");

    test_encode_internal_message_deploy(
        &client,
        &image,
        &abi,
        &tvc,
        None,
        Some("te6ccgECHAEABG0AAmliADYO5IoxskLmUfURre2fOB04OmP32VjPwA/lDM/Cpvh8IdzWUAAAAAAAAAAAAAAAAAACMgYBAQHAAgIDzyAFAwEB3gQAA9AgAEHYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQCJv8A9KQgIsABkvSg4YrtU1gw9KEJBwEK9KQg9KEIAAACASAMCgHo/38h0wABjiaBAgDXGCD5AQFw7UTQ9AWAQPQO8orXC/8B7Ucib3XtVwMB+RDyqN7tRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4tM/Ae1HbxMhuSALAGCfMCD4I4ED6KiCCBt3QKC53pntRyFvUyDtVzCUgDTy8OIw0x8B+CO88rnTHwHxQAECASAYDQIBIBEOAQm6i1Xz+A8B+u1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t7tR28WkvIzl+1HcW9W7VfiAPgA0fgjtR/tRyBvETAByMsfydBvUe1X7UdvEsj0AO1HbxPPCz/tR28WEAAczwsA7UdvEc8Wye1UcGoCAWoVEgEJtAAa1sATAfztR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le7UdvZSBukjBw3nDtR28SgED0DvKK1wv/uvLgZPgA+kDRIMjJ+wSBA+hwgQCAyHHPCwEizwoAcc9A+CgUAI7PFiTPFiP6AnHPQHD6AnD6AoBAz0D4I88LH3LPQCDJIvsAXwUw7UdvEsj0AO1HbxPPCz/tR28WzwsA7UdvEc8Wye1UcGrbMAEJtGX2i8AWAfjtR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le0e1HbxHXCx/IghBQy+0XghCAAAAAsc8LHyHPCx/Ic88LAfgozxZyz0D4Jc8LP4Ahz0AgzzUizzG8FwB4lnHPQCHPF5Vxz0EhzeIgyXH7AFshwP+OHu1HbxLI9ADtR28Tzws/7UdvFs8LAO1HbxHPFsntVN5xatswAgEgGxkBCbtzEuRYGgD47UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3vgA0fgjtR/tRyBvETAByMsfydBvUe1X7UdvEsj0AO1HbxPPCz/tR28WzwsA7UdvEc8Wye1UcGrbMADK3XAh10kgwSCOKyDAAI4cI9Bz1yHXCwAgwAGW2zBfB9swltswXwfbMOME2ZbbMF8G2zDjBNngItMfNCB0uyCOFTAgghD/////uiCZMCCCEP////6639+W2zBfB9sw4CMh8UABXwc=")
    ).await?;

    test_encode_internal_message_deploy(
        &client,
        &image,
        &abi,
        &tvc,
        Some(CallSet {
            function_name: "constructor".into(),
            header: None,
            input: None,
        }),
        Some("te6ccgECHAEABHEAAnFiADYO5IoxskLmUfURre2fOB04OmP32VjPwA/lDM/Cpvh8IdzWUAAAAAAAAAAAAAAAAAACMaLVfP4GAQEBwAICA88gBQMBAd4EAAPQIABB2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoB6P9/IdMAAY4mgQIA1xgg+QEBcO1E0PQFgED0DvKK1wv/Ae1HIm917VcDAfkQ8qje7UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+LTPwHtR28TIbkgCwBgnzAg+COBA+iogggbd0Cgud6Z7Uchb1Mg7VcwlIA08vDiMNMfAfgjvPK50x8B8UABAgEgGA0CASARDgEJuotV8/gPAfrtR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le7UdvFpLyM5ftR3FvVu1X4gD4ANH4I7Uf7UcgbxEwAcjLH8nQb1HtV+1HbxLI9ADtR28Tzws/7UdvFhAAHM8LAO1HbxHPFsntVHBqAgFqFRIBCbQAGtbAEwH87UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3u1Hb2UgbpIwcN5w7UdvEoBA9A7yitcL/7ry4GT4APpA0SDIyfsEgQPocIEAgMhxzwsBIs8KAHHPQPgoFACOzxYkzxYj+gJxz0Bw+gJw+gKAQM9A+CPPCx9yz0AgySL7AF8FMO1HbxLI9ADtR28Tzws/7UdvFs8LAO1HbxHPFsntVHBq2zABCbRl9ovAFgH47UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3tHtR28R1wsfyIIQUMvtF4IQgAAAALHPCx8hzwsfyHPPCwH4KM8Wcs9A+CXPCz+AIc9AIM81Is8xvBcAeJZxz0AhzxeVcc9BIc3iIMlx+wBbIcD/jh7tR28SyPQA7UdvE88LP+1HbxbPCwDtR28RzxbJ7VTecWrbMAIBIBsZAQm7cxLkWBoA+O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t74ANH4I7Uf7UcgbxEwAcjLH8nQb1HtV+1HbxLI9ADtR28Tzws/7UdvFs8LAO1HbxHPFsntVHBq2zAAyt1wIddJIMEgjisgwACOHCPQc9ch1wsAIMABltswXwfbMJbbMF8H2zDjBNmW2zBfBtsw4wTZ4CLTHzQgdLsgjhUwIIIQ/////7ogmTAgghD////+ut/fltswXwfbMOAjIfFAAV8H"),
    ).await?;

    let expected_boc = "te6ccgEBAQEAOgAAcGIACRorPEhV5veJGis8SFXm94kaKzxIVeb3iRorPEhV5veh3NZQAAAAAAAAAAAAAAAAAABQy+0X";
    test_encode_internal_message_run(
        &client,
        Some(&abi),
        Some(CallSet {
            function_name: "sayHello".into(),
            header: None,
            input: None,
        }),
        None,
        Some(address.clone()),
        Some(expected_boc),
    )
    .await?;

    test_encode_internal_message_run(
        &client,
        Some(&abi),
        Some(CallSet {
            function_name: format!("0x{:x}", func_id),
            header: None,
            input: None,
        }),
        None,
        Some(address.clone()),
        Some(expected_boc),
    )
    .await?;

    test_encode_internal_message_run(
        &client,
        Some(&abi),
        Some(CallSet {
            function_name: format!("{}", func_id),
            header: None,
            input: None,
        }),
        None,
        Some(address.clone()),
        Some(expected_boc),
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_encode_internal_message_empty_body() -> Result<()> {
    let client = TestClient::new();
    let dst_address =
        String::from("0:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    let src_address =
        String::from("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");

    let mut msg_header = InternalMessageHeader::default();
    msg_header.ihr_disabled = true;
    msg_header.bounce = true;
    msg_header.value = CurrencyCollection::with_grams(1000000000);
    msg_header.set_dst(account_decode(&dst_address)?);

    let msg = Message::with_int_header(msg_header.clone());
    let expected_boc = serialize_object_to_base64(&msg, "message")?;

    test_encode_internal_message_run(
        &client,
        None,
        None,
        None,
        Some(dst_address.clone()),
        Some(&expected_boc),
    )
    .await?;

    msg_header.set_src(account_decode(&src_address)?);
    let msg = Message::with_int_header(msg_header.clone());
    let expected_boc = serialize_object_to_base64(&msg, "message")?;

    test_encode_internal_message_run(
        &client,
        None,
        None,
        Some(src_address.clone()),
        Some(dst_address.clone()),
        Some(&expected_boc),
    )
    .await
}

async fn test_encode_internal_message_run(
    client: &TestClient,
    abi: Option<&Abi>,
    call_set: Option<CallSet>,
    src: Option<String>,
    dst: Option<String>,
    expected_boc: Option<&str>,
) -> Result<()> {
    let result: ResultOfEncodeInternalMessage = client
        .request_async(
            "abi.encode_internal_message",
            ParamsOfEncodeInternalMessage {
                abi: abi.map(|x| x.clone()),
                src_address: src.clone(),
                address: dst.clone(),
                deploy_set: None,
                call_set,
                value: "1000000000".to_string(),
                bounce: Some(true),
                enable_ihr: None,
            },
        )
        .await?;

    if dst.is_some() {
        assert_eq!(&result.address, dst.as_ref().unwrap());
    }
    assert_eq!(
        result.message_id,
        get_boc_hash(&base64::decode(&result.message)?)?
    );
    if let Some(expected_boc) = expected_boc {
        assert_eq!(&result.message, expected_boc);
    }

    let parsed: ResultOfParse = client
        .request_async(
            "boc.parse_message",
            ParamsOfParse {
                boc: result.message,
            },
        )
        .await?;

    assert_eq!(parsed.parsed["msg_type_name"], "internal");
    assert_eq!(parsed.parsed["src"], src.unwrap_or("".to_owned()).as_str());
    assert_eq!(
        parsed.parsed["dst"],
        dst.unwrap_or(result.address.to_owned()).as_str()
    );
    assert_eq!(parsed.parsed["value"], "0x3b9aca00");
    assert_eq!(parsed.parsed["bounce"], true);
    assert_eq!(parsed.parsed["ihr_disabled"], true);

    Ok(())
}

async fn test_encode_internal_message_deploy(
    client: &TestClient,
    image: &ContractImage,
    abi: &Abi,
    tvc: &Option<String>,
    call_set: Option<CallSet>,
    expected_boc: Option<&str>,
) -> Result<()> {
    let result: ResultOfEncodeInternalMessage = client
        .request_async(
            "abi.encode_internal_message",
            ParamsOfEncodeInternalMessage {
                abi: Some(abi.clone()),
                src_address: None,
                address: None,
                deploy_set: Some(DeploySet {
                    tvc: tvc.clone(),
                    ..Default::default()
                }),
                call_set,
                value: "1000000000".to_string(),
                bounce: None,
                enable_ihr: None,
            },
        )
        .await?;

    assert_eq!(result.address, image.msg_address(0).to_string());
    assert_eq!(
        result.message_id,
        get_boc_hash(&base64::decode(&result.message)?)?
    );
    if let Some(expected_boc) = expected_boc {
        assert_eq!(&result.message, expected_boc);
    }

    let parsed: ResultOfParse = client
        .request_async(
            "boc.parse_message",
            ParamsOfParse {
                boc: result.message,
            },
        )
        .await?;

    let code_from_tvc: ResultOfGetCodeFromTvc = client
        .request_async(
            "boc.get_code_from_tvc",
            ParamsOfGetCodeFromTvc {
                tvc: tvc.clone().unwrap_or_default(),
            },
        )
        .await?;

    assert_eq!(parsed.parsed["code"], code_from_tvc.code);
    assert_eq!(parsed.parsed["msg_type_name"], "internal");
    assert_eq!(parsed.parsed["dst"], result.address);
    assert_eq!(parsed.parsed["value"], "0x3b9aca00");
    assert_eq!(parsed.parsed["bounce"], true);
    assert_eq!(parsed.parsed["ihr_disabled"], true);

    Ok(())
}

#[test]
fn test_tips() {
    let client = TestClient::new();
    let (abi, _tvc) = TestClient::package(EVENTS_OLD, Some(2));
    let err = client.request::<_, DecodedMessageBody>(
        "abi.decode_message",
        ParamsOfDecodeMessage {
            abi: abi.clone(),
            message: "te6ccgEBAgEAlgAB4a3f2/jCeWWvgMoAXOakv3VSD56sQrDPT76n1cbrSvpZ0BCs0KEUy2Duvo3zPExePONW3TYy0MCA1i+FFRXcSIXTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGQVviwSgAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            ..Default::default()
        },
    ).expect_err("Error expected");

    assert!(
        err.message.contains("Tip: Please check that you have specified the message's BOC, not body, as a parameter."),
        "{}",
        err.message
    );

    let err = client.request::<_, DecodedMessageBody>(
        "abi.decode_message_body",
        ParamsOfDecodeMessageBody {
            abi: abi.clone(),
            body: "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".into(),
            ..Default::default()
        },
    ).expect_err("Error expected");

    assert!(
        err.message
            .contains("Tip: Please check that you specified message's body, not full BOC."),
        "{}",
        err.message
    );
}

const ACCOUNT_STATE: &str = "te6ccgECGgEAAx0AAgE0BQEEWeix2Dmr4nsqu51KKUOpFDqcfirgZ5m9JN7B16iJGuXdAAABeqRZIJYAAAAW4AwDCwIBQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPDwHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACBABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEJIrtUyDjAyDA/+MCIMD+4wLyCxcHBg8Ciu1E0NdJwwH4ZiHbPNMAAZ+BAgDXGCD5AVj4QvkQ8qje0z8B+EMhufK0IPgjgQPoqIIIG3dAoLnytPhj0x8B2zz4R27yfA0IA1DtRNDXScMB+GYi0NcLA6k4ANwhxwDjAiHXDR/yvCHjAwHbPPhHbvJ8FhYIAiggghAclg/zuuMCIIIQaLVfP7rjAhEJBDow+EJu4wD4RvJz0YAW+GqI+GtyeHJwbwT4bIj4bQ0MCwoCIoj4boAP+G/4QvLgZNs8f/hnDxIAFEkgbGlrZSBpdC4ACkhlbGxvAhbtRNDXScIBio6A4hAOAlZw7UTQ9AWCEP/////4am34a234bIj4bYj4bnD4b4BA9A7yvdcL//hicPhjDw8AAACQ7UTQ0//TP9Mf0gABkdSSbQHi0gABjhXV0//T/9P/1w3/ldTR0NP/39FvBAGSbQHi1NTR0NTT/9H4b/hu+G34bPhr+Gr4Y/hiAiww+Eby4EzTH/QEWW8CAdHbPOMAf/hnFRIBcPhP+E74TfhM+Ev4SvhD+ELIy//LP8sfASBukzDPgZQBz4PM4gEgbpMwz4GOgOLMWcjMy//Nye1UEwEOAc+DAds8zRQAIG8kXiDIy//L/8v/AcjL/80AUHGVUwFvELmOHXAibxGAIPQP8rL5AFMSbxGAIPQP8rL5ALry4GWk6FsACvhG8uBMAgr0pCD0oRkYABRzb2wgMC40OC4wABKgAAAAFoBgbCE=";
const ACCOUNT_ABI: &str = r#"{
	"ABI version": 2,
	"version": "2.1",
	"header": ["time"],
	"functions": [],
	"data": [],
	"events": [],
	"fields": [
		{"name":"__pubkey","type":"uint256"},
		{"name":"__timestamp","type":"uint64"},
		{"name":"fun","type":"uint32"},
		{"name":"opt","type":"optional(bytes)"},
        {
            "name":"big",
            "type":"optional(tuple)",
            "components":[
                {"name":"value0","type":"uint256"},
                {"name":"value1","type":"uint256"},
                {"name":"value2","type":"uint256"},
                {"name":"value3","type":"uint256"}
            ]
        },
		{"name":"a","type":"bytes"},
		{"name":"b","type":"bytes"},
		{"name":"length","type":"uint256"}
	]
}"#;

#[test]
fn test_decode_account_data() {
    let abi = Abi::Json(ACCOUNT_ABI.to_owned());
    let state =
        deserialize_object_from_base64::<ton_block::StateInit>(ACCOUNT_STATE, "state").unwrap();
    let data = serialize_cell_to_base64(&state.object.data.unwrap(), "data").unwrap();

    let client = TestClient::new();
    let decoded = client
        .request::<_, ResultOfDecodeAccountData>(
            "abi.decode_account_data",
            ParamsOfDecodeAccountData {
                data,
                abi,
                allow_partial: false,
            },
        )
        .unwrap()
        .data;

    assert_eq!(
        decoded,
        json!({
            "__pubkey": "0xe8b1d839abe27b2abb9d4a2943a9143a9c7e2ae06799bd24dec1d7a8891ae5dd",
            "__timestamp": "1626254942358",
            "fun": "22",
            "opt": "48656c6c6f",
            "big": {
              "value0": "0x0000000000000000000000000000000000000000000000000000000000000002",
              "value1": "0x0000000000000000000000000000000000000000000000000000000000000008",
              "value2": "0x0000000000000000000000000000000000000000000000000000000000000002",
              "value3": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
            "a": "49206c696b652069742e",
            "b": "",
            "length": "0x000000000000000000000000000000000000000000000000000000000000000f"
        })
    );
}

#[test]
fn test_init_data() {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package("t24_initdata", Some(2));

    let data = client
        .request::<_, ResultOfDecodeStateInit>(
            "boc.decode_state_init",
            ParamsOfDecodeStateInit {
                state_init: tvc.unwrap(),
                boc_cache: None,
            },
        )
        .unwrap()
        .data
        .unwrap();

    let result: ResultOfDecodeInitialData = client
        .request(
            "abi.decode_initial_data",
            ParamsOfDecodeInitialData {
                abi: Some(abi.clone()),
                data: data.clone(),
                allow_partial: false,
            },
        )
        .unwrap();
    assert_eq!(result.initial_data, Some(json!({})));
    assert_eq!(result.initial_pubkey, hex::encode(&[0u8; 32]));

    let result: ResultOfEncodeInitialData = client
        .request(
            "abi.encode_initial_data",
            ParamsOfEncodeInitialData {
                abi: Some(abi.clone()),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(result.data, data);

    let initial_data = json!({
        "a": abi_uint(123, 8),
        "s": "some string",
    });

    const ENCODED_INITIAL_DATA: &str =
        "te6ccgEBBwEARwABAcABAgPPoAQCAQFIAwAWc29tZSBzdHJpbmcCASAGBQA\
        DHuAAQQiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIoA==";

    let result: ResultOfEncodeInitialData = client
        .request(
            "abi.encode_initial_data",
            ParamsOfEncodeInitialData {
                abi: Some(abi.clone()),
                initial_data: Some(initial_data.clone()),
                initial_pubkey: Some(hex::encode(&[0x22u8; 32])),
                boc_cache: None,
            },
        )
        .unwrap();

    assert_eq!(result.data, ENCODED_INITIAL_DATA);

    let result: ResultOfUpdateInitialData = client
        .request(
            "abi.update_initial_data",
            ParamsOfUpdateInitialData {
                abi: Some(abi.clone()),
                data: data.clone(),
                initial_data: Some(initial_data.clone()),
                initial_pubkey: Some(hex::encode(&[0x22u8; 32])),
                boc_cache: None,
            },
        )
        .unwrap();
    assert_eq!(result.data, ENCODED_INITIAL_DATA);

    let result: ResultOfDecodeInitialData = client
        .request(
            "abi.decode_initial_data",
            ParamsOfDecodeInitialData {
                abi: Some(abi.clone()),
                data: result.data,
                allow_partial: false,
            },
        )
        .unwrap();
    assert_eq!(result.initial_data, Some(initial_data.clone()));
    assert_eq!(result.initial_pubkey, hex::encode(&[0x22u8; 32]));

    let encode_result: ResultOfEncodeInitialData = client
        .request(
            "abi.encode_initial_data",
            ParamsOfEncodeInitialData {
                abi: Some(abi.clone()),
                initial_data: Some(initial_data.clone()),
                ..Default::default()
            },
        )
        .unwrap();

    let update_result: ResultOfUpdateInitialData = client
        .request(
            "abi.update_initial_data",
            ParamsOfUpdateInitialData {
                abi: Some(abi.clone()),
                data,
                initial_data: Some(initial_data.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(encode_result.data, update_result.data);
}

#[test]
fn test_decode_boc() {
    let mut builder = BuilderData::new();
    builder.append_u32(0).unwrap();
    builder
        .checked_append_reference(123u64.write_to_new_cell().unwrap().into_cell().unwrap())
        .unwrap();
    builder.append_bit_one().unwrap();

    let boc = serialize_cell_to_base64(&builder.into_cell().unwrap(), "").unwrap();

    let mut params = vec![
        AbiParam {
            name: "a".to_owned(),
            param_type: "uint32".to_owned(),
            ..Default::default()
        },
        AbiParam {
            name: "b".to_owned(),
            param_type: "ref(int64)".to_owned(),
            ..Default::default()
        },
        AbiParam {
            name: "c".to_owned(),
            param_type: "bool".to_owned(),
            ..Default::default()
        },
    ];

    let client = TestClient::new();
    let decoded = client
        .request::<_, ResultOfDecodeBoc>(
            "abi.decode_boc",
            ParamsOfDecodeBoc {
                boc: boc.clone(),
                params: params.clone(),
                allow_partial: false,
            },
        )
        .unwrap()
        .data;

    assert_eq!(
        decoded,
        json!({
            "a": "0",
            "b": "123",
            "c": true,
        })
    );

    params.pop();

    let decoded = client
        .request::<_, ResultOfDecodeBoc>(
            "abi.decode_boc",
            ParamsOfDecodeBoc {
                boc: boc.clone(),
                params: params.clone(),
                allow_partial: true,
            },
        )
        .unwrap()
        .data;

    assert_eq!(
        decoded,
        json!({
            "a": "0",
            "b": "123",
        })
    );
}

#[test]
fn test_encode_boc() {
    let client = TestClient::new();

    let params = vec![
        AbiParam {
            name: "dest".to_owned(),
            param_type: "address".to_owned(),
            ..Default::default()
        },
        AbiParam {
            name: "value".to_owned(),
            param_type: "uint128".to_owned(),
            ..Default::default()
        },
        AbiParam {
            name: "bounce".to_owned(),
            param_type: "bool".to_owned(),
            ..Default::default()
        },
    ];

    let boc = client
        .request::<_, ResultOfAbiEncodeBoc>(
            "abi.encode_boc",
            ParamsOfAbiEncodeBoc {
                params,
                data: json!({
                    "dest": "-1:3333333333333333333333333333333333333333333333333333333333333333",
                    "value": 1234567,
                    "bounce": true,
                }),
                boc_cache: None,
            },
        )
        .unwrap()
        .boc;

    assert_eq!(
        boc,
        "te6ccgEBAQEANAAAY5/mZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmAAAAAAAAAAAAAAAAACWtD4"
    );
}

#[test]
fn test_calc_function_id() {
    let client = TestClient::new();
    let abi = TestClient::abi("GiverV2", Some(2));

    let result: ResultOfCalcFunctionId = client
        .request(
            "abi.calc_function_id",
            ParamsOfCalcFunctionId {
                abi: abi.clone(),
                function_name: "getMessages".to_owned(),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(result.function_id, 0x7744C7E2);

    let result: ResultOfCalcFunctionId = client
        .request(
            "abi.calc_function_id",
            ParamsOfCalcFunctionId {
                abi: abi.clone(),
                function_name: "getMessages".to_owned(),
                output: Some(true),
            },
        )
        .unwrap();

    assert_eq!(result.function_id, 0xF744C7E2);
}

#[test]
fn decode_responsible() {
    TestClient::init_log();
    let client = TestClient::new();
    let abi = TestClient::abi("PriceXchg", Some(2));

    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Output,
        name: "onTip3LendOwnership".into(),
        value: Some(json!({
            "err_code": abi_uint(103, 32),
            "processed": abi_uint(0, 128),
            "enqueued": abi_uint(0, 128),
            "price_num": abi_uint(72600000000, 128),
            "price_denum": abi_uint(1000000000, 128),
            "user_id": abi_uint(1, 256),
            "order_id": abi_uint(1, 256),
            "pair": "0:7eb109bdfa9770a074f5aa1459bb28bbecedeffab49e2f00c7eb2d8c5e3ffaf0",
            "major_decimals": abi_uint(3, 8),
            "minor_decimals": abi_uint(3, 8),
            "sell": true,
        })),
        header: None,
    };

    let body = "te6ccgEBAgEAsQAB0IAAAAAAAABnAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABDnTBYAAAAAAAAAAAAAAAAAO5rKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAQCHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAD9YhN79S7hQOnrVCizdlF32dvf9Wk8XgGP1lsYvH/14AYHg=";

    let result: DecodedMessageBody = client
        .request(
            "abi.decode_message_body",
            ParamsOfDecodeMessageBody {
                abi,
                body: body.to_owned(),
                function_name: Some("onTip3LendOwnership".to_owned()),
                data_layout: Some(DataLayout::Output),
                is_internal: true,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(expected, result);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_deploy_code_variants() -> Result<()> {
    test_deploy_code_variants_with_contract(EVENTS_OLD, None, true).await?;
    test_deploy_code_variants_with_contract(EVENTS, None, false).await?;
    test_deploy_code_variants_with_contract(
        T24_INIT_DATA,
        Some(json!({
            "a": 123,
            "s": "abc"
        })),
        false,
    )
    .await?;
    Ok(())
}

async fn test_deploy_code_variants_with_contract(
    contract: &str,
    initial_data: Option<Value>,
    ignore_data: bool,
) -> Result<()> {
    test_deploy_code_variants_with_fn(
        encode_internal_deploy,
        contract,
        initial_data.clone(),
        ignore_data,
    )
    .await?;
    test_deploy_code_variants_with_fn(encode_deploy, contract, initial_data, ignore_data).await?;
    Ok(())
}

async fn test_deploy_code_variants_with_fn<
    F: Fn(Abi, Option<String>, Option<Value>, Option<String>, Option<String>, Option<String>) -> R,
    R: Future<Output = Value>,
>(
    encode: F,
    contract: &str,
    initial_data: Option<Value>,
    ignore_data: bool,
) -> Result<()> {
    let client = TestClient::new();
    let (abi, unknown_tvc) = TestClient::package(contract, Some(2));
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };
    let state_init_cell =
        resolve_state_init_cell(&client.context(), &unknown_tvc.clone().unwrap()).await?;
    let state_init =
        deserialize_object_from_cell::<StateInit>(state_init_cell.clone(), "state init")?;

    let encoded_with_unknown_tvc = encode(
        abi.clone(),
        Some(keys.public.clone()),
        initial_data.clone(),
        unknown_tvc.clone(),
        None,
        None,
    )
    .await;

    let encoded_with_code = encode(
        abi.clone(),
        Some(keys.public.clone()),
        initial_data.clone(),
        None,
        Some(serialize_cell_to_base64(
            &state_init.code.clone().unwrap(),
            "state init",
        )?),
        None,
    )
    .await;
    assert_eq!(
        convert_parsed(&encoded_with_unknown_tvc, ignore_data),
        convert_parsed(&encoded_with_code, ignore_data)
    );

    let encoded_with_state_init = encode(
        abi.clone(),
        Some(keys.public.clone()),
        initial_data.clone(),
        None,
        None,
        Some(serialize_cell_to_base64(&state_init_cell, "state init").unwrap()),
    )
    .await;
    assert_eq!(encoded_with_unknown_tvc, encoded_with_state_init);

    let tvc = base64::encode(
        &TVC {
            tvc: TvmSmc::TvcFrst(TvcFrst {
                code: state_init.code.clone().unwrap(),
                meta: Some(Metadata {
                    name: SmallStr {
                        string: "Some Toolchain".to_string(),
                    },
                    compiled_at: 123,
                    sold: Version::new([0u8; 20], "v1.2.3".to_string()),
                    linker: Version::new([0u8; 20], "v1.2.3".to_string()),
                    desc: "Some Contract".to_string(),
                }),
            }),
        }
        .write_to_bytes()
        .unwrap(),
    );
    let encoded_with_tvc = encode(
        abi.clone(),
        Some(keys.public.clone()),
        initial_data.clone(),
        Some(tvc),
        None,
        None,
    )
    .await;
    assert_eq!(
        convert_parsed(&encoded_with_unknown_tvc, ignore_data),
        convert_parsed(&encoded_with_tvc, ignore_data)
    );
    Ok(())
}

async fn encode_internal_deploy(
    abi: Abi,
    initial_pubkey: Option<String>,
    initial_data: Option<Value>,
    tvc: Option<String>,
    code: Option<String>,
    state_init: Option<String>,
) -> Value {
    let client = TestClient::new();
    let encoded: ResultOfEncodeInternalMessage = client
        .request_async(
            "abi.encode_internal_message",
            ParamsOfEncodeInternalMessage {
                abi: Some(abi),
                deploy_set: Some(DeploySet {
                    tvc,
                    code,
                    state_init,
                    initial_pubkey,
                    initial_data,
                    ..Default::default()
                }),
                value: "1000000000".to_string(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    parse_message(
        client.context().clone(),
        ParamsOfParse {
            boc: encoded.message,
        },
    )
    .await
    .unwrap()
    .parsed
}

async fn encode_deploy(
    abi: Abi,
    initial_pubkey: Option<String>,
    initial_data: Option<Value>,
    tvc: Option<String>,
    code: Option<String>,
    state_init: Option<String>,
) -> Value {
    let client = TestClient::new();
    let encoded: ResultOfEncodeMessage = client
        .request_async(
            "abi.encode_message",
            ParamsOfEncodeMessage {
                abi,
                deploy_set: Some(DeploySet {
                    tvc,
                    code,
                    state_init,
                    initial_pubkey,
                    initial_data,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    parse_message(
        client.context().clone(),
        ParamsOfParse {
            boc: encoded.message,
        },
    )
    .await
    .unwrap()
    .parsed
}

fn convert_parsed(value: &Value, strip_data: bool) -> Value {
    if !strip_data {
        return value.clone();
    }
    let mut value = value.clone();
    if let Value::Object(obj) = &mut value {
        for field in ["id", "boc", "data", "data_hash", "dst"] {
            obj.remove(field);
        }
    }
    value
}
