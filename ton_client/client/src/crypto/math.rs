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

use crate::crypto;
use crate::error::{ApiResult, ApiError};
use num_bigint::BigInt;
use rand::RngCore;
use crate::client::ClientContext;
use std::fmt::Display;
use crate::encoding::base64_decode;

//----------------------------------------------------------------------------------- modular_power

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfModularPower {
    /// `base` argument of calculation.
    pub base: String,
    /// `exponent` argument of calculation.
    pub exponent: String,
    /// `modulus` argument of calculation.
    pub modulus: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfModularPower {
    /// result of modular exponentiation
    pub modular_power: String,
}

#[doc(summary = "Modular exponentiation")]
/// Performs modular exponentiation for big integers (`base`^`exponent` mod `modulus`).
/// See [https://en.wikipedia.org/wiki/Modular_exponentiation]
#[api_function]
pub fn modular_power(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfModularPower,
) -> ApiResult<ResultOfModularPower> {
    let base = parse_big_int(&params.base)?;
    let exp = parse_big_int(&params.exponent)?;
    let modulus = parse_big_int(&params.modulus)?;
    let modular_power = base.modpow(&exp, &modulus);
    Ok(ResultOfModularPower {
        modular_power: modular_power.to_str_radix(16)
    })
}

fn parse_big_int(hex: &str) -> ApiResult<BigInt> {
    BigInt::parse_bytes(hex.as_bytes(), 16)
        .ok_or(crypto::Error::invalid_big_int(&hex.to_string()))
}


//--------------------------------------------------------------------------------------- factorize

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfFactorize {
    /// Hexadecimal representation of u64 composite number.
    pub composite: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfFactorize {
    /// Two factors of composite or empty if composite can't be factorized.
    pub factors: [String; 2],
}

#[doc(summary = "Integer factorization")]
/// Performs prime factorization – decomposition of a composite number
/// into a product of smaller prime integers (factors).
/// See [https://en.wikipedia.org/wiki/Integer_factorization]
#[api_function]
pub fn factorize(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfFactorize,
) -> ApiResult<ResultOfFactorize> {
    fn invalid_composite<E: Display>(composite: &String, err: E) -> ApiError {
        crypto::Error::invalid_factorize_challenge(composite, err)
    }
    let composite = u64::from_str_radix(&params.composite, 16).
        map_err(|err| invalid_composite(&params.composite, err))?;
    if composite == 0 {
        return Err(invalid_composite(&params.composite, "Composite number can not be zero"));
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
            factors: [format!("{:X}", p1), format!("{:X}", p2)],
        })
    } else {
        Err(invalid_composite(&params.composite, "Composite number can't be factorized"))
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

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfTonCrc16 {
    /// Input data for CRC calculation. Encoded with `base64`.
    pub data: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfTonCrc16 {
    /// Calculated CRC for input data.
    pub crc: u16,
}

/// Calculates CRC16 using TON algorithm.
#[api_function]
pub fn ton_crc16(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfTonCrc16,
) -> ApiResult<ResultOfTonCrc16> {
    Ok(ResultOfTonCrc16 {
        crc: crate::crypto::internal::ton_crc16(&(base64_decode(&params.data)?))
    })
}

//--------------------------------------------------------------------------- generate_random_bytes

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfGenerateRandomBytes {
    /// Size of random byte array.
    pub length: usize,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfGenerateRandomBytes {
    /// Generated bytes, encoded with `base64`.
    pub bytes: String,
}

#[doc(summary = "Generates random byte array of the specified length in the spesified encoding")]
#[api_function]
pub fn generate_random_bytes(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfGenerateRandomBytes,
) -> ApiResult<ResultOfGenerateRandomBytes> {
    let mut rng = rand::thread_rng();
    let mut bytes: Vec<u8> = Vec::new();
    bytes.resize(params.length, 0);
    rng.fill_bytes(&mut bytes);
    Ok(ResultOfGenerateRandomBytes {
        bytes: base64::encode(&bytes)
    })
}
