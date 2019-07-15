use super::{
    DeserializationError,
    ABITypeSignature,
    ABIDeserialized,
    ABISerialized
};

use tvm::stack::{BuilderData, SliceData, IBitstring};

impl ABISerialized for BuilderData {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let mut data = self.clone();
        data.prepend_builder(&destination).unwrap();
        data
    }

    fn get_in_cell_size(&self) -> usize {
        self.cell().data().to_vec().get_in_cell_size()
    }
}

impl ABIDeserialized for BuilderData {
    type Out = BuilderData;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (bits, cursor) = <Vec<bool> as ABIDeserialized>::read_from(cursor)?;
        
        let mut result = BuilderData::new();
        bits.iter()
            .for_each(|x| {
                result.append_bit_bool(x.clone()).unwrap();
        });

        Ok((result, cursor))
    }
}