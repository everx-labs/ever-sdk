use tonlabs_sdk_emulator::stack::SliceData;
use std::option::Option;
use super::{
    ABIParameter,
    DeserializationError
};

pub struct Reader {
    cursor: Option<SliceData>
}

impl Reader {
    pub fn new(cursor: SliceData) -> Reader {
        Reader { cursor: Some(cursor) }
    }

    pub fn read_next<T>(&mut self) -> Result<T, DeserializationError> 
    where
        T: ABIParameter
    {
        let cursor = self.cursor.take().unwrap();
        let (result, next) = T::read_from(cursor)?;
        self.cursor = Some(next);
        Ok(result)
    }

    pub fn is_empty(&self) -> bool {
        self.cursor
            .as_ref()
            .map_or_else(
                || true,
                |e| {
                    e.remaining_references() == 0
                    && e.remaining_bits() == 0
                }
            )
    }

    pub fn remainder(self) -> SliceData {
        self.cursor.unwrap()
    }
}
