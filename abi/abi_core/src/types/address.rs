use super::{
    ABISerialized,
    ABIDeserialized,
    DeserializationError
};

use tvm::stack::{BuilderData, SliceData};
use tvm::block::{MsgAddress, Serializable};

impl ABISerialized for MsgAddress {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        // TODO: write somehow builder to builder let builder = self.write_to_new_cell().unwrap();
        destination
    }

    fn get_in_cell_size(&self) -> usize {
        self.write_to_new_cell().unwrap().length_in_bits()
    }
}

impl ABIDeserialized for MsgAddress {
    type Out = MsgAddress;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        // let mut cursor = Reader::new(cursor);
        // match cursor.read_next::<(bool, bool)>()? {
        //     (true, false) => {
        //         let size = cursor.read_next::<u8>()?;
        //         let mut cursor = cursor.remainder();
        //         let result = MsgAddress::construct_from::<MsgAddress>(&mut cursor).unwrap();
        //         Ok((result, cursor))
        //     }
        //     _ => Err(DeserializationError::with(cursor.remainder()))
        // }

        // TODO: Somehow bits to SliceData and to MsgAddress
        Ok((MsgAddress::AddrNone, cursor))
    }
}
