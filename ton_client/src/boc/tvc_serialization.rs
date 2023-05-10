/*
    Copyright 2023 EverX Labs.

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use ton_block::{Deserializable, Serializable};
use ton_types::{BuilderData, Cell, HashmapE, HashmapType, IBitstring, Result, SliceData};

#[derive(Debug, failure::Fail)]
pub enum DeserializationError {
    #[fail(display = "unexpected tlb tag")]
    UnexpectedTLBTag,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct TVC {
    pub code: Option<Cell>,
    pub desc: Option<String>,
}

impl TVC {
    const TVC_TAG: u32 = 0x0167f70c;

    #[cfg(test)]
    pub fn new(code: Option<Cell>, desc: Option<String>) -> Self {
        Self { code, desc }
    }
}

pub fn str_to_hashmap(input: &str) -> Result<Cell> {
    let mut input = input.as_bytes().chunks(127).enumerate();

    let mut dict = HashmapE::with_bit_len(16);

    while let Some((i, c)) = input.next() {
        let mut cb = BuilderData::new();
        cb.append_raw(c, c.len() * 8)?;

        let mut b = BuilderData::new();
        b.checked_append_reference(cb.into_cell()?)?;

        let mut ib = BuilderData::new();
        ib.append_u16(i as u16)?;

        dict.set_builder(SliceData::load_builder(ib.clone())?, &b)?;
    }

    dict.write_to_new_cell()?.into_cell()
}

pub fn hashmap_to_str(dict: &HashmapE) -> Result<String> {
    let mut result = Vec::new();

    for i in dict.iter() {
        let (_, v) = i.unwrap();
        let cs = SliceData::load_cell(v.reference(0)?)?;
        result.append(&mut cs.get_bytestring(0));
    }

    Ok(String::from_utf8(result)?)
}

impl Serializable for TVC {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        builder.append_u32(Self::TVC_TAG)?;

        if let Some(c) = &self.code {
            builder.append_bit_one()?;
            builder.checked_append_reference(c.to_owned())?;
        } else {
            builder.append_bit_zero()?;
        }

        if let Some(d) = &self.desc {
            let dict = str_to_hashmap(d.as_str())?;
            let dict = BuilderData::from_cell(&dict)?;
            builder.append_builder(&dict)?;
        } else {
            builder.append_bit_zero()?;
        }

        Ok(())
    }
}

impl Deserializable for TVC {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let tag = slice.get_next_u32()?;
        if tag != Self::TVC_TAG {
            return Err(DeserializationError::UnexpectedTLBTag.into());
        }

        let mut ref_count = 0;

        if slice.get_next_bit()? {
            self.code = Some(slice.reference(ref_count)?);
            ref_count += 1;
        }

        if slice.get_next_bit()? {
            let data = Some(slice.reference(ref_count)?);
            let dict = HashmapE::with_hashmap(16, data);
            self.desc = Some(hashmap_to_str(&dict)?);
        }

        Ok(())
    }
}
