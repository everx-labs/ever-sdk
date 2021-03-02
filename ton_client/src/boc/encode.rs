use std::slice::Iter;

use crate::{error::ClientResult, ClientContext};
use serde_json::Value;
use ton_types::{BuilderData, Cell, IBitstring};

use super::{internal::serialize_cell_to_boc, Error};
use crate::boc::BocCacheType;

/// Cell builder operation.
#[derive(Serialize, Deserialize, Clone, ApiType)]
pub enum BuilderOp {
    /// Append integer to cell data.
    Integer {
        /// Bit size of the value.
        size: u8,
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
    /// Append ref to nested cells
    Cell {
        /// Nested cell builder
        builder: Vec<BuilderOp>,
    },
    /// Append ref to nested cell
    CellBoc {
        /// Nested cell BOC
        boc: String,
    },
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
    /// BOC root hash encoded with hex
    pub boc: String,
}

struct Builder<'a> {
    result: BuilderData,
    input: Iter<'a, BuilderOp>,
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
            result: BuilderData::new(),
            input: builder.iter(),
        }
    }
    fn build(mut self) -> ClientResult<BuildResult<'a>> {
        while let Some(op) = self.input.next() {
            match op {
                BuilderOp::Integer { size, value } => {
                    self.result.append_u8(1);
                }
                BuilderOp::BitString { value } => {}
                BuilderOp::CellBoc { boc } => {}
                BuilderOp::Cell { ref builder } => {
                    return Ok(BuildResult::Nested {
                        nested: Self::new(builder),
                        prev: self,
                    });
                }
            }
        }
        Ok(BuildResult::Complete(self.result.into_cell().map_err(
            |err| Error::serialization_error(err, "encode_boc"),
        )?))
    }
}

/// Calculates BOC root hash
#[api_function]
pub async fn encode_boc(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeBoc,
) -> ClientResult<ResultOfEncodeBoc> {
    let mut stack = Vec::<Builder>::new();
    let mut current = Builder::new(&params.builder);
    loop {
        match current.build()? {
            BuildResult::Nested { nested, prev } => {
                stack.push(prev);
                current = nested;
            }
            BuildResult::Complete(cell) => {
                if let Some(prev) = stack.pop() {
                    current = prev;
                    current.result.append_reference_cell(cell);
                } else {
                    return Ok(ResultOfEncodeBoc {
                        boc: serialize_cell_to_boc(&context, cell, "encode_bic", params.boc_cache)
                            .await?,
                    });
                }
            }
        }
    }
}
