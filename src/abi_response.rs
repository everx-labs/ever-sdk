use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::marker::PhantomData;
use std::sync::Arc;
use tonlabs_sdk_emulator::bitstring::Bitstring;
use tonlabs_sdk_emulator::cells_serialization::BagOfCells;
use tonlabs_sdk_emulator::stack::{BuilderData, CellData, SliceData};
use tonlabs_sdk_emulator::types::Exception as InnerBagOfCellsDeserializationException;
use types::common::prepend_data_to_chain;
use types::{
    ABIParameter, 
    DeserializationError as InnerTypeDeserializationError
};

pub struct ABIResponse<TOut: ABIOutParameter> {
    output: PhantomData<TOut>,
}

pub enum Exception {
    BagOfCellsDeserializationError(InnerBagOfCellsDeserializationException),
    TooManyRootCells,
    EmptyResponse,
    TypeDeserializationError(InnerTypeDeserializationError),
    IncompleteDeserializationError
}

impl<TOut: ABIOutParameter> ABICall<TOut> {
    pub fn decode_response<T>(response: &Vec<u8>) -> Result<TOut, Exception> {
        let mut cursor = Cursor::new(response);
        deserialize_cells_tree(&mut cursor)
            .and_then(|cells| {
                if cells.len() > 1 {
                    return Err(Exception::TooManyRootCells); 
                }
                if cells.len() == 0 {
                    return Err(Exception::EmptyResponse); 
                }
                let root_cell = cells[0];
                TOut::read_from(SliceData::from(root_cell))
                    .map_err(|e| Exception::TypeDeserializationError(e))
                    .and_then(|(result, remainder)| {
                        if remainder.remaining_references() != 0 ||
                            remainder.remaining_bits() != 0 
                        {
                            return Err(Exception::IncompleteDeserializationError);
                        }
                        Ok(result)
                    });
            });

        let builder = BuilderData::new();
        let builder = parameters.prepend_to(builder);
        let builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut bitstring = Bitstring::new();

            bitstring.append_u8(ABI_VERSION);
            for chunk in Self::get_function_id(fn_name).iter() {
                bitstring.append_u8(*chunk);
            }
            bitstring
        });

        // serialize tree into Vec<u8>
        let root_cell = Arc::<CellData>::from(&builder);
        let root = SliceData::from(root_cell);

        let mut data = Vec::new();
        BagOfCells::with_root(root)
            .write_to(&mut data, false, 2, 2)
            .unwrap();

        data
    }
}
