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

// _ (## 8) = Byte;
// _ Byte = Char;
//
// _ Cell = Remainder;
// _ Remainder = StrCont;

use std::convert::TryInto;
use ton_block::{Deserializable, Serializable};
use ton_types::{BuilderData, Cell, IBitstring, SliceData};

pub const MAX_UINT7: usize = 127; // 2 ** 7 - 1

#[derive(Debug, failure::Fail)]
pub enum DeserializationError {
    #[fail(display = "unexpected tlb tag")]
    UnexpectedTLBTag,
}

// small_str#_ len:(## 7) string:(len * [ Char ]) = SmallStr;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmallStr {
    pub string: String,
}

#[derive(Debug, Eq, PartialEq, failure::Fail)]
pub enum SmallStrError {
    #[fail(display = "string length must be <= 127")]
    TooLarge,
}

impl Serializable for SmallStr {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let str_bytes = self.string.as_bytes();
        let str_bytes_len = str_bytes.len();

        if str_bytes_len > MAX_UINT7 {
            return Err(SmallStrError::TooLarge.into());
        }

        builder.append_bits(str_bytes_len, 7)?;
        builder.append_raw(str_bytes, str_bytes_len * 8)?;

        Ok(())
    }
}

impl Deserializable for SmallStr {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let str_bytes_len = slice.get_bits(0, 7)?;
        slice.move_by(7)?;

        let str_bytes = slice.get_next_bytes(str_bytes_len.into())?;
        self.string = String::from_utf8(str_bytes)?;

        Ok(())
    }
}

// version#_ commit:bits160 semantic:StrCont = Version;
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub commit: [u8; 20],
    pub semantic: String,
}

impl Version {
    pub fn new(commit: [u8; 20], semantic: String) -> Self {
        Self { commit, semantic }
    }
}

impl Serializable for Version {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let semantic_bytes = self.semantic.as_bytes();

        builder.append_raw(self.commit.as_slice(), 20 * 8)?;
        builder.append_raw(semantic_bytes, semantic_bytes.len() * 8)?;

        Ok(())
    }
}

impl Deserializable for Version {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        self.commit = slice.get_next_bytes(20)?.try_into().unwrap();
        self.semantic = String::from_utf8(slice.remaining_data().data().to_vec())?;

        Ok(())
    }
}

// metadata#_ sold:^Version linker:^Version
//         compiled_at:uint64 name:SmallStr
//         desc:StrCont = Metadata;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    pub sold: Version,
    pub linker: Version,
    pub compiled_at: u64,
    pub name: SmallStr,
    pub desc: String,
}

impl Serializable for Metadata {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let desc_bytes = self.desc.as_bytes();

        builder.checked_append_reference(self.sold.serialize()?)?;
        builder.checked_append_reference(self.linker.serialize()?)?;
        builder.append_u64(self.compiled_at)?;
        builder.append_builder(&self.name.write_to_new_cell()?)?;
        builder.append_raw(desc_bytes, desc_bytes.len() * 8)?;

        Ok(())
    }
}

impl Deserializable for Metadata {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        self.sold = Version::construct_from_cell(slice.reference(0)?)?;
        self.linker = Version::construct_from_cell(slice.reference(1)?)?;
        self.compiled_at = slice.get_next_u64()?;

        let mut name = SmallStr::default();
        name.read_from(slice)?;

        self.name = name;
        self.desc = String::from_utf8(slice.remaining_data().data().to_vec())?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TvcFrst {
    pub code: Cell,
    pub meta: Option<Metadata>,
}

impl TvcFrst {
    pub fn new(code: Cell, meta: Option<Metadata>) -> Self {
        Self { code, meta }
    }
}

// tvc_none#fa90fdb2 = TvmSmc;
// tvc_frst#b96aa11b code:^Cell meta:(Maybe ^Metadata) = TvmSmc;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum TvmSmc {
    #[default]
    None,
    TvcFrst(TvcFrst),
}

impl TvmSmc {
    const TVC_NONE_TAG: u32 = 0xfa90fdb2;
    const TVC_FRST_TAG: u32 = 0xb96aa11b;

    fn tvc_frst_from_slice(slice: &mut SliceData) -> ton_types::Result<Self> {
        let code = Cell::construct_from_cell(slice.reference(0)?)?;

        let meta = if slice.get_next_bit()? {
            Some(Metadata::construct_from_cell(slice.reference(1)?)?)
        } else {
            None
        };

        Ok(Self::TvcFrst(TvcFrst::new(code, meta)))
    }
}

impl Serializable for TvmSmc {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        if let TvmSmc::None = self {
            builder.append_u32(Self::TVC_NONE_TAG)?;
            return Ok(());
        }

        if let TvmSmc::TvcFrst(tvc_frst) = self {
            builder.append_u32(Self::TVC_FRST_TAG)?;
            builder.checked_append_reference(tvc_frst.code.serialize()?)?;

            if let Some(meta) = &tvc_frst.meta {
                builder.append_bit_one()?;
                builder.checked_append_reference(meta.serialize()?)?;
            } else {
                builder.append_bit_zero()?;
            }

            return Ok(());
        }

        unreachable!()
    }
}

impl Deserializable for TvmSmc {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let tag = slice.get_next_u32()?;

        *self = match tag {
            Self::TVC_NONE_TAG => Self::None,
            Self::TVC_FRST_TAG => Self::tvc_frst_from_slice(slice)?,
            _ => return Err(DeserializationError::UnexpectedTLBTag.into()),
        };

        Ok(())
    }
}

// tvc#8b5f2433 tvc:TvmSmc = TVC;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TVC {
    pub tvc: TvmSmc,
}

impl TVC {
    const TLB_TAG: u32 = 0x8b5f2433;
}

impl Serializable for TVC {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        builder.append_u32(Self::TLB_TAG)?;
        builder.append_builder(&self.tvc.write_to_new_cell()?)?;

        Ok(())
    }
}

impl Deserializable for TVC {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let tag = slice.get_next_u32()?;
        if tag != Self::TLB_TAG {
            return Err(DeserializationError::UnexpectedTLBTag.into());
        }

        self.tvc = TvmSmc::construct_from(slice)?;

        Ok(())
    }
}
