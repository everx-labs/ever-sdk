/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use num_bigint::BigInt;
use types::{ApiResult, ApiError};
use rand::RngCore;

fn parse_big_int(hex: &str) -> ApiResult<BigInt> {
    BigInt::parse_bytes(hex.as_bytes(), 16)
        .ok_or(ApiError::crypto_invalid_big_int(&hex.to_string()))
}

pub fn modular_power(base: &String, exponent: &String, modulus: &String) -> ApiResult<String> {
    let base = parse_big_int(&base)?;
    let exp = parse_big_int(&exponent)?;
    let modulus = parse_big_int(&modulus)?;
    let answer = base.modpow(&exp, &modulus);
    Ok(answer.to_str_radix(16))
}

pub fn factorize(what: u64) -> Vec<u64> {
    let mut it = 0;
    let mut i = 0;
    let mut g: u64 = 0;
    let mut rng = rand::rngs::OsRng::new().unwrap();

    while i < 3 || it < 1000 {
        let mut x = rng.next_u64() % (what - 1) + 1;
        let mut y = x;

        let q = ((rng.next_u64() & 0xF) + 17) % what;
        let lim = 1 << (i + 18);

        for j in 1..lim {
            it += 1;
            let mut a = x;
            let mut b = x;
            let mut c = q;

            while b != 0 {
                if b & 1 != 0 {
                    c += a;
                    if c >= what {
                        c -= what;
                    }
                }

                a += a;

                if a >= what {
                    a -= what;
                }
                b >>= 1;
            }

            x = c;

            let z = if x < y {
                what + x - y
            } else {
                x - y
            };

            g = gcd(z, what);

            if g != 1 {
                break;
            }

            if (j & (j - 1)) == 0 {
                y = x;
            }
        }

        if g > 1 && g < what {
            break;
        }

        i += 1;
    }

    if g > 1 && g < what {
        let mut p1 = g;
        let mut p2 = what / g;
        if p1 > p2 {
            let tmp = p1;
            p1 = p2;
            p2 = tmp;
        }

        vec![p1, p2]
    } else {
        vec![]
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

