use super::{
    Bit,
};

use std::cmp;
use std::fmt;
use std::ops::{Add, Range, RangeBounds};
use std::ops::Bound::{Excluded, Included, Unbounded};
use byteorder::{BigEndian, WriteBytesExt};
use tvm::types::AccountId;

struct Bits {
    pub data: Vec<Bit>,
}

#[derive(Default, PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
pub struct Bitstring {
    data: Vec<u8>,
    length_in_bits: usize,
}

impl From<Bits> for u8 {
    fn from(bits: Bits) -> u8 {
        if bits.data.len() > 8 {
            panic!("Can not fit into u8.");
        } else {
            let mut result = 0;
            for bit in bits.data.iter() {
                result = (result << 1) | match bit {
                    Bit::One => 1,
                    Bit::Zero => 0,
                };
            }
            return result;
        }
    }
}

impl From<Bits> for usize {
    fn from(bits: Bits) -> usize {
        if bits.data.len() > 32 {
            panic!("Can not fit into usize.");
        } else {
            let mut result = 0;
            for bit in bits.data.iter() {
                result = (result << 1) | match bit {
                    Bit::One => 1,
                    Bit::Zero => 0,
                };
            }
            return result;
        }
    }
}

impl fmt::Debug for Bit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Bit::One => write!(f, "1"),
            Bit::Zero => write!(f, "0"),
        }
    }
}

impl fmt::Display for Bitstring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let trailing_byte_display = if self.length_in_bits % 8 == 0 {
            vec![]
        } else {
            let trailing_bits = self.length_in_bits % 8;
            let trailing_bits_display: String = format!(
                "[{:?}]",
                self.bits(self.length_in_bits - trailing_bits..self.length_in_bits)
                    .data
                    .iter()
                    .map(|bit| match bit {
                        Bit::One => 1,
                        Bit::Zero => 0,
                    })
                    .collect::<Vec<_>>()
            );

            vec![trailing_bits_display]
        };
        write!(
            f,
            "x{}_ (length: {})",
            (0..(self.length_in_bits / 8))
                .map(|i| format!("{:02x}", &self.data[i]))
                .chain(trailing_byte_display)
                .collect::<Vec<_>>()
                .join("."),
            self.length_in_bits
        )
    }
}

impl Bitstring {

    pub fn new() -> Bitstring {
        Bitstring::create(Vec::<u8>::new(), 0)
    }

    pub fn length_in_bits(&self) -> usize {
        self.length_in_bits
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.length_in_bits = 0;
    }

    fn bits(&self, range: Range<usize>) -> Bits {
        if range.end > self.length_in_bits {
            panic!("Range is out of bounds.")
        }

        return Bits {
            data: range
                .map(|i| {
                    if i >= self.length_in_bits {
                        if i == self.length_in_bits {
                            Bit::One
                        } else {
                            Bit::Zero
                        }
                    } else {
                        let mut mask = 1;
                        if i % 8 != 7 {
                            mask <<= 7 - (i % 8);
                        }
                        if self.data[i / 8] & mask != 0 {
                            Bit::One
                        } else {
                            Bit::Zero
                        }
                    }
                })
                .collect(),
        };
    }

    pub fn create(mut data: Vec<u8>, length_in_bits: usize) -> Bitstring {
        let shift = length_in_bits % 8;
        let new_size = (length_in_bits + 7) >> 3;
        data.truncate(new_size);
        if shift != 0 {
            let last_bits = data.pop().unwrap() >> (8 - shift);
            data.push(last_bits << (8 - shift));
        }

        Bitstring {
            data: data,
            length_in_bits: length_in_bits,
        }
    }

