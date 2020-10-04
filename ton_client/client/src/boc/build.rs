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
 *
 */

use crate::boc::internal::deserialize_cell_from_base64;
use crate::boc::Error;
use crate::error::ApiResult;
use ton_block::{Account, Serializable};
use ton_block::{
    AccountState, AccountStorage, AccountStuff, CurrencyCollection, MsgAddressInt,
    StateInit, StateInitLib, StorageInfo, StorageUsed,
};

pub struct ParamsOfBuildAccount {
    pub code: String,
    pub data: String,
    pub balance: Option<u64>,
    pub last_trans_lt: Option<u64>,
    pub last_paid: Option<u32>,
}

pub struct ResultOfBuildAccount {
    pub account: String,
}

pub fn build_account(params: ParamsOfBuildAccount) -> ApiResult<ResultOfBuildAccount> {
    let account = Account::Account(AccountStuff {
        addr: MsgAddressInt::default(),
        storage: AccountStorage {
            balance: CurrencyCollection::from(params.balance.unwrap_or(100000000000)),
            last_trans_lt: params.last_trans_lt.unwrap_or(0),
            state: AccountState::AccountActive(StateInit {
                code: Some(deserialize_cell_from_base64(&params.code, "account code")?.1),
                data: Some(deserialize_cell_from_base64(&params.data, "account data")?.1),
                library: StateInitLib::default(),
                special: None,
                split_depth: None,
            }),
        },
        storage_stat: StorageInfo {
            due_payment: None,
            last_paid: params.last_paid.unwrap_or(0),
            used: StorageUsed::default(),
        },
    });
    let cell = account
        .serialize()
        .map_err(|err| Error::serialization_error(err, "account"))?;
    Ok(ResultOfBuildAccount {
        account: base64::encode(
            &ton_types::cells_serialization::serialize_toc(&cell)
                .map_err(|err| Error::serialization_error(err, "account"))?,
        ),
    })
}
