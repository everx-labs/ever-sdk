use crate::*;
use futures::stream::Stream;
use ton_block::{BlockId, BlockProcessingStatus};

const BLOCKS_TABLE_NAME: &str = "blocks";

#[derive(Debug)]
pub struct Block {
    block: ton_block::Block,
}

// The struct represents block and allows to access their properties.
#[allow(dead_code)]
impl Block {

    // Asynchronously loads a Block instance or None if block with given id is not exists
    pub fn load(id: BlockId) -> SdkResult<Box<Stream<Item = Option<Block>, Error = SdkError>>> {
        let map = db_helper::load_record(BLOCKS_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let block: ton_block::Block = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                    Ok(Some(Block { block }))
                }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Block's json representation 
    // or null if block with given id is not exists
    pub fn load_json(id: BlockId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(BLOCKS_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Returns block's processing status   
    pub fn status(&self) -> BlockProcessingStatus {
        self.block.status.clone()
    }

    // Returns block's identifier
    pub fn id(&self) -> BlockId {
        self.block.id.clone()
    }

    // Returns blockchain's block struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn block(&self) -> &ton_block::Block {
         &self.block
    }
}