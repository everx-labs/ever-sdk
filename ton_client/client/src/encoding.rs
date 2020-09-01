/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

#![allow(dead_code)]

use crate::error::{ApiResult, ApiError};
use ton_block::MsgAddressInt;
use std::str::FromStr;


#[derive(Serialize, Deserialize, TypeInfo)]
pub struct InputData {
    pub text: Option<String>,
    pub hex: Option<String>,
    pub base64: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum OutputEncoding {
    Text,
    Hex,
    HexUppercase,
    Base64,
}

impl InputData {
    pub fn text(value: &str) -> Self {
        Self { text: Some(value.into()), hex: None, base64: None }
    }

    pub fn hex(value: &str) -> Self {
        Self { text: None, hex: Some(value.into()), base64: None }
    }

    pub fn base64(value: &str) -> Self {
        Self { text: None, hex: None, base64: Some(value.into()) }
    }

    pub fn decode(&self) -> ApiResult<Vec<u8>> {
        if let Some(ref text) = self.text {
            Ok(text.as_bytes().to_vec())
        } else if let Some(ref hex) = self.hex {
            hex_decode(hex)
        } else if let Some(ref base64) = self.base64 {
            base64_decode(base64)
        } else {
            Err(ApiError::crypto_convert_input_data_missing())
        }
    }
}

impl OutputEncoding {
    pub fn encode(&self, output: Vec<u8>) -> ApiResult<String> {
        match self {
            OutputEncoding::Text => Ok(String::from_utf8(output)
                .map_err(|err| ApiError::crypto_convert_output_can_not_be_encoded_to_utf8(err))?),
            OutputEncoding::Hex => Ok(hex::encode(output)),
            OutputEncoding::HexUppercase => Ok(hex::encode_upper(output)),
            OutputEncoding::Base64 => Ok(base64::encode(&output))
        }
    }
}


//------------------------------------------------------------------------------------------------------

pub(crate) fn method_api(name: &str) -> api_doc::api::Method {
    api_doc::api::Method {
        name: name.into(),
        summary: None,
        description: None,
        params: vec![],
        result: api_doc::api::Type::None,
        errors: None,
    }
}

pub(crate) fn account_encode(value: &MsgAddressInt) -> String {
    value.to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum AccountAddressType {
    AccountId,
    Hex,
    Base64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Base64AddressParams {
    pub url: bool,
    pub test: bool,
    pub bounce: bool,
}

pub(crate) fn account_encode_ex(
    value: &MsgAddressInt,
    addr_type: AccountAddressType,
    base64_params: Option<Base64AddressParams>,
) -> ApiResult<String> {
    match addr_type {
        AccountAddressType::AccountId => Ok(value.get_address().to_hex_string()),
        AccountAddressType::Hex => Ok(value.to_string()),
        AccountAddressType::Base64 => {
            let params = base64_params.ok_or(ApiError::contracts_address_conversion_failed(
                "No base64 address parameters provided".to_owned()))?;
            encode_base64(value, params.bounce, params.test, params.url)
        }
    }
}

pub(crate) fn account_decode(string: &str) -> ApiResult<MsgAddressInt> {
    match MsgAddressInt::from_str(string) {
        Ok(address) => Ok(address),
        Err(_) if string.len() == 48 => {
            decode_std_base64(string)
        }
        Err(err) => Err(ApiError::crypto_invalid_address(err, string))
    }
}

fn decode_std_base64(data: &str) -> ApiResult<MsgAddressInt> {
    // conversion from base64url
    let data = data.replace('_', "/").replace('-', "+");

    let vec = base64::decode(&data)
        .map_err(|err| ApiError::crypto_invalid_address(err, &data))?;

    // check CRC and address tag
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(&vec[..34]);

    if crc.get_crc_vec_be() != &vec[34..36] || vec[0] & 0x3f != 0x11 {
        return Err(ApiError::crypto_invalid_address("CRC mismatch", &data).into());
    };

    MsgAddressInt::with_standart(None, vec[1] as i8, vec[2..34].into())
        .map_err(|err| ApiError::crypto_invalid_address(err, &data).into())
}

fn encode_base64(address: &MsgAddressInt, bounceable: bool, test: bool, as_url: bool) -> ApiResult<String> {
    if let MsgAddressInt::AddrStd(address) = address {
        let mut tag = if bounceable { 0x11 } else { 0x51 };
        if test { tag |= 0x80 };
        let mut vec = vec![tag];
        vec.extend_from_slice(&address.workchain_id.to_be_bytes());
        vec.append(&mut address.address.get_bytestring(0));

        let mut crc = crc_any::CRC::crc16xmodem();
        crc.digest(&vec);
        vec.extend_from_slice(&crc.get_crc_vec_be());

        let result = base64::encode(&vec);

        if as_url {
            Ok(result.replace('/', "_").replace('+', "-"))
        } else {
            Ok(result)
        }
    } else {
        Err(ApiError::crypto_invalid_address("Non-std address", &address.to_string()).into())
    }
}

pub(crate) fn hex_decode(hex: &String) -> ApiResult<Vec<u8>> {
    if hex.starts_with("x") || hex.starts_with("X") {
        hex_decode(&hex.chars().skip(1).collect())
    } else if hex.starts_with("0x") || hex.starts_with("0X") {
        hex_decode(&hex.chars().skip(2).collect())
    } else {
        hex::decode(hex).map_err(|err| ApiError::crypto_invalid_hex(&hex, err))
    }
}

pub(crate) fn base64_decode(base64: &String) -> ApiResult<Vec<u8>> {
    base64::decode(base64).map_err(|err| ApiError::crypto_invalid_base64(&base64, err))
}

pub(crate) fn long_num_to_json_string(num: u64) -> String {
    format!("0x{:x}", num)
}

pub(crate) fn default_output_encoding_hex() -> OutputEncoding {
    OutputEncoding::Hex
}
