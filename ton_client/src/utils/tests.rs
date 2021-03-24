use crate::{boc::tests::ACCOUNT, json_interface::modules::UtilsModule};
use crate::tests::TestClient;
use super::*;
use crate::utils::compression::{ParamsOfCompressZstd, ResultOfCompressZstd, ResultOfDecompressZstd, ParamsOfDecompressZstd};
use api_info::ApiModule;

#[tokio::test(core_threads = 2)]
async fn test_utils() {
    TestClient::init_log();
    let client = TestClient::new();
    let convert_address = client.wrap(
        convert_address,
        UtilsModule::api(),
        super::conversion::convert_address_api(),
    );

    let account_id = "fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let hex = "-1:fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let hex_workchain0 = "0:fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let base64 = "Uf/8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15+KsQHFLbKSMiYG+9";
    let base64url = "kf_8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15-KsQHFLbKSMiYIny";

    let converted = convert_address
        .call(ParamsOfConvertAddress {
            address: account_id.into(),
            output_format: AddressStringFormat::Hex {},
        })
        .unwrap();
    assert_eq!(converted.address, hex_workchain0);

    let converted = convert_address
        .call(ParamsOfConvertAddress {
            address: account_id.into(),
            output_format: AddressStringFormat::AccountId {},
        })
        .unwrap();
    assert_eq!(converted.address, account_id);

    let converted = convert_address
        .call(ParamsOfConvertAddress {
            address: hex.into(),
            output_format: AddressStringFormat::Base64 {
                bounce: false,
                test: false,
                url: false,
            },
        })
        .unwrap();
    assert_eq!(converted.address, base64);

    let converted = convert_address
        .call(ParamsOfConvertAddress {
            address: base64.into(),
            output_format: AddressStringFormat::Base64 {
                bounce: true,
                test: true,
                url: true,
            },
        })
        .unwrap();
    assert_eq!(converted.address, base64url);

    let converted = convert_address
        .call(ParamsOfConvertAddress {
            address: base64url.into(),
            output_format: AddressStringFormat::Hex {},
        })
        .unwrap();
    assert_eq!(converted.address, hex);
}

#[tokio::test(core_threads = 2)]
async fn test_calc_storage_fee() {
    let client = TestClient::new();

    let result: ResultOfCalcStorageFee = client.request_async(
        "utils.calc_storage_fee",
        ParamsOfCalcStorageFee {
            account: String::from(ACCOUNT),
            period: 1000,
        }
    ).await.unwrap();

    assert_eq!(result.fee, "330");
}

#[test]
fn test_compression() {
    let client = TestClient::new();
    let uncompressed =
        b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
        incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
        exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure \
        dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
        Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit \
        anim id est laborum.";

    let compressed: ResultOfCompressZstd = client.request(
        "utils.compress_zstd",
        ParamsOfCompressZstd {
            uncompressed: base64::encode(uncompressed),
            level: 21,
        }
    ).unwrap();

    assert_eq!(
        compressed.compressed,
        "KLUv/QCAdQgAJhc5GJCnsA2AIm2tVzjno88mHb3Ttx9b8fXHHDAAMgAyAMUsVo6Pi3rPTDF2WDl510aHTwt44hrUxb\
        n5oF6iUfiUiRbQhYo/PSM2WvKYt/hMIOQmuOaY/bmJQoRky46EF+cEd+Thsep5Hloo9DLCSwe1vFwcqIHycEKlMqBSo\
        +szAiIBhkukH5kSIVlFukEWNF2SkIv6HBdPjFAjoUliCPjzKB/4jK91X95rTAKoASkPNqwUEw2Gkscdb3lR8YRYOR+P\
        0sULCqzPQ8mQFJWnBSyP25mWIY2bFEUSJiGsWD+9NBqLhIAGDggQkLMbt5Y1aDR4uLKqwJXmQFPg/XTXIL7LCgspIF1\
        YYplND4Uo"
    );

    let decompressed: ResultOfDecompressZstd = client.request(
        "utils.decompress_zstd",
        ParamsOfDecompressZstd {
            compressed: compressed.compressed
        }
    ).unwrap();

    let decompressed = base64::decode(&decompressed.decompressed).unwrap();

    assert_eq!(decompressed, uncompressed);
}
