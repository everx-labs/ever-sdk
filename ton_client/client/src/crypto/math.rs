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

use crate::types::{ApiResult, ApiError, InputData, OutputEncoding};
use num_bigint::BigInt;
use rand::RngCore;
use serde::{Deserializer, Serializer};
use crate::client::ClientContext;
use crate::serialization::{default_output_encoding_base64};

//----------------------------------------------------------------------------------- modular_power

#[derive(Deserialize)]
pub struct ParamsOfModularPower {
    /// `base` argument of calculation.
    pub base: String,
    /// `exponent` argument of calculation.
    pub exponent: String,
    /// `modulus` argument of calculation.
    pub modulus: String,
}

#[derive(Serialize)]
pub struct ResultOfModularPower {
    pub modular_power: String,
}

/// Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`).
/// See [https://en.wikipedia.org/wiki/Modular_exponentiation]
pub fn modular_power(
    _context: &mut ClientContext,
    params: ParamsOfModularPower,
) -> ApiResult<ResultOfModularPower> {
    Ok(ResultOfModularPower {
        modular_power: base.modpow(&params.exponent, &params.modulus)
    })
}

//--------------------------------------------------------------------------------------- factorize

#[derive(Deserialize)]
pub struct ParamsOfFactorize {
    /// Hexadecimal representation of u64 composite number.
    pub composite: String,
}

#[derive(Serialize)]
pub struct ResultOfFactorize {
    /// Two products of composite or empty if composite can't be factorized.
    pub products: [String; 2],
}

/// Performs prime factorization – decomposition of a composite number
/// into a product of smaller prime integers.
/// See [https://en.wikipedia.org/wiki/Integer_factorization]
pub fn factorize(
    _context: &mut ClientContext,
    params: ParamsOfFactorize,
) -> ApiResult<ResultOfFactorize> {
    let invalid_composite = |err|
        Err(ApiError::crypto_invalid_factorize_challenge(&params.composite, err));
    let composite = u64::from_str_radix(&params.composite, 16).
        map_err(|err| invalid_composite(err))?;
    if composite == 0 {
        return invalid_composite("Composite number can not be zero");
    }

    let mut it = 0;
    let mut i = 0;
    let mut g: u64 = 0;
    let mut rng = rand::thread_rng();

    while i < 3 || it < 1000 {
        let mut x = rng.next_u64() % (composite - 1) + 1;
        let mut y = x;

        let q = ((rng.next_u64() & 0xF) + 17) % composite;
        let lim = 1 << (i + 18);

        for j in 1..lim {
            it += 1;
            let mut a = x;
            let mut b = x;
            let mut c = q;

            while b != 0 {
                if b & 1 != 0 {
                    c += a;
                    if c >= composite {
                        c -= composite;
                    }
                }

                a += a;

                if a >= composite {
                    a -= composite;
                }
                b >>= 1;
            }

            x = c;

            let z = if x < y {
                composite + x - y
            } else {
                x - y
            };

            g = gcd(z, composite);

            if g != 1 {
                break;
            }

            if (j & (j - 1)) == 0 {
                y = x;
            }
        }

        if g > 1 && g < composite {
            break;
        }

        i += 1;
    }

    if g > 1 && g < composite {
        let mut p1 = g;
        let mut p2 = composite / g;
        if p1 > p2 {
            let tmp = p1;
            p1 = p2;
            p2 = tmp;
        }
        Ok(ResultOfFactorize {
            products: [format!("{:X}", p1), format!("{:X}", p2)],
        })
    } else {
        invalid_composite("Composite number can't be factorized");
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while a != 0 && b != 0 {
        while (b & 1) == 0 {
            b = b.clone() >> 1;
        }

        while (a & 1) == 0 {
            a = a.clone() >> 1;
        }

        if a > b {
            a = a.clone() - b;
        } else {
            b = b.clone() - a;
        }
    }

    if b == 0 {
        a
    } else {
        b
    }
}

//------------------------------------------------------------------- ton_crc16

#[derive(Deserialize)]
pub struct ParamsOfTonCrc16 {
    /// Input data for CRC calculation.
    data: InputData,
}

#[derive(Serialize)]
pub struct ResultOfTonCrc16 {
    /// Calculated CRC for input data.
    crc: u16,
}

/// Calculates CRC16 using TON algorithm.
pub fn ton_crc16(
    _context: &mut ClientContext,
    params: ParamsOfTonCrc16,
) -> ApiResult<ResultOfTonCrc16> {
    Ok(ResultOfTonCrc16 {
        crc: internal_ton_crc16(&(params.data.decode()?))
    })
}

pub(crate) fn internal_ton_crc16(data: &[u8]) -> u16 {
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(data);
    crc.get_crc() as u16
}

//--------------------------------------------------------------------------- generate_random_bytes

#[derive(Deserialize)]
pub struct ParamsOfGenerateRandomBytes {
    /// Size of random byte array.
    length: u32,
    /// Encoding of generated bytes. Default is `base64`.
    #[serde(default = "default_output_encoding_base64")]
    output_encoding: OutputEncoding,
}

#[derive(Serialize)]
pub struct ResultOfGenerateRandomBytes {
    /// Generated bytes, encoded into string using `output_encoding` parameter.
    bytes: String,
}

/// Generates random byte array of specified length.
pub fn generate_random_bytes(
    _context: &mut ClientContext,
    params: ParamsOfGenerateRandomBytes,
) -> ApiResult<ResultOfGenerateRandomBytes> {
    let mut rng = rand::thread_rng();
    let mut bytes: Vec<u8> = Vec::new();
    bytes.resize(len, 0);
    rng.fill_bytes(&mut bytes);

    Ok(ResultOfGenerateRandomBytes {
        bytes: params.output_encoding.encode(bytes)?
    })
}
