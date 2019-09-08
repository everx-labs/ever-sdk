use std::marker::PhantomData;
use std::io::Cursor;

use tvm::cells_serialization::deserialize_cells_tree;
use tvm::stack::SliceData;
use tvm::error::TvmError as InnerBagOfCellsDeserializationException;

use types::{
    ABIOutParameter,
    ABIDeserialized,
    DeserializationError as InnerTypeDeserializationError
};
use abi_call::ABI_VERSION;

/// Empty struct for contract answer deserialization
pub struct ABIResponse<TOut: ABIOutParameter> {
    output: PhantomData<TOut>,
}

/// Returning errors during deserialization
#[derive(Debug)]
pub enum Exception {
    BagOfCellsDeserializationError(InnerBagOfCellsDeserializationException),
    TooManyRootCells,
    EmptyResponse,
    TypeDeserializationError(InnerTypeDeserializationError),
    IncompleteDeserializationError,
    WrongVersion(u8)
}

impl<TOut: ABIOutParameter> ABIResponse<TOut> {
    /// Decodes ABI contract answer from `Vec<u8>` into type values
    pub fn decode_response(response: &Vec<u8>) -> Result<(u32, TOut::Out), Exception> {
        let mut cursor = Cursor::new(response);
        deserialize_cells_tree(&mut cursor)
            .map_err(|e| Exception::BagOfCellsDeserializationError(e))
            .and_then(|cells| {
                if cells.len() > 1 {
                    return Err(Exception::TooManyRootCells);
                }
                if cells.len() == 0 {
                    return Err(Exception::EmptyResponse);
                }
                let root_cell = &cells[0];
                Self::decode_response_from_slice(SliceData::from(root_cell))
            })
    }

    /// Decodes ABI contract answer from `SliceData` into type values
    pub fn decode_response_from_slice(response: SliceData) -> Result<(u32, TOut::Out), Exception> {        
        let (version, remainder) = u8::read_from(response)
            .map_err(|e| Exception::TypeDeserializationError(e))?;

        if version != ABI_VERSION { Err(Exception::WrongVersion(version))? }

        let (func_id, remainder) = u32::read_from(remainder)
            .map_err(|e| Exception::TypeDeserializationError(e))?;

        let (out, remainder) = TOut::read_from(remainder)
            .map_err(|e| Exception::TypeDeserializationError(e))?;

        if remainder.remaining_references() != 0 ||
            remainder.remaining_bits() != 0
        {
            Err(Exception::IncompleteDeserializationError)
        } else {
            Ok((func_id, out))
        }
    }
}
