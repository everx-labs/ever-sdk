/*
* Copyright 2018-2021 TON Labs LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use super::*;
use crate::abi::{
    CallSet, DeploySet, FunctionHeader, ParamsOfEncodeMessage, ResultOfEncodeMessage, Signer,
};
use crate::api_info::ApiModule;
use crate::crypto::KeyPair;
use crate::json_interface::modules::BocModule;
use crate::tests::{TestClient, EVENTS};
use internal::serialize_cell_to_base64;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::str::FromStr;
use ton_block::{MsgAddrStd, MsgAddressInt, Serializable};
use ton_types::{AccountId, BuilderData, IBitstring};

#[tokio::test(core_threads = 2)]
async fn test_encode_boc() {
    fn write_b(value: u8) -> BuilderOp {
        BuilderOp::Integer {
            size: 1,
            value: Value::from(value),
        }
    }
    fn write_u128(value: u128) -> BuilderOp {
        BuilderOp::Integer {
            size: 128,
            value: Value::from(value.to_string()),
        }
    }
    fn write_u8(value: u8) -> BuilderOp {
        BuilderOp::Integer {
            size: 8,
            value: Value::from(value),
        }
    }
    fn write_i8(value: i8) -> BuilderOp {
        BuilderOp::Integer {
            size: 8,
            value: Value::from(value),
        }
    }
    fn write_i(value: Value, size: u32) -> BuilderOp {
        BuilderOp::Integer { size, value }
    }
    fn write_bitstring(value: &str) -> BuilderOp {
        BuilderOp::BitString {
            value: value.into(),
        }
    }
    fn write_address(address: String) -> BuilderOp {
        BuilderOp::Address { address }
    }
    fn write_cell(write: Vec<BuilderOp>) -> BuilderOp {
        BuilderOp::Cell { builder: write }
    }

    let client = TestClient::new();
    let encode_boc = client.wrap_async(
        encode_boc,
        BocModule::api(),
        super::encode::encode_boc_api(),
    );
    let mut inner_builder = BuilderData::new();
    inner_builder.append_bits(0b101100111000, 12).unwrap();
    inner_builder.append_bits(0b100111000, 9).unwrap();
    inner_builder.append_bits(0b111, 3).unwrap();
    inner_builder.append_bits(2, 3).unwrap();
    inner_builder.append_u16(0b100111000).unwrap();
    inner_builder.append_u16(0x123).unwrap();
    inner_builder.append_i16(0x123).unwrap();
    inner_builder.append_i16(-0x123).unwrap();

    let burner_account_id = "efd5a14409a8a129686114fc092525fddd508f1ea56d1b649a3a695d3a5b188c";

    let burner_address = MsgAddressInt::AddrStd(MsgAddrStd::with_address(
        None,
        -1,
        AccountId::from_str(burner_account_id).unwrap(),
    ))
    .write_to_new_cell()
    .unwrap();

    let mut builder = BuilderData::new();
    builder
        .append_bit_one()
        .unwrap()
        .append_bit_zero()
        .unwrap()
        .append_u8(255)
        .unwrap()
        .append_i8(127)
        .unwrap()
        .append_i8(-127)
        .unwrap()
        .append_u128(123456789123456789u128)
        .unwrap()
        .append_bits(0b100010, 6)
        .unwrap()
        .append_bits(0b100010, 6)
        .unwrap()
        .append_bits(0x123, 12)
        .unwrap()
        .append_bits(0b00101101100, 11)
        .unwrap()
        .append_builder(&burner_address)
        .unwrap();
    let inner_cell = inner_builder.into_cell().unwrap();
    builder.append_reference_cell(inner_cell.clone());

    let cell = builder.into_cell().unwrap();
    let boc = serialize_cell_to_base64(&cell, "cell").unwrap();

    let response = encode_boc
        .call(ParamsOfEncodeBoc {
            builder: vec![
                write_b(1),
                write_b(0),
                write_u8(255),
                write_i8(127),
                write_i8(-127),
                write_u128(123456789123456789u128),
                write_bitstring("8A_"),
                write_bitstring("x{8A0_}"),
                write_bitstring("123"),
                write_bitstring("x2d9_"),
                write_bitstring("80_"),
                write_address(format!("-1:{}", burner_account_id)),
                write_cell(vec![
                    write_bitstring("n101100111000"),
                    write_bitstring("N100111000"),
                    write_i(Value::from(-1), 3),
                    write_i(Value::from(2), 3),
                    write_i(Value::from(0b100111000), 16),
                    write_i(Value::from("0x123"), 16),
                    write_i(Value::from("0x123"), 16),
                    write_i(Value::from("-0x123"), 16),
                ]),
            ],
            boc_cache: None,
        })
        .await
        .unwrap();
    assert_eq!(boc, response.boc);

    let response = encode_boc
        .call(ParamsOfEncodeBoc {
            builder: vec![
                write_b(1),
                write_b(0),
                write_u8(255),
                write_i8(127),
                write_i8(-127),
                write_u128(123456789123456789u128),
                write_bitstring("8A_"),
                write_bitstring("x{8A0_}"),
                write_bitstring("123"),
                write_bitstring("x2d9_"),
                write_bitstring("80_"),
                write_address(format!("-1:{}", burner_account_id)),
                BuilderOp::CellBoc {
                    boc: serialize_cell_to_base64(&inner_cell, "cell").unwrap(),
                },
            ],
            boc_cache: None,
        })
        .await
        .unwrap();
    assert_eq!(boc, response.boc);
}

#[tokio::test(core_threads = 2)]
async fn test_pinned_cache() {
    let client = TestClient::new();
    let cache_set = client.wrap_async(cache_set, BocModule::api(), super::cache::cache_set_api());
    let cache_get = client.wrap_async(cache_get, BocModule::api(), super::cache::cache_get_api());
    let cache_unpin = client.wrap_async(
        cache_unpin,
        BocModule::api(),
        super::cache::cache_unpin_api(),
    );

    let boc1 = TestClient::tvc(crate::tests::HELLO, None);
    let boc2 = TestClient::tvc(crate::tests::EVENTS, None);

    let pin1 = "pin1".to_owned();
    let pin2 = "pin2".to_owned();

    let ref1 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc1.clone(),
            cache_type: BocCacheType::Pinned { pin: pin1.clone() },
        })
        .await
        .unwrap()
        .boc_ref;

    assert!(ref1.starts_with("*"));
    assert_eq!(ref1.len(), 65);

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, Some(boc1.clone()));

    let ref2 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc2.clone(),
            cache_type: BocCacheType::Pinned { pin: pin1.clone() },
        })
        .await
        .unwrap()
        .boc_ref;
    assert_ne!(ref2, ref1);

    let ref3 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc1.clone(),
            cache_type: BocCacheType::Pinned { pin: pin2.clone() },
        })
        .await
        .unwrap()
        .boc_ref;
    assert_eq!(ref3, ref1);

    // unpin pin1 and check that boc2 which had only this pin is removed from cache but boc1 which
    // had both pins is still in cache
    cache_unpin
        .call(ParamsOfBocCacheUnpin {
            boc_ref: None,
            pin: pin1.clone(),
        })
        .await
        .unwrap();

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, Some(boc1.clone()));

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref2.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, None);

    let ref4 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc2,
            cache_type: BocCacheType::Pinned { pin: pin2.clone() },
        })
        .await
        .unwrap()
        .boc_ref;

    // unpin pin2 with particular ref and that only this ref is removed from cache
    cache_unpin
        .call(ParamsOfBocCacheUnpin {
            boc_ref: Some(ref4.clone()),
            pin: pin2.clone(),
        })
        .await
        .unwrap();

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, Some(boc1.clone()));

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref4.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, None);

    cache_unpin
        .call(ParamsOfBocCacheUnpin {
            boc_ref: None,
            pin: pin2.clone(),
        })
        .await
        .unwrap();
    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, None);
}
#[tokio::test(core_threads = 2)]
async fn test_unpinned_cache() {
    let boc1 = TestClient::tvc(crate::tests::TEST_DEBOT, None);
    let boc2 = TestClient::tvc(crate::tests::SUBSCRIBE, None);

    let boc_max_size = std::cmp::max(
        base64::decode(&boc1).unwrap().len(),
        base64::decode(&boc2).unwrap().len(),
    );
    let client = TestClient::new_with_config(json!({
        "boc": {
            "cache_max_size": boc_max_size / 1024 + 1
        }
    }));
    let cache_set = client.wrap_async(cache_set, BocModule::api(), super::cache::cache_set_api());
    let cache_get = client.wrap_async(cache_get, BocModule::api(), super::cache::cache_get_api());

    let ref1 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc1.clone(),
            cache_type: BocCacheType::Unpinned,
        })
        .await
        .unwrap()
        .boc_ref;

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, Some(boc1.clone()));

    // add second BOC to remove first BOC by insufficient cache size
    let ref2 = cache_set
        .call(ParamsOfBocCacheSet {
            boc: boc2.clone(),
            cache_type: BocCacheType::Unpinned,
        })
        .await
        .unwrap()
        .boc_ref;

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref1.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, None);

    let boc = cache_get
        .call(ParamsOfBocCacheGet {
            boc_ref: ref2.clone(),
        })
        .await
        .unwrap();
    assert_eq!(boc.boc, Some(boc2.clone()));
}

#[test]
fn get_boc_hash() {
    let client = TestClient::new();

    let result: super::ResultOfGetBocHash = client.request(
        "boc.get_boc_hash",
        super::ParamsOfGetBocHash {
            boc: String::from("te6ccgEBAQEAWAAAq2n+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE/zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzSsG8DgAAAAAjuOu9NAL7BxYpA")
        }
    ).unwrap();

    assert_eq!(
        result.hash,
        "dfd47194f3058ee058bfbfad3ea40cbbd9ad17ca77cd0904d4d9f18a48c2fbca"
    );
}

#[test]
fn get_boc_depth() {
    let client = TestClient::new();

    let result: super::ResultOfGetBocDepth = client
        .request(
            "boc.get_boc_depth",
            super::ParamsOfGetBocDepth {
                boc: base64::encode(include_bytes!("test_data/account.boc")),
            },
        )
        .unwrap();

    assert_eq!(result.depth, 8);
}

#[test]
fn get_code_from_tvc() {
    let client = TestClient::new();

    let result: super::ResultOfGetCodeFromTvc = client.request(
        "boc.get_code_from_tvc",
        super::ParamsOfGetCodeFromTvc {
            tvc: String::from("te6ccgECHAEABDkAAgE0BgEBAcACAgPPIAUDAQHeBAAD0CAAQdgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAIm/wD0pCAiwAGS9KDhiu1TWDD0oQkHAQr0pCD0oQgAAAIBIAwKAej/fyHTAAGOJoECANcYIPkBAXDtRND0BYBA9A7yitcL/wHtRyJvde1XAwH5EPKo3u1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi0z8B7UdvEyG5IAsAYJ8wIPgjgQPoqIIIG3dAoLneme1HIW9TIO1XMJSANPLw4jDTHwH4I7zyudMfAfFAAQIBIBgNAgEgEQ4BCbqLVfP4DwH67UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3u1HbxaS8jOX7Udxb1btV+IA+ADR+CO1H+1HIG8RMAHIyx/J0G9R7VftR28SyPQA7UdvE88LP+1HbxYQABzPCwDtR28RzxbJ7VRwagIBahUSAQm0ABrWwBMB/O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t7tR29lIG6SMHDecO1HbxKAQPQO8orXC/+68uBk+AD6QNEgyMn7BIED6HCBAIDIcc8LASLPCgBxz0D4KBQAjs8WJM8WI/oCcc9AcPoCcPoCgEDPQPgjzwsfcs9AIMki+wBfBTDtR28SyPQA7UdvE88LP+1HbxbPCwDtR28RzxbJ7VRwatswAQm0ZfaLwBYB+O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t7R7UdvEdcLH8iCEFDL7ReCEIAAAACxzwsfIc8LH8hzzwsB+CjPFnLPQPglzws/gCHPQCDPNSLPMbwXAHiWcc9AIc8XlXHPQSHN4iDJcfsAWyHA/44e7UdvEsj0AO1HbxPPCz/tR28WzwsA7UdvEc8Wye1U3nFq2zACASAbGQEJu3MS5FgaAPjtR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le+ADR+CO1H+1HIG8RMAHIyx/J0G9R7VftR28SyPQA7UdvE88LP+1HbxbPCwDtR28RzxbJ7VRwatswAMrdcCHXSSDBII4rIMAAjhwj0HPXIdcLACDAAZbbMF8H2zCW2zBfB9sw4wTZltswXwbbMOME2eAi0x80IHS7II4VMCCCEP////+6IJkwIIIQ/////rrf35bbMF8H2zDgIyHxQAFfBw==")
        }
    ).unwrap();

    assert_eq!(
        result.code,
        "te6ccgECFgEAA/8AAib/APSkICLAAZL0oOGK7VNYMPShAwEBCvSkIPShAgAAAgEgBgQB6P9/IdMAAY4mgQIA1xgg+QEBcO1E0PQFgED0DvKK1wv/Ae1HIm917VcDAfkQ8qje7UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+LTPwHtR28TIbkgBQBgnzAg+COBA+iogggbd0Cgud6Z7Uchb1Mg7VcwlIA08vDiMNMfAfgjvPK50x8B8UABAgEgEgcCASALCAEJuotV8/gJAfrtR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOGPQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le7UdvFpLyM5ftR3FvVu1X4gD4ANH4I7Uf7UcgbxEwAcjLH8nQb1HtV+1HbxLI9ADtR28Tzws/7UdvFgoAHM8LAO1HbxHPFsntVHBqAgFqDwwBCbQAGtbADQH87UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3u1Hb2UgbpIwcN5w7UdvEoBA9A7yitcL/7ry4GT4APpA0SDIyfsEgQPocIEAgMhxzwsBIs8KAHHPQPgoDgCOzxYkzxYj+gJxz0Bw+gJw+gKAQM9A+CPPCx9yz0AgySL7AF8FMO1HbxLI9ADtR28Tzws/7UdvFs8LAO1HbxHPFsntVHBq2zABCbRl9ovAEAH47UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi3tHtR28R1wsfyIIQUMvtF4IQgAAAALHPCx8hzwsfyHPPCwH4KM8Wcs9A+CXPCz+AIc9AIM81Is8xvBEAeJZxz0AhzxeVcc9BIc3iIMlx+wBbIcD/jh7tR28SyPQA7UdvE88LP+1HbxbPCwDtR28RzxbJ7VTecWrbMAIBIBUTAQm7cxLkWBQA+O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t74ANH4I7Uf7UcgbxEwAcjLH8nQb1HtV+1HbxLI9ADtR28Tzws/7UdvFs8LAO1HbxHPFsntVHBq2zAAyt1wIddJIMEgjisgwACOHCPQc9ch1wsAIMABltswXwfbMJbbMF8H2zDjBNmW2zBfBtsw4wTZ4CLTHzQgdLsgjhUwIIIQ/////7ogmTAgghD////+ut/fltswXwfbMOAjIfFAAV8H"
    );
}

#[test]
fn parse_message() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_message",
        ParamsOfParse {
            boc: String::from("te6ccgEBAQEAWAAAq2n+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE/zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzSsG8DgAAAAAjuOu9NAL7BxYpA")
        }
    ).unwrap();

    assert_eq!(
        result.parsed["id"],
        "dfd47194f3058ee058bfbfad3ea40cbbd9ad17ca77cd0904d4d9f18a48c2fbca"
    );
    assert_eq!(
        result.parsed["src"],
        "-1:0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(
        result.parsed["dst"],
        "-1:3333333333333333333333333333333333333333333333333333333333333333"
    );
}

#[test]
fn parse_account() {
    let client = TestClient::new();

    let result: ResultOfParse = client
        .request(
            "boc.parse_account",
            ParamsOfParse {
                boc: base64::encode(&include_bytes!("test_data/account.boc")),
            },
        )
        .unwrap();

    assert_eq!(
        result.parsed["id"],
        "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"
    );
    assert_eq!(result.parsed["last_trans_lt"], "0x20eadff7e03");
    assert_eq!(result.parsed["balance"], "0x958a26eb8e7a18d");
}

#[test]
fn parse_transaction() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_transaction",
        ParamsOfParse {
            boc: String::from("te6ccgECBwEAAZQAA7V75gA6WK5sEDTiHFGnH9ILOy2irjKLWTkWQMyMogsg40AAACDribjoE3gOAbYNpCaX4uLeXPQHt2Kw/Jp2OKkR2s+BASyeQM6wAAAg64IXyBX2DobAABRrMENIBQQBAhUEQojmJaAYazBCEQMCAFvAAAAAAAAAAAAAAAABLUUtpEnlC4z33SeGHxRhIq/htUa7i3D8ghbwxhQTn44EAJxC3UicQAAAAAAAAAAAdwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgnJAnYEvIQY6SnQKc3lXk6x1Z/lyplGFRbwAuNtVBi9EeceU3Ojl0F3EkRdylowY5x2qlgHNv4lNZUjhq0WqrLMNAQGgBgC3aADLL4ChL2HyLHwOLub5Mep87W3xdnMW8BpxKyVoGe3RPQAvmADpYrmwQNOIcUacf0gs7LaKuMotZORZAzIyiCyDjQ5iWgAGFFhgAAAEHXC9CwS+wdDGKTmMFkA=")
        }
    ).unwrap();

    assert_eq!(
        result.parsed["id"],
        "d6315dbb2a741a2765da250bea4a186adf942469369c703c57c2050e2d6e9fe3"
    );
    assert_eq!(result.parsed["lt"], "0x20eb89b8e81");
    assert_eq!(result.parsed["now"], 1600186476);
}

#[test]
fn parse_block() {
    let client = TestClient::new();

    let result: ResultOfParse = client.request(
        "boc.parse_block",
        ParamsOfParse {
            boc: String::from("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")
        }
    ).unwrap();

    assert_eq!(
        result.parsed["id"],
        "048f59d5d652459939ea5c5e7b291155205696b71e0c556f641df69e70e1e725"
    );
    assert_eq!(result.parsed["seq_no"], 4296363);
    assert_eq!(result.parsed["gen_utime"], 1600234696);
}

#[test]
fn parse_shardstate() {
    let client = TestClient::new();

    let result: ResultOfParse = client
        .request(
            "boc.parse_shardstate",
            ParamsOfParseShardstate {
                id: String::from("zerostate:-1"),
                workchain_id: -1,
                boc: base64::encode(&include_bytes!("test_data/zerostate.boc")),
            },
        )
        .unwrap();

    assert_eq!(result.parsed["id"], "zerostate:-1");
    assert_eq!(result.parsed["workchain_id"], -1);
    assert_eq!(result.parsed["seq_no"], 0);
}

#[test]
fn get_blockchain_config() {
    let client = TestClient::new();

    let result: ResultOfGetBlockchainConfig = client
        .request(
            "boc.get_blockchain_config",
            ParamsOfGetBlockchainConfig {
                block_boc: base64::encode(&include_bytes!("test_data/block.boc")),
            },
        )
        .unwrap();

    assert_eq!(
        result.config_boc,
        base64::encode(&include_bytes!("test_data/block_config.boc"))
    );

    let result: ResultOfGetBlockchainConfig = client
        .request(
            "boc.get_blockchain_config",
            ParamsOfGetBlockchainConfig {
                block_boc: base64::encode(&include_bytes!("test_data/zerostate.boc")),
            },
        )
        .unwrap();

    assert_eq!(
        result.config_boc,
        base64::encode(&include_bytes!("test_data/zerostate_config.boc"))
    );
}

fn read_salted_boc(name: &str) -> String {
    base64::encode(&std::fs::read("src/boc/test_data/salt/".to_owned() + name).unwrap())
}

fn check_salt(
    client: &TestClient,
    name: &str,
    read_salt: Option<&str>,
    set_salt: &str,
    name_with_salt: Option<&str>,
) {
    let code = read_salted_boc(name);
    let result: ResultOfGetCodeSalt = client
        .request(
            "boc.get_code_salt",
            ParamsOfGetCodeSalt {
                code: code.clone(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.salt.as_ref().map(AsRef::as_ref), read_salt);

    let result: ResultOfSetCodeSalt = client
        .request(
            "boc.set_code_salt",
            ParamsOfSetCodeSalt {
                code,
                salt: set_salt.to_owned(),
                boc_cache: Some(BocCacheType::Unpinned),
            },
        )
        .unwrap();

    if let Some(name_with_salt) = name_with_salt {
        let boc: ResultOfBocCacheGet = client
            .request(
                "boc.cache_get",
                ParamsOfBocCacheGet {
                    boc_ref: result.code.clone(),
                },
            )
            .unwrap();
        assert_eq!(boc.boc.unwrap(), read_salted_boc(name_with_salt));
    }

    let result: ResultOfGetCodeSalt = client
        .request(
            "boc.get_code_salt",
            ParamsOfGetCodeSalt {
                code: result.code,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.salt.unwrap(), set_salt);
}

#[test]
fn test_code_salt() {
    let client = TestClient::new();

    check_salt(
        &client,
        "old_cpp_sel_nosalt.boc",
        None,
        "te6ccgEBAQEAJAAAQ4AGPqCXQ2drhdqhLLt3rJ80LxA65YMTwgWLLUmt9EbElFA=",
        None,
    );
    check_salt(
        &client,
        "old_cpp_sel_salt.boc",
        Some("te6ccgEBAQEAJAAAQ4AGPqCXQ2drhdqhLLt3rJ80LxA65YMTwgWLLUmt9EbElFA="),
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADk",
        None,
    );
    check_salt(
        &client,
        "new_sel_nodict_nosalt.boc",
        None,
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADk",
        Some("new_sel_nodict_salt.boc"),
    );
    check_salt(
        &client,
        "new_sel_nodict_salt.boc",
        Some("te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADk"),
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABF",
        None,
    );
    check_salt(
        &client,
        "new_sel_dict_nosalt.boc",
        None,
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABF",
        Some("new_sel_dict_salt.boc"),
    );
    check_salt(
        &client,
        "new_sel_dict_salt.boc",
        Some("te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABF"),
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKa",
        None,
    );
    check_salt(
        &client,
        "mycode_sel_nodict_nosalt.boc",
        None,
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKa",
        Some("mycode_sel_nodict_salt.boc"),
    );
    check_salt(
        &client,
        "mycode_sel_nodict_salt.boc",
        Some("te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKa"),
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACG",
        None,
    );
    check_salt(
        &client,
        "mycode_sel_dict_nosalt.boc",
        None,
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACG",
        Some("mycode_sel_dict_salt.boc"),
    );
    check_salt(
        &client,
        "mycode_sel_dict_salt.boc",
        Some("te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACG"),
        "te6ccgEBAQEAIgAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADk",
        None,
    );

    let code = read_salted_boc("old_sol_sel.boc");
    let result: ResultOfGetCodeSalt = client
        .request(
            "boc.get_code_salt",
            ParamsOfGetCodeSalt {
                code: code.clone(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.salt, None);
}

fn check_encode_tvc(client: &TestClient, tvc: String, decoded: ResultOfDecodeTvc) {
    let result: ResultOfDecodeTvc = client
        .request(
            "boc.decode_tvc",
            ParamsOfDecodeTvc {
                tvc: tvc.clone(),
                boc_cache: None,
            },
        )
        .unwrap();
    assert_eq!(result, decoded);

    let result: ResultOfEncodeTvc = client
        .request(
            "boc.encode_tvc",
            ParamsOfEncodeTvc {
                code: result.code,
                data: result.data,
                library: result.library,
                split_depth: result.split_depth,
                tick: result.tick,
                tock: result.tock,
                boc_cache: None,
            },
        )
        .unwrap();
    assert_eq!(result.tvc, tvc);
}

#[test]
fn test_tvc_encode() {
    let client = TestClient::new();

    let tvc = TestClient::tvc("t24_initdata", Some(2));
    let decoded = ResultOfDecodeTvc {
        code: Some(String::from("te6ccgECEAEAAYkABCSK7VMg4wMgwP/jAiDA/uMC8gsNAgEPAoTtRNDXScMB+GYh2zzTAAGfgQIA1xgg+QFY+EL5EPKo3tM/AfhDIbnytCD4I4ED6KiCCBt3QKC58rT4Y9MfAds88jwFAwNK7UTQ10nDAfhmItDXCwOpOADcIccA4wIh1w0f8rwh4wMB2zzyPAwMAwIoIIIQBoFGw7rjAiCCEGi1Xz+64wIIBAIiMPhCbuMA+Ebyc9H4ANs88gAFCQIW7UTQ10nCAYqOgOILBgFccO1E0PQFcSGAQPQOk9cLB5Fw4vhqciGAQPQPjoDf+GuAQPQO8r3XC//4YnD4YwcBAogPA3Aw+Eby4Ez4Qm7jANHbPCKOICTQ0wH6QDAxyM+HIM6AYs9AXgHPkhoFGw7LB8zJcPsAkVvi4wDyAAsKCQAq+Ev4SvhD+ELIy//LP8+DywfMye1UAAj4SvhLACztRNDT/9M/0wAx0wfU0fhr+Gr4Y/hiAAr4RvLgTAIK9KQg9KEPDgAUc29sIDAuNTEuMAAA")),
        code_depth: Some(7),
        code_hash: Some(String::from("0ad23f96d7b1c1ce78dae573ac8cdf71523dc30f36316b5aaa5eb3cc540df0e0")),
        data: Some(String::from("te6ccgEBAgEAKAABAcABAEPQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg")),
        data_depth: Some(1),
        data_hash: Some(String::from("55a703465a160dce20481375de2e5b830c841c2787303835eb5821d62d65ca9d")),
        library: None,
        split_depth: None,
        tick: None,
        tock: None,
        compiler_version: Some("sol 0.51.0".to_owned()),
    };

    check_encode_tvc(&client, tvc, decoded);

    let tvc = base64::encode(include_bytes!("test_data/state_init_lib.boc"));
    let decoded = ResultOfDecodeTvc {
        code: Some(String::from("te6ccgEBBAEAhwABFP8A9KQT9LzyyAsBAgEgAwIA36X//3aiaGmP6f/o5CxSZ4WPkOeF/+T2qmRnxET/s2X/wQgC+vCAfQFANeegZLh9gEB354V/wQgD39JAfQFANeegZLhkZ82JA6Mrm6RBCAOt5or9AUA156BF6kMrY2N5YQO7e5NjIQxni2S4fYB9gEAAAtI=")),
        code_depth: Some(2),
        code_hash: Some(String::from("45910e27fe37d8dcf1fac777ebb3bda38ae1ea8389f81bfb1bc0079f3f67ef5b")),
        data: Some(String::from("te6ccgEBAQEAJgAASBHvVgMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==")),
        data_depth: Some(0),
        data_hash: Some(String::from("97bfef744b0d45f78b901e2997fb55f6dbc1d396a8d2f8f4c3a5468c010db67a")),
        library: Some(String::from("te6ccgEBBgEAYAACAWIEAQFCv0EkKSBepm1vIATt+lcPb1az6F5ZuqG++8c7faXVW9xhAgEEEjQDAARWeAFCv1ou71BWd19blXL/OtY90qcdH7KByhd6Xhx0cw7MsuUTBQAPq6yrrausq6g=")),
        split_depth: None,
        tick: Some(true),
        tock: Some(true),
        compiler_version: None,
    };

    check_encode_tvc(&client, tvc, decoded);
}

#[test]
fn test_get_compiler_version() {
    let client = TestClient::new();

    let tvc = TestClient::tvc("t24_initdata", Some(2));

    let code = client
        .request::<_, ResultOfDecodeTvc>(
            "boc.decode_tvc",
            ParamsOfDecodeTvc {
                tvc,
                boc_cache: None,
            },
        )
        .unwrap()
        .code
        .unwrap();

    let result: ResultOfGetCompilerVersion = client
        .request(
            "boc.get_compiler_version",
            ParamsOfGetCompilerVersion { code },
        )
        .unwrap();

    assert_eq!(result.version.as_deref(), Some("sol 0.51.0"));
}

#[test]
fn encode_external_in_message() {
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

    let deploy_params = |signing: Signer| ParamsOfEncodeMessage {
        abi: abi.clone(),
        address: None,
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
        processing_try_index: None,
    };

    let abi_encoded: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::Keys { keys: keys.clone() }),
        )
        .unwrap();
    assert_eq!(abi_encoded.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    let abi_parsed = client
        .request::<ParamsOfParse, ResultOfParse>(
            "boc.parse_message",
            ParamsOfParse {
                boc: abi_encoded.message.clone(),
            },
        )
        .unwrap()
        .parsed;
    let init = client
        .request::<ParamsOfEncodeTvc, ResultOfEncodeTvc>(
            "boc.encode_tvc",
            ParamsOfEncodeTvc {
                code: abi_parsed["code"].as_str().map(|x| x.to_string()),
                data: abi_parsed["data"].as_str().map(|x| x.to_string()),
                library: abi_parsed["library"].as_str().map(|x| x.to_string()),
                ..Default::default()
            },
        )
        .unwrap()
        .tvc;
    let boc_encoded: ResultOfEncodeExternalInMessage = client
        .request(
            "boc.encode_external_in_message",
            ParamsOfEncodeExternalInMessage {
                dst: abi_encoded.address.clone(),
                body: abi_parsed["body"].as_str().map(|x| x.to_string()),
                init: Some(init),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(boc_encoded.message, abi_encoded.message);
}