    /// Returns specified range of bits as new Bitstring
    pub fn substring<TRange>(&self, range: TRange) -> Bitstring
    where
        TRange: RangeBounds<usize>
    {
        let mut result = Bitstring::new();

        let range_start = match range.start_bound() {
            Unbounded => 0usize,
            Included(start) => *start,
            Excluded(start) => start + 1,
        };

        let range_end = match range.end_bound() {
            Unbounded => self.length_in_bits(),
            Included(end) => end + 1,
            Excluded(end) => *end,
        };

        self.bits(range_start..range_end)
            .data
            .iter()
            .for_each(|x| {
                result.append_bit(x);
        });

        result
    }

    pub fn from_bitstring_with_completion_tag(data: Vec<u8>) -> Bitstring {
        let length = Bitstring::find_last_bit(&data);
        Bitstring::create(data, length)
    }

    pub fn into_bitstring_with_completion_tag(&self, destination: &mut Vec<u8>) {
        if self.length_in_bits == 0 {
            destination.push(0x80);
            return;
        }
        let complete_bytes = self.length_in_bits / 8;
        if complete_bytes > 0 {
            destination.extend_from_slice(&self.data[0..complete_bytes]);
        }
        let mut trailing_bits = self.bits(8 * complete_bytes..self.length_in_bits);
        trailing_bits.data.push(Bit::One);
        while trailing_bits.data.len() < 8 {
            trailing_bits.data.push(Bit::Zero);
        }
        destination.push(trailing_bits.into());
    }

    pub fn append(&mut self, bitstring: &Bitstring) -> &mut Bitstring {
        let shift = self.length_in_bits % 8;
        if shift == 0 {
            self.data.truncate(self.length_in_bits / 8);
            self.data.extend(&bitstring.data);
        } else {
            self.data.truncate(1 + self.length_in_bits / 8);
            let last_bits = self.data.pop().unwrap() >> (8 - shift);
            let mut y: u16 = last_bits.into();
            for x in bitstring.data.iter() {
                y = (y << 8) | (*x as u16);
                self.data.push((y >> shift) as u8);
            }
            self.data.push((y << (8 - shift)) as u8);
        }
        self.length_in_bits += bitstring.length_in_bits;
        if self.length_in_bits % 8 == 0 {
            self.data.truncate(self.length_in_bits / 8);
        } else {
            self.data.truncate(self.length_in_bits / 8 + 1);
        }
        self
    }

    pub fn append_bit_zero(&mut self) -> &mut Bitstring {
        self.append_bit(&Bit::Zero)
    }
    pub fn append_bit_one(&mut self) -> &mut Bitstring {
        self.append_bit(&Bit::One)
    }
    pub fn append_bit_bool(&mut self, bit: bool) -> &mut Bitstring {
        self.append_bit(if bit {&Bit::One} else {&Bit::Zero})
    }
    pub fn append_bit(&mut self, bit: &Bit) -> &mut Bitstring {
        let shift = self.length_in_bits % 8;
        let mask = 1u8 << (7 - shift);
        let element_index = self.length_in_bits / 8;
        while self.data.len() <= element_index {
            self.data.push(0);
        }
        let mut element = self.data[element_index];
        element = match bit {
            &Bit::One => element | mask,
            &Bit::Zero => element & (!mask),
        };
        self.data[element_index] = element;
        self.length_in_bits += 1;
        self
    }

    ///
    /// is bitstring empty
    /// 
    pub fn is_empty(&self) -> bool {
        self.length_in_bits() == 0
    }

    ///
    /// Get first bit as bool
    /// 
    pub fn first_bit(&self) -> Option<bool> {
        if self.is_empty() {
            None
        } else {
            Some((self.data().first().unwrap() & 0b10000000) == 0b10000000)
        }
    }

    ///
    /// Erase first bit of Bitstring and return it
    /// 
    pub fn erase_first_bit(&mut self) -> Option<usize> {
        if self.length_in_bits == 0 {
            None
        } else {
            let bit = self.data[0] >> 7;
            let len = self.data.len();
            for i in 1..len {
                self.data[i - 1] = self.data[i - 1] << 1 | self.data[i] >> 7;
            }
            self.length_in_bits -= 1;
            if self.length_in_bits % 8 == 0 {
                self.data.pop().unwrap();
            } else {
                self.data[len - 1] <<= 1;
            }
            Some(bit as usize)
        }
    }

