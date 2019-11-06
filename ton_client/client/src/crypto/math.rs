use num_bigint::BigInt;
use types::{ApiResult, ApiError};
use rand::RngCore;

fn parse_big_int(hex: &str) -> ApiResult<BigInt> {
    BigInt::parse_bytes(hex.as_bytes(), 16)
        .ok_or(ApiError::crypto_invalid_big_int(&hex.to_string()))
}

static TON_CRC16_TABLE: [u16; 256] = [
    0x0000u16, 0x1021u16, 0x2042u16, 0x3063u16, 0x4084u16, 0x50a5u16, 0x60c6u16, 0x70e7u16,
    0x8108u16, 0x9129u16, 0xa14au16, 0xb16bu16, 0xc18cu16, 0xd1adu16, 0xe1ceu16, 0xf1efu16,
    0x1231u16, 0x0210u16, 0x3273u16, 0x2252u16, 0x52b5u16, 0x4294u16, 0x72f7u16, 0x62d6u16,
    0x9339u16, 0x8318u16, 0xb37bu16, 0xa35au16, 0xd3bdu16, 0xc39cu16, 0xf3ffu16, 0xe3deu16,
    0x2462u16, 0x3443u16, 0x0420u16, 0x1401u16, 0x64e6u16, 0x74c7u16, 0x44a4u16, 0x5485u16,
    0xa56au16, 0xb54bu16, 0x8528u16, 0x9509u16, 0xe5eeu16, 0xf5cfu16, 0xc5acu16, 0xd58du16,
    0x3653u16, 0x2672u16, 0x1611u16, 0x0630u16, 0x76d7u16, 0x66f6u16, 0x5695u16, 0x46b4u16,
    0xb75bu16, 0xa77au16, 0x9719u16, 0x8738u16, 0xf7dfu16, 0xe7feu16, 0xd79du16, 0xc7bcu16,
    0x48c4u16, 0x58e5u16, 0x6886u16, 0x78a7u16, 0x0840u16, 0x1861u16, 0x2802u16, 0x3823u16,
    0xc9ccu16, 0xd9edu16, 0xe98eu16, 0xf9afu16, 0x8948u16, 0x9969u16, 0xa90au16, 0xb92bu16,
    0x5af5u16, 0x4ad4u16, 0x7ab7u16, 0x6a96u16, 0x1a71u16, 0x0a50u16, 0x3a33u16, 0x2a12u16,
    0xdbfdu16, 0xcbdcu16, 0xfbbfu16, 0xeb9eu16, 0x9b79u16, 0x8b58u16, 0xbb3bu16, 0xab1au16,
    0x6ca6u16, 0x7c87u16, 0x4ce4u16, 0x5cc5u16, 0x2c22u16, 0x3c03u16, 0x0c60u16, 0x1c41u16,
    0xedaeu16, 0xfd8fu16, 0xcdecu16, 0xddcdu16, 0xad2au16, 0xbd0bu16, 0x8d68u16, 0x9d49u16,
    0x7e97u16, 0x6eb6u16, 0x5ed5u16, 0x4ef4u16, 0x3e13u16, 0x2e32u16, 0x1e51u16, 0x0e70u16,
    0xff9fu16, 0xefbeu16, 0xdfddu16, 0xcffcu16, 0xbf1bu16, 0xaf3au16, 0x9f59u16, 0x8f78u16,
    0x9188u16, 0x81a9u16, 0xb1cau16, 0xa1ebu16, 0xd10cu16, 0xc12du16, 0xf14eu16, 0xe16fu16,
    0x1080u16, 0x00a1u16, 0x30c2u16, 0x20e3u16, 0x5004u16, 0x4025u16, 0x7046u16, 0x6067u16,
    0x83b9u16, 0x9398u16, 0xa3fbu16, 0xb3dau16, 0xc33du16, 0xd31cu16, 0xe37fu16, 0xf35eu16,
    0x02b1u16, 0x1290u16, 0x22f3u16, 0x32d2u16, 0x4235u16, 0x5214u16, 0x6277u16, 0x7256u16,
    0xb5eau16, 0xa5cbu16, 0x95a8u16, 0x8589u16, 0xf56eu16, 0xe54fu16, 0xd52cu16, 0xc50du16,
    0x34e2u16, 0x24c3u16, 0x14a0u16, 0x0481u16, 0x7466u16, 0x6447u16, 0x5424u16, 0x4405u16,
    0xa7dbu16, 0xb7fau16, 0x8799u16, 0x97b8u16, 0xe75fu16, 0xf77eu16, 0xc71du16, 0xd73cu16,
    0x26d3u16, 0x36f2u16, 0x0691u16, 0x16b0u16, 0x6657u16, 0x7676u16, 0x4615u16, 0x5634u16,
    0xd94cu16, 0xc96du16, 0xf90eu16, 0xe92fu16, 0x99c8u16, 0x89e9u16, 0xb98au16, 0xa9abu16,
    0x5844u16, 0x4865u16, 0x7806u16, 0x6827u16, 0x18c0u16, 0x08e1u16, 0x3882u16, 0x28a3u16,
    0xcb7du16, 0xdb5cu16, 0xeb3fu16, 0xfb1eu16, 0x8bf9u16, 0x9bd8u16, 0xabbbu16, 0xbb9au16,
    0x4a75u16, 0x5a54u16, 0x6a37u16, 0x7a16u16, 0x0af1u16, 0x1ad0u16, 0x2ab3u16, 0x3a92u16,
    0xfd2eu16, 0xed0fu16, 0xdd6cu16, 0xcd4du16, 0xbdaau16, 0xad8bu16, 0x9de8u16, 0x8dc9u16,
    0x7c26u16, 0x6c07u16, 0x5c64u16, 0x4c45u16, 0x3ca2u16, 0x2c83u16, 0x1ce0u16, 0x0cc1u16,
    0xef1fu16, 0xff3eu16, 0xcf5du16, 0xdf7cu16, 0xaf9bu16, 0xbfbau16, 0x8fd9u16, 0x9ff8u16,
    0x6e17u16, 0x7e36u16, 0x4e55u16, 0x5e74u16, 0x2e93u16, 0x3eb2u16, 0x0ed1u16, 0x1ef0u16
];

pub(crate) fn ton_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for c in data {
        let t: u8 = ((*c as u8) ^ ((crc >> 8) as u8)) & 0xff;
        crc = TON_CRC16_TABLE[t as usize] ^ (crc << 8);
    }
    return crc;
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

