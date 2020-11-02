/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::{MessageId, TransactionId};
use crate::contract::ShardDescr;
use crate::types::BlockId;

#[derive(Deserialize, Debug, Clone)]
pub struct MsgDescr {
    pub msg_id: Option<MessageId>,
    pub transaction_id: Option<TransactionId>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub gen_utime: u32,
    pub after_split: bool,
    #[serde(flatten)]
    pub shard_descr: ShardDescr,
    pub in_msg_descr: Vec<MsgDescr>
}

