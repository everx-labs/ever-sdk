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

#![allow(dead_code)]

use crate::client;
use crate::error::ClientResult;
use num_bigint::BigInt;
use num_traits::cast::NumCast;
use std::str::FromStr;
use ton_block::MsgAddressInt;
use ton_types::SliceData;

//------------------------------------------------------------------------------------------------------

pub(crate) fn account_encode(value: &MsgAddressInt) -> String {
    value.to_string()
}

#[derive(Serialize, Deserialize, Debug, ApiType, Clone, PartialEq, Eq)]
pub enum AccountAddressType {
    AccountId,
    Hex,
    Base64,
}

#[derive(Serialize, Deserialize, Debug, ApiType, Default, Clone)]
pub struct Base64AddressParams {
    pub url: bool,
    pub test: bool,
    pub bounce: bool,
}

pub(crate) fn account_encode_ex(
    value: &MsgAddressInt,
    addr_type: AccountAddressType,
    base64_params: Option<Base64AddressParams>,
) -> ClientResult<String> {
    match addr_type {
        AccountAddressType::AccountId => Ok(format!("{:x}", value.get_address())),
        AccountAddressType::Hex => Ok(value.to_string()),
        AccountAddressType::Base64 => {
            let params =
                base64_params.ok_or(client::Error::contracts_address_conversion_failed(
                    "No base64 address parameters provided",
                ))?;
            encode_base64(value, params.bounce, params.test, params.url)
        }
    }
}

pub(crate) fn account_decode(string: &str) -> ClientResult<MsgAddressInt> {
    match MsgAddressInt::from_str(string) {
        Ok(address) => Ok(address),
        Err(_) if string.len() == 48 => decode_std_base64(string),
        Err(err) => Err(client::Error::invalid_address(err, string)),
    }
}

pub(crate) fn decode_std_base64(data: &str) -> ClientResult<MsgAddressInt> {
    // conversion from base64url
    let data = data.replace('_', "/").replace('-', "+");

    let vec = base64::decode(&data).map_err(|err| client::Error::invalid_address(err, &data))?;

    // check CRC and address tag
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(&vec[..34]);

    if crc.get_crc_vec_be() != &vec[34..36] || vec[0] & 0x3f != 0x11 {
        return Err(client::Error::invalid_address("CRC mismatch", &data).into());
    };

    MsgAddressInt::with_standart(
        None,
        vec[1] as i8,
        SliceData::from_raw(vec[2..34].to_vec(), 256),
    )
    .map_err(|err| client::Error::invalid_address(err, &data).into())
}

fn encode_base64(
    address: &MsgAddressInt,
    bounceable: bool,
    test: bool,
    as_url: bool,
) -> ClientResult<String> {
    if let MsgAddressInt::AddrStd(address) = address {
        let mut tag = if bounceable { 0x11 } else { 0x51 };
        if test {
            tag |= 0x80
        };
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
        Err(client::Error::invalid_address("Non-std address", &address.to_string()).into())
    }
}

pub(crate) fn hex_decode(hex: &str) -> ClientResult<Vec<u8>> {
    if hex.starts_with("x") || hex.starts_with("X") {
        hex_decode(&hex[1..])
    } else if hex.starts_with("0x") || hex.starts_with("0X") {
        hex_decode(&hex[2..])
    } else {
        hex::decode(hex).map_err(|err| client::Error::invalid_hex(&hex, err))
    }
}

pub(crate) fn base64_decode(base64: &str) -> ClientResult<Vec<u8>> {
    base64::decode(base64).map_err(|err| client::Error::invalid_base64(base64, err))
}

pub(crate) fn long_num_to_json_string(num: u64) -> String {
    format!("0x{:x}", num)
}

pub fn decode_abi_bigint(string: &str) -> ClientResult<BigInt> {
    let result = if string.starts_with("-0x") || string.starts_with("-0X") {
        BigInt::parse_bytes(&string[3..].as_bytes(), 16).map(|number| -number)
    } else if string.starts_with("0x") || string.starts_with("0X") {
        BigInt::parse_bytes(&string[2..].as_bytes(), 16)
    } else {
        BigInt::parse_bytes(string.as_bytes(), 10)
    };

    result.ok_or(client::Error::can_not_parse_number(string))
}

pub fn decode_abi_number<N: NumCast>(string: &str) -> ClientResult<N> {
    let bigint = decode_abi_bigint(string)?;
    NumCast::from(bigint).ok_or(client::Error::can_not_parse_number(string))
}
