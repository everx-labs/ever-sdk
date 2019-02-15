use std::marker::PhantomData;
use std::io::Cursor;

use tonlabs_sdk_emulator::cells_serialization::deserialize_cells_tree;
use tonlabs_sdk_emulator::stack::SliceData;
use tonlabs_sdk_emulator::types::Exception as InnerBagOfCellsDeserializationException;

use types::{
    ABIOutParameter,
    DeserializationError as InnerTypeDeserializationError
};

pub struct ABIResponse<TOut: ABIOutParameter> {
    output: PhantomData<TOut>,
}

#[derive(Debug)]
pub enum Exception {
    BagOfCellsDeserializationError(InnerBagOfCellsDeserializationException),
    TooManyRootCells,
    EmptyResponse,
    TypeDeserializationError(InnerTypeDeserializationError),
    IncompleteDeserializationError
}

impl<TOut: ABIOutParameter> ABIResponse<TOut> {
    pub fn decode_response(response: &Vec<u8>) -> Result<TOut::Out, Exception> {
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
                TOut::read_from(SliceData::from(root_cell))
                    .map_err(|e| Exception::TypeDeserializationError(e))
                    .and_then(|(result, remainder)| {
                        if remainder.remaining_references() != 0 ||
                            remainder.remaining_bits() != 0 
                        {
                            return Err(Exception::IncompleteDeserializationError);
                        }
                        Ok(result)
                    })
            })
    }
}
