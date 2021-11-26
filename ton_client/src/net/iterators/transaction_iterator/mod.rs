/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use std::collections::HashSet;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::iterators::block::{BlockFields, BLOCK_TRANSACTIONS_FIELDS};
use crate::net::iterators::block_iterator::BlockIterator;
use crate::net::iterators::transaction::{TransactionFields, TRANSACTION_FIELDS};
use crate::net::iterators::{query_by_ids, register_iterator, ResultOfIteratorNext};
use crate::net::{ChainIterator, ParamsOfCreateBlockIterator, RegisteredIterator};

#[derive(Serialize, Deserialize)]
pub(crate) struct ResumeState {
    blocks: crate::net::iterators::block_iterator::ResumeState,
    result_fields: String,
    include_transfers: bool,
    next: Vec<String>,
}

impl ResumeState {}

pub(crate) struct TransactionIterator {
    blocks: BlockIterator,
    accounts_filter: HashSet<String>,
    result_fields: String,
    include_transfers: bool,
    next: Vec<Value>,
}

impl TransactionIterator {
    pub async fn new(
        context: &Arc<ClientContext>,
        params: ParamsOfCreateTransactionIterator,
    ) -> ClientResult<Self> {
        let blocks = BlockIterator::new(
            context,
            ParamsOfCreateBlockIterator {
                start_time: params.start_time,
                end_time: params.end_time,
                result: Some(BLOCK_TRANSACTIONS_FIELDS.to_string()),
                shard_filter: params.shard_filter,
            },
        )
        .await?;
        Ok(Self {
            blocks,
            accounts_filter: params
                .accounts_filter
                .map(|x| x.iter().cloned().collect())
                .unwrap_or(Default::default()),
            result_fields: params.result.unwrap_or(String::default()),
            include_transfers: params.include_transfers.unwrap_or(false),
            next: Vec::new(),
        })
    }

    pub fn get_resume_state(&self) -> ResumeState {
        ResumeState {
            blocks: self.blocks.get_resume_state(),
            next: self
                .next
                .iter()
                .map(|x| x["id"].as_str().unwrap_or("").to_string())
                .collect(),
            result_fields: self.result_fields.clone(),
            include_transfers: self.include_transfers,
        }
    }

    pub fn get_resume_state_value(&self) -> ClientResult<Value> {
        serde_json::to_value(self.get_resume_state()).map_err(|e| {
            crate::client::Error::internal_error(format!(
                "Can't serialize iterator resume state: {}",
                e
            ))
        })
    }

    pub async fn from_resume_state(
        context: &Arc<ClientContext>,
        resume: ResumeState,
        accounts_filter: Option<Vec<String>>,
    ) -> ClientResult<Self> {
        let blocks = BlockIterator::from_resume_state(context, resume.blocks).await?;
        let next = Self::query_transactions(context, resume.next, &resume.result_fields).await?;
        Ok(Self {
            blocks,
            accounts_filter: accounts_filter
                .map(|x| x.iter().cloned().collect())
                .unwrap_or(Default::default()),
            result_fields: resume.result_fields,
            include_transfers: resume.include_transfers,
            next,
        })
    }

    pub async fn resume(
        context: &Arc<ClientContext>,
        params: ParamsOfResumeTransactionIterator,
    ) -> ClientResult<Self> {
        let resume = ResumeState::deserialize(&params.resume_state).map_err(|e| {
            crate::client::Error::internal_error(format!("Invalid iterator resume state: {}", e))
        })?;
        Self::from_resume_state(context, resume, params.accounts_filter).await
    }

    async fn query_transactions(
        context: &Arc<ClientContext>,
        transaction_ids: Vec<String>,
        fields: &str,
    ) -> ClientResult<Vec<Value>> {
        query_by_ids(
            context,
            "transactions",
            transaction_ids,
            &format!("{} {}", TRANSACTION_FIELDS, fields),
        )
        .await
    }

    fn wanted(&self, account_addr: &str) -> bool {
        self.accounts_filter.is_empty() || self.accounts_filter.contains(account_addr)
    }

