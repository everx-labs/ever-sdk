use std::slice::Iter;

use crate::{error::ClientResult, ClientContext};
use serde_json::Value;
use ton_block::Serializable;
use ton_types::{BuilderData, Cell, IBitstring};

use super::{internal::serialize_cell_to_boc, Error};
use crate::boc::internal::deserialize_cell_from_boc;
use crate::boc::BocCacheType;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Num;
use std::ops::ShlAssign;
use crate::encoding::account_decode;

/// Cell builder operation.
#[derive(Serialize, Deserialize, Clone, ApiType)]
#[serde(tag = "type")]
pub enum BuilderOp {
    /// Append integer to cell data.
    Integer {
        /// Bit size of the value.
        size: u32,
        /// Value:
        /// - `Number` containing integer number. e.g. `123`, `-123`.
        /// - Decimal string. e.g. `"123"`, `"-123"`.
        /// - `0x` prefixed hexadecimal string.
        ///   e.g `0x123`, `0X123`, `-0x123`.
        value: Value,
    },
    /// Append bit string to cell data.
    BitString {
        /// Bit string content using bitstring notation.
        /// See `TON VM specification` 1.0.
        ///
        /// Contains hexadecimal string representation:
        /// - Can end with `_` tag.
        /// - Can be prefixed with `x` or `X`.
        /// - Can be prefixed with `x{` or `X{` and ended with `}`.
        ///
        /// Contains binary string represented as a sequence
        /// of `0` and `1` prefixed with `n` or `N`.
        ///
        /// Examples:
        /// `1AB`, `x1ab`, `X1AB`, `x{1abc}`, `X{1ABC}`
        /// `2D9_`, `x2D9_`, `X2D9_`, `x{2D9_}`, `X{2D9_}`
        /// `n00101101100`, `N00101101100`
        value: String,
    },
    /// Append ref to nested cells.
    Cell {
        /// Nested cell builder.
        builder: Vec<BuilderOp>,
    },
    /// Append ref to nested cell.
    CellBoc {
        /// Nested cell BOC encoded with `base64` or BOC cache key.
        boc: String,
    },
    /// Address.
    Address {
        /// Address in a common `workchain:account` or base64 format.
        address: String,
    }
}

