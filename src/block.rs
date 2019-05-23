use crate::*;
use futures::stream::Stream;
use ton_block::{BlockId, BlockProcessingStatus};

const BLOCKS_TABLE_NAME: &str = "blocks";

#[derive(Debug)]
pub struct Block {
    block: ton_block::Block,
}

#[allow(dead_code)]
impl Block {

    pub fn load(id: BlockId) -> SdkResult<Box<Stream<Item = Block, Error = SdkError>>> {
        let map = db_helper::load_record(BLOCKS_TABLE_NAME, &id_to_string(&id))?
            .and_then(|val| {
                let block: ton_block::Block = serde_json::from_value(val)
                    .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                Ok(Block { block })
            });

        Ok(Box::new(map))
    }

    pub fn load_json(id: BlockId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(BLOCKS_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn status(&self) -> BlockProcessingStatus {
        self.block.status.clone()
    }

    pub fn id(&self) -> BlockId {
        self.block.id.clone()
    }
}