    async fn query_next(&mut self, context: &Arc<ClientContext>) -> ClientResult<()> {
        if !self.blocks.state.has_more() {
            self.next = Vec::new();
            return Ok(());
        }
        let mut transaction_ids = Vec::new();
        let mut blocks = self.blocks.clone();

        while transaction_ids.is_empty() {
            let next_blocks = blocks.next(context, 1, false).await?;
            if next_blocks.items.is_empty() && !next_blocks.has_more {
                break;
            }
            for block in next_blocks.items {
                if let Some(account_blocks) = BlockFields(&block).account_blocks() {
                    for account_block in account_blocks {
                        if self.wanted(account_block.account_addr()) {
                            if let Some(transactions) = account_block.transactions() {
                                for tr in transactions {
                                    transaction_ids.push(tr.transaction_id().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        self.next = Self::query_transactions(context, transaction_ids, &self.result_fields).await?;
        self.blocks = blocks;
        Ok(())
    }
}

const BOUNCE_TYPE_OK: u32 = 2;

fn get_transfers(transaction: TransactionFields) -> Value {
    let mut transfers = Vec::new();
    let is_bounced = transaction
        .bounce()
        .map(|x| x.bounce_type() == BOUNCE_TYPE_OK)
        .unwrap_or(false);
    if let Some(inbound) = transaction.in_message() {
        if u64::from_str_radix(inbound.value(), 10).unwrap_or(0) > 0 {
            transfers.push(json!({
                "message": inbound.id(),
                "isBounced": is_bounced,
                "isDeposit": true,
                "counterparty": inbound.src(),
                "value": inbound.value(),
            }));
        }
    }
    if let Some(out_messages) = transaction.out_messages() {
        for outbound in out_messages {
            if u64::from_str_radix(outbound.value(), 10).unwrap_or(0) > 0 {
                transfers.push(json!({
                    "message": outbound.id(),
                    "isBounced": is_bounced,
                    "isDeposit": false,
                    "counterparty": outbound.dst(),
                    "value": outbound.value(),
                }));
            }
        }
    }
    Value::Array(transfers)
}

#[async_trait::async_trait]
impl ChainIterator for TransactionIterator {
    async fn next(
        &mut self,
        context: &Arc<ClientContext>,
        limit: u32,
        return_resume_state: bool,
    ) -> ClientResult<ResultOfIteratorNext> {
        let limit = limit.max(1) as usize;

        if self.next.is_empty() {
            self.query_next(context).await?;
        }

        let mut items = Vec::new();
        while items.len() < limit && !self.next.is_empty() {
            let mut transaction = self.next.remove(0);
            if self.include_transfers {
                transaction["transfers"] = get_transfers(TransactionFields(&transaction))
            }
            items.push(transaction);
        }

        let has_more = !self.next.is_empty() || self.blocks.state.has_more();

        let resume_state = if return_resume_state {
            Some(self.get_resume_state_value()?)
        } else {
            None
        };

        Ok(ResultOfIteratorNext {
            has_more,
            items,
            resume_state,
        })
    }

    fn after_remove(&mut self, _context: &Arc<ClientContext>) {}
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfCreateTransactionIterator {
    /// Starting time to iterate from.
    ///
    /// If the application specifies this parameter then the iteration
    /// includes blocks with `gen_utime` >= `start_time`.
    /// Otherwise the iteration starts from zero state.
    ///
    /// Must be specified in seconds.
    pub start_time: Option<u32>,

    /// Optional end time to iterate for.
    ///
    /// If the application specifies this parameter then the iteration
    /// includes blocks with `gen_utime` < `end_time`.
    /// Otherwise the iteration never stops.
    ///
    /// Must be specified in seconds.
    pub end_time: Option<u32>,

    /// Shard prefix filters.
    ///
    /// If the application specifies this parameter and it is not an empty array
    /// then the iteration will include items related to accounts that belongs to
    /// the specified shard prefixes.
    /// Shard prefix must be represented as a string "workchain:prefix".
    /// Where `workchain` is a signed integer and the `prefix` if a hexadecimal
    /// representation if the 64-bit unsigned integer with tagged shard prefix.
    /// For example: "0:3800000000000000".
    /// Account address conforms to the shard filter if
    /// it belongs to the filter workchain and the first bits of address match to
    /// the shard prefix. Only transactions with suitable account addresses are iterated.
    pub shard_filter: Option<Vec<String>>,

    /// Account address filter.
    ///
    /// Application can specify the list of accounts for which
    /// it wants to iterate transactions.
    ///
    /// If this parameter is missing or an empty list then the library iterates
    /// transactions for all accounts that pass the shard filter.
    ///
    /// Note that the library doesn't detect conflicts between the account filter and the shard filter
    /// if both are specified.
    /// So it is an application responsibility to specify the correct filter combination.
    pub accounts_filter: Option<Vec<String>>,

    /// Projection (result) string.
    ///
    /// List of the fields that must be returned for iterated items.
    /// This field is the same as the `result` parameter of
    /// the `query_collection` function.
    /// Note that iterated items can contain additional fields that are
    /// not requested in the `result`.
    pub result: Option<String>,

    /// Include `transfers` field in iterated transactions.
    ///
    /// If this parameter is `true` then each transaction contains field
    /// `transfers` with list of transfer. See more about this structure in function description.
    pub include_transfers: Option<bool>,
}

/// Creates transaction iterator.
///
/// Transaction iterator uses robust iteration methods that guaranty that every
/// transaction in the specified range isn't missed or iterated twice.
///
/// Iterated range can be reduced with some filters:
/// - `start_time` – the bottom time range. Only transactions with `now`
/// more or equal to this value are iterated. If this parameter is omitted then there is
/// no bottom time edge, so all the transactions since zero state are iterated.
/// - `end_time` – the upper time range. Only transactions with `now`
/// less then this value are iterated. If this parameter is omitted then there is
/// no upper time edge, so iterator never finishes.
/// - `shard_filter` – workchains and shard prefixes that reduce the set of interesting
/// accounts. Account address conforms to the shard filter if
/// it belongs to the filter workchain and the first bits of address match to
/// the shard prefix. Only transactions with suitable account addresses are iterated.
/// - `accounts_filter` – set of account addresses whose transactions must be iterated.
/// Note that accounts filter can conflict with shard filter so application must combine
/// these filters carefully.
///
/// Iterated item is a JSON objects with transaction data. The minimal set of returned
/// fields is:
/// ```text
/// id
/// account_addr
/// now
/// balance_delta(format:DEC)
/// bounce { bounce_type }
/// in_message {
///     id
///     value(format:DEC)
///     msg_type
///     src
/// }
/// out_messages {
///     id
///     value(format:DEC)
///     msg_type
///     dst
/// }
/// ```
/// Application can request an additional fields in the `result` parameter.
///
/// Another parameter that affects on the returned fields is the `include_transfers`.
/// When this parameter is `true` the iterator computes and adds `transfer` field containing
/// list of the useful `TransactionTransfer` objects.
/// Each transfer is calculated from the particular message related to the transaction
/// and has the following structure:
/// - message – source message identifier.
/// - isBounced – indicates that the transaction is bounced, which means the value will be returned back to the sender.
/// - isDeposit – indicates that this transfer is the deposit (true) or withdraw (false).
/// - counterparty – account address of the transfer source or destination depending on `isDeposit`.
/// - value – amount of nano tokens transferred. The value is represented as a decimal string
/// because the actual value can be more precise than the JSON number can represent. Application
/// must use this string carefully – conversion to number can follow to loose of precision.
///
/// Application should call the `remove_iterator` when iterator is no longer required.
#[api_function]
pub async fn create_transaction_iterator(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfCreateTransactionIterator,
) -> ClientResult<RegisteredIterator> {
    register_iterator(
        &context,
        Box::new(TransactionIterator::new(&context, params).await?),
    )
    .await
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfResumeTransactionIterator {
    /// Iterator state from which to resume.
    ///
    /// Same as value returned from `iterator_next`.
    pub resume_state: Value,

    /// Account address filter.
    ///
    /// Application can specify the list of accounts for which
    /// it wants to iterate transactions.
    ///
    /// If this parameter is missing or an empty list then the library iterates
    /// transactions for all accounts that passes the shard filter.
    ///
    /// Note that the library doesn't detect conflicts between the account filter and the shard filter
    /// if both are specified.
    /// So it is the application's responsibility to specify the correct filter combination.
    pub accounts_filter: Option<Vec<String>>,
}

/// Resumes transaction iterator.
///
/// The iterator stays exactly at the same position where the `resume_state` was caught.
/// Note that `resume_state` doesn't store the account filter. If the application requires
/// to use the same account filter as it was when the iterator was created then the application
/// must pass the account filter again in `accounts_filter` parameter.
///
/// Application should call the `remove_iterator` when iterator is no longer required.
#[api_function]
pub async fn resume_transaction_iterator(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfResumeTransactionIterator,
) -> ClientResult<RegisteredIterator> {
    register_iterator(
        &context,
        Box::new(TransactionIterator::resume(&context, params).await?),
    )
    .await
}