impl Default for BuilderOp {
    fn default() -> Self {
        Self::Integer {
            size: 0,
            value: Value::from(0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfEncodeBoc {
    /// Cell builder operations.
    pub builder: Vec<BuilderOp>,
    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfEncodeBoc {
    /// Encoded cell BOC or BOC cache key.
    pub boc: String,
}

/// Encodes bag of cells (BOC) with builder operations.
/// This method provides the same functionality as Solidity TvmBuilder.
/// Resulting BOC of this method can be passed into 
/// Solidity and C++ contracts as TvmCell type.
#[api_function]
pub async fn encode_boc(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeBoc,
) -> ClientResult<ResultOfEncodeBoc> {
    let mut stack = Vec::<Builder>::new();
    let mut builder = Builder::new(&params.builder);
    loop {
        match builder.build(&context).await? {
            BuildResult::Nested { nested, prev } => {
                stack.push(prev);
                builder = nested;
            }
            BuildResult::Complete(cell) => {
                if let Some(prev) = stack.pop() {
                    builder = prev;
                    builder.result.checked_append_reference(cell).map_err(
                        |err| Error::serialization_error(err, "encoded cell"),
                    )?;
                } else {
                    return Ok(ResultOfEncodeBoc {
                        boc: serialize_cell_to_boc(&context, cell, "encoded cell", params.boc_cache)
                            .await?,
                    });
                }
            }
        }
    }
}

struct Builder<'a> {
    input: Iter<'a, BuilderOp>,
    result: BuilderData,
}

enum BuildResult<'a> {
    Nested {
        nested: Builder<'a>,
        prev: Builder<'a>,
    },
    Complete(Cell),
}

impl<'a> Builder<'a> {
    fn new(builder: &'a Vec<BuilderOp>) -> Self {
        Self {
            input: builder.iter(),
            result: BuilderData::new(),
        }
    }

    /// Append data using operation iterator until the end or the nested cell operation.
    /// Returns resulting cell or nested Builder.
    async fn build(
        mut self,
        context: &std::sync::Arc<ClientContext>,
    ) -> ClientResult<BuildResult<'a>> {
        while let Some(op) = self.input.next() {
            match op {
                BuilderOp::Integer { size, value } => {
                    append_integer(&mut self.result, *size as usize, value)?;
                }
                BuilderOp::BitString { value } => {
                    append_bitstring(&mut self.result, &value)?;
                }
                BuilderOp::CellBoc { boc } => {
                    self.result.checked_append_reference(
                        deserialize_cell_from_boc(context, boc, "CellBoc")
                            .await?
                            .1,
                    ).map_err(
                        |err| Error::serialization_error(err, "encode_boc"),
                    )?;
                }
                BuilderOp::Cell { ref builder } => {
                    return Ok(BuildResult::Nested {
                        nested: Self::new(builder),
                        prev: self,
                    });
                }
                BuilderOp::Address { address } => {
                    account_decode(address)?
                        .write_to(&mut self.result)
                        .map_err(|err| Error::invalid_boc(err))?;
                }
            }
        }
        Ok(BuildResult::Complete(self.result.into_cell().map_err(
            |err| Error::serialization_error(err, "encode_boc"),
        )?))
    }
}

fn append_integer(builder: &mut BuilderData, size: usize, value: &Value) -> ClientResult<()> {
    if let Some(value) = value.as_i64() {
        append_number(
            builder,
            value < 0,
            BigUint::from(value.abs() as u64),
            size,
            "Integer",
        )
    } else if let Some(str) = value.as_str() {
        parse_integer(builder, str, size)
    } else {
        Err(Error::serialization_error(
            "Integer value must be a number or a string representation",
            "builder operations",
        ))
    }
}

/// Appends `size` high bits of the specified `number`.
///
/// If count of the significant bits in `number` is less than `size`
/// then the buffer will be padded with bit `0` for positive
/// or bit `1` for `negative`.
fn append_number(
    builder: &mut BuilderData,
    negative: bool,
    number: BigUint,
    size: usize,
    name: &str,
) -> ClientResult<()> {
    let mut number = number;
    let shift = 8 - (size % 8);
    if shift > 0 && shift < 8 {
        number.shl_assign(shift);
    }
    let mut bytes = if negative {
        BigInt::from_biguint(Sign::Minus, number).to_signed_bytes_be()
    } else {
        number.to_bytes_be()
    };
    let expected_len = (size + 7) / 8;
    while bytes.len() < expected_len {
        bytes.insert(0, if negative { 0xFF } else { 0 });
    }
    while bytes.len() > expected_len {
        bytes.remove(0);
    }
    builder
        .append_raw(&bytes, size)
        .map_err(|err| Error::serialization_error(err, name))?;
    Ok(())
}

/// Append `size` high bits from the number that is represented as a `string`.
fn parse_integer(builder: &mut BuilderData, string: &str, size: usize) -> ClientResult<()> {
    let mut num_str = string.trim();
    let negative = if num_str.starts_with("-") {
        num_str = &num_str[1..];
        true
    } else if num_str.starts_with("+") {
        num_str = &num_str[1..];
        false
    } else {
        false
    };
    let radix = if num_str.starts_with("0x") || num_str.starts_with("0X") {
        num_str = &num_str[2..];
        16
    } else {
        10
    };
    parse_string(builder, num_str, negative, radix, size)
}

/// Parses `string` using `radix` and pass resulting number
/// to the `append_number` (see above).
fn parse_string(
    builder: &mut BuilderData,
    string: &str,
    negative: bool,
    radix: u32,
    size: usize,
) -> ClientResult<()> {
    let unsigned = BigUint::from_str_radix(string, radix)
        .map_err(|err| Error::serialization_error(err, string))?;
    append_number(builder, negative, unsigned, size, string)
}

/// Append bitstring canonical (extended with `n` prefix) representation.
fn append_bitstring(builder: &mut BuilderData, string: &str) -> ClientResult<()> {
    let mut num_str = string.trim();

    // Try parse direct binary form
    if num_str.starts_with("n") || num_str.starts_with("N") {
        parse_string(builder, &num_str[1..], false, 2, num_str.len() - 1)?;
        return Ok(());
    }

    // Escape from decorations
    if num_str.starts_with("x{") || num_str.starts_with("X{") {
        if !num_str.ends_with("}") {
            return Err(Error::serialization_error(
                "Missing terminating `}`",
                string,
            ));
        }
        num_str = &num_str[2..num_str.len() - 1];
    } else if num_str.starts_with("x") || num_str.starts_with("X") {
        num_str = &num_str[1..];
    }

    // Check if there is a tagged representation
    if num_str.ends_with("_") {
        num_str = &num_str[0..num_str.len() - 1];
        // Escape from trailing zeros because of BuilderData doesn't support this
        while num_str.ends_with("0") {
            num_str = &num_str[0..num_str.len() - 1];
        }
        // Check if the bitstring isn't empty
        if num_str != "" && num_str != "8" {
            let mut number = BigUint::from_str_radix(num_str, 16)
                .map_err(|err| Error::serialization_error(err, string))?;
            // If hex string has an odd len, we need to pad it with zero bits at the end
            if (num_str.len() & 1) != 0 {
                number.shl_assign(4);
            }
            let bytes = number.to_bytes_be();
            builder
                .append_bitstring(&bytes)
                .map_err(|err| Error::serialization_error(err, string))?;
        }
    } else {
        // Here is an untagged representation
        let size = num_str.len() * 4;
        parse_string(builder, num_str, false, 16, size)?;
    }
    Ok(())
}