    ///
    /// Erase prefix of Bitstring
    /// 
    pub fn erase_prefix(&mut self, prefix: &Bitstring) -> bool {
        if self.is_empty() || self.length_in_bits() < prefix.length_in_bits() {
            false
        } else if prefix.is_empty() {
            true
        } else if *self == *prefix {
            self.clear();
            true
        } else {
            match Bitstring::common_prefix(&self, prefix) {
                (_, _, Some(_)) => false, // prefix should be fully in self
                (_, Some(remainder), _) => {
                    *self = remainder;
                    true
                }
                (_, None, _) => unreachable!("strings are the same"),
            }
        }
    }

    //TODO: append_Vec<u8> for performance

    pub fn append_u8(&mut self, value: u8) -> &mut Bitstring {
        self.append(&Bitstring::create(vec![value], 8))
    }

    pub fn append_u16(&mut self, value: u16) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_u16::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 16))
    }

    pub fn append_u32(&mut self, value: u32) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_u32::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 32))
    }

    pub fn append_u64(&mut self, value: u64) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_u64::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 64))
    }

    pub fn append_u128(&mut self, value: u128) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_u128::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 128))
    }

    pub fn append_i8(&mut self, value: i8) -> &mut Bitstring {
        self.append(&Bitstring::create(vec![value as u8], 8))
    }

    pub fn append_i16(&mut self, value: i16) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_i16::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 16))
    }

    pub fn append_i32(&mut self, value: i32) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_i32::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 32))
    }

    pub fn append_i64(&mut self, value: i64) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_i64::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 64))
    }

    pub fn append_i128(&mut self, value: i128) -> &mut Bitstring {
        let mut vec = vec![];
        vec.write_i128::<BigEndian>(value).unwrap();
        self.append(&Bitstring::create(vec, 128))
    }

    /// writes low bits of integer
    pub fn append_bits(&mut self, value: usize, bits: usize) -> &mut Bitstring {
        let mut vec = vec![];
        match bits {
            0 => (),
            1..=7 => vec.push((value as u8) << (8 - bits)),
            8..=15 => vec.write_u16::<BigEndian>((value as u16) << (16 - bits)).unwrap(),
            16..=31 => vec.write_u32::<BigEndian>((value as u32) << (32 - bits)).unwrap(),
            32..=63 => vec.write_u64::<BigEndian>((value as u64) << (64 - bits)).unwrap(),
            bits @ _ => unimplemented!("bits: {}", bits)
        }
        self.append(&Bitstring::create(vec, bits))
    }

    pub fn common_prefix(
        a: &Bitstring,
        b: &Bitstring,
    ) -> (Option<Bitstring>, Option<Bitstring>, Option<Bitstring>) {
        let max_possible_prefix_length_in_bits = cmp::min(a.length_in_bits, b.length_in_bits);
        let mut buffer: Vec<u8> = Vec::with_capacity(1 + max_possible_prefix_length_in_bits / 8);
        for i in 0..max_possible_prefix_length_in_bits / 8 {
            if a.data[i] == b.data[i] {
                buffer.push(a.data[i]);
            } else {
                break;
            }
        }
        let n = 8 * buffer.len();
        let mut prefix_bitstring = Bitstring::create(buffer, n);
        let tail_byte_bits_range = Range {
            start: n,
            end: cmp::min(max_possible_prefix_length_in_bits, 8 + n),
        };
        a.bits(tail_byte_bits_range.clone())
            .data
            .iter()
            .zip(b.bits(tail_byte_bits_range).data.iter())
            .take_while(|(x, y)| x == y)
            .for_each(|(x, _)| { prefix_bitstring.append_bit(x); });
        let mut rem_a = Bitstring::new();
        let mut rem_b = Bitstring::new();
        if prefix_bitstring.length_in_bits < a.length_in_bits {
            a.bits(prefix_bitstring.length_in_bits .. a.length_in_bits).data.iter()
            .for_each(|x| { rem_a.append_bit(x); });
        }
        if prefix_bitstring.length_in_bits < b.length_in_bits {
            b.bits(prefix_bitstring.length_in_bits .. b.length_in_bits).data.iter()
            .for_each(|x| { rem_b.append_bit(x); });
        }
        return (
            if prefix_bitstring.length_in_bits > 0 { Some(prefix_bitstring) } else { None },
            if rem_a.length_in_bits > 0 { Some(rem_a) } else { None },
            if rem_b.length_in_bits > 0 { Some(rem_b) } else { None },
        );
    }

    pub fn find_last_bit(bitstring: &Vec<u8>) -> usize {
        let mut last_bit: usize = bitstring.len() * 8;
        for x in bitstring.iter().rev() {
            if *x == 0 {
                last_bit -= 8;
            } else {
                let mut skip = 1;
                let mut mask = 1;
                while (*x & mask) == 0 {
                    skip += 1;
                    mask <<= 1
                }
                last_bit -= skip;
                break;
            }
        }
        return last_bit;
    }

    /// Splits bitstring into two. Truncates self to [size] bits and returns remaining bitstring.
    pub fn split_off(&mut self, size: usize) -> (Bitstring) {
        #[inline]
        fn bits_to_bytes(bits: usize) -> usize {
            (bits + 7) >> 3
        }

        assert!(self.length_in_bits >= size);

        let suffix_bitlen = self.length_in_bits - size;
        let suffix_bytelen = bits_to_bytes(suffix_bitlen);

        let mut suffix = self.data.split_off(size >> 3);

        let excess_bits = size & 0b111;
        if excess_bits != 0 {
            let shift = 8 - excess_bits;
            let last_bits = *suffix.get(0).unwrap() >> shift << shift;
            self.data.push(last_bits);
            for i in 0..suffix_bytelen {
                suffix[i] <<= shift;
            }
        }

        Bitstring::create(suffix, suffix_bitlen)
    }
}

