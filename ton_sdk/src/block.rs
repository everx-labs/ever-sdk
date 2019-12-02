/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::*;
use futures::stream::Stream;
use ton_block::{Block as TvmBlock, BlockId, BlockProcessingStatus, GenericId};

#[derive(Debug)]
pub struct Block {
    block: TvmBlock,
}

// The struct represents block and allows to access their properties.
#[allow(dead_code)]
impl Block {

    // Asynchronously loads a Block instance or None if block with given id is not exists
    pub fn load(id: BlockId) -> SdkResult<Box<dyn Stream<Item = Option<Block>, Error = SdkError>>> {
        let map = queries_helper::load_record(BLOCKS_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let block: TvmBlock = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                    Ok(Some(Block { block }))
                }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Block's json representation 
    // or null if block with given id is not exists
    pub fn load_json(id: BlockId) -> SdkResult<Box<dyn Stream<Item = String, Error = SdkError>>> {

        let map = queries_helper::load_record(BLOCKS_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Returns block's processing status   
    pub fn status(&self) -> BlockProcessingStatus {
        self.block.status.clone()
    }

    // Returns block's identifier
    pub fn id(&self) -> BlockId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.block.calc_id().unwrap()
    }

    // Returns blockchain's block struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn block(&self) -> &TvmBlock {
         &self.block
    }
}