impl Add for Bitstring {
    type Output = Bitstring;
    fn add(self, other: Bitstring) -> Bitstring {
        let mut bitstring = self;
        bitstring.append(&other);
        bitstring
    }
}

impl From<Vec<u8>> for Bitstring {
    fn from(value: Vec<u8>) -> Bitstring {
        let len = value.len() * 8;
        Bitstring::create(value, len)
    }
}

macro_rules! bitstring_from_integer {
    ($inner_type:ty) => {
        impl From<$inner_type> for Bitstring {
            fn from(value: $inner_type) -> Bitstring {
                let vec = value.to_be_bytes().to_vec();
                let len = vec.len() * 8;
                Bitstring::create(vec, len)
            }
        }
    };
}

bitstring_from_integer!(u8);
bitstring_from_integer!(u16);
bitstring_from_integer!(u32);
bitstring_from_integer!(u64);
bitstring_from_integer!(u128);
bitstring_from_integer!(i8);
bitstring_from_integer!(i16);
bitstring_from_integer!(i32);
bitstring_from_integer!(i64);
bitstring_from_integer!(i128);

impl From<Bit> for Bitstring {
    fn from(value: Bit) -> Bitstring {
        Bitstring::new().append_bit(&value).to_owned()
    }
}

impl From<AccountId> for Bitstring
{
    fn from(value: AccountId) -> Bitstring {
        let vec = value.get_bytestring(0);
        let len = vec.len() * 8;
        Bitstring::create(vec, len)
    }
}

impl From<Bitstring> for AccountId
{
    fn from(value: Bitstring) -> AccountId {
        let len = value.data().len();
        if len >= 32 {
            AccountId::from(&value.data()[0..32])
        } else {
            AccountId::from(&value.data()[..])
        }
    }
}
