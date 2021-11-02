use std::array::IntoIter;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use ton_block::{Block, Transaction};
use ton_block_json::{BlockSerializationSet, TransactionSerializationSet};
use ton_types::{Result, UInt256};

use crate::error::ClientResult;
use crate::proofs::errors::Error;

lazy_static! {
    static ref BLOCK_IGNORE_FIELDS: HashSet<&'static str> = IntoIter::new([
        "chain_order",
        "gen_software_capabilities",
    ]).collect();

    static ref TRANSACTION_IGNORE_FIELDS: HashSet<&'static str> = IntoIter::new([
        "chain_order",
    ]).collect();

    static ref BLOCK_NUMERIC_FIELDS: HashSet<&'static str> = IntoIter::new([
        "account_blocks.transactions.lt",
        "account_blocks.transactions.total_fees",
        "account_blocks.transactions.total_fees_other.value",
        "end_lt",
        "gen_software_capabilities",
        "in_msg_descr.fwd_fee",
        "in_msg_descr.ihr_fee",
        "in_msg_descr.in_msg.fwd_fee_remaining",
        "in_msg_descr.out_msg.fwd_fee_remaining",
        "in_msg_descr.transit_fee",
        "master.config.p14.basechain_block_fee",
        "master.config.p14.masterchain_block_fee",
        "master.config.p17.max_stake",
        "master.config.p17.min_stake",
        "master.config.p17.min_total_stake",
        "master.config.p18.bit_price_ps",
        "master.config.p18.cell_price_ps",
        "master.config.p18.mc_bit_price_ps",
        "master.config.p18.mc_cell_price_ps",
        "master.config.p20.block_gas_limit",
        "master.config.p20.delete_due_limit",
        "master.config.p20.flat_gas_limit",
        "master.config.p20.flat_gas_price",
        "master.config.p20.freeze_due_limit",
        "master.config.p20.gas_credit",
        "master.config.p20.gas_limit",
        "master.config.p20.gas_price",
        "master.config.p20.special_gas_limit",
        "master.config.p21.block_gas_limit",
        "master.config.p21.delete_due_limit",
        "master.config.p21.flat_gas_limit",
        "master.config.p21.flat_gas_price",
        "master.config.p21.freeze_due_limit",
        "master.config.p21.gas_credit",
        "master.config.p21.gas_limit",
        "master.config.p21.gas_price",
        "master.config.p21.special_gas_limit",
        "master.config.p24.bit_price",
        "master.config.p24.cell_price",
        "master.config.p24.lump_price",
        "master.config.p25.bit_price",
        "master.config.p25.cell_price",
        "master.config.p25.lump_price",
        "master.config.p32.list.weight",
        "master.config.p32.total_weight",
        "master.config.p33.list.weight",
        "master.config.p33.total_weight",
        "master.config.p34.list.weight",
        "master.config.p34.total_weight",
        "master.config.p35.list.weight",
        "master.config.p35.total_weight",
        "master.config.p36.list.weight",
        "master.config.p36.total_weight",
        "master.config.p37.list.weight",
        "master.config.p37.total_weight",
        "master.config.p8.capabilities",
        "master.recover_create_msg.fwd_fee",
        "master.recover_create_msg.ihr_fee",
        "master.recover_create_msg.in_msg.fwd_fee_remaining",
        "master.recover_create_msg.out_msg.fwd_fee_remaining",
        "master.recover_create_msg.transit_fee",
        "master.shard_fees.create",
        "master.shard_fees.create_other.value",
        "master.shard_fees.fees",
        "master.shard_fees.fees_other.value",
        "master.shard_hashes.descr.end_lt",
        "master.shard_hashes.descr.fees_collected",
        "master.shard_hashes.descr.fees_collected_other.value",
        "master.shard_hashes.descr.funds_created",
        "master.shard_hashes.descr.funds_created_other.value",
        "master.shard_hashes.descr.start_lt",
        "master_ref.end_lt",
        "out_msg_descr.import_block_lt",
        "out_msg_descr.imported.fwd_fee",
        "out_msg_descr.imported.ihr_fee",
        "out_msg_descr.imported.in_msg.fwd_fee_remaining",
        "out_msg_descr.imported.out_msg.fwd_fee_remaining",
        "out_msg_descr.imported.transit_fee",
        "out_msg_descr.next_addr_pfx",
        "out_msg_descr.out_msg.fwd_fee_remaining",
        "out_msg_descr.reimport.fwd_fee",
        "out_msg_descr.reimport.ihr_fee",
        "out_msg_descr.reimport.in_msg.fwd_fee_remaining",
        "out_msg_descr.reimport.out_msg.fwd_fee_remaining",
        "out_msg_descr.reimport.transit_fee",
        "prev_alt_ref.end_lt",
        "prev_ref.end_lt",
        "prev_vert_alt_ref.end_lt",
        "prev_vert_ref.end_lt",
        "start_lt",
        "value_flow.created",
        "value_flow.created_other.value",
        "value_flow.exported",
        "value_flow.exported_other.value",
        "value_flow.fees_collected",
        "value_flow.fees_collected_other.value",
        "value_flow.fees_imported",
        "value_flow.fees_imported_other.value",
        "value_flow.from_prev_blk",
        "value_flow.from_prev_blk_other.value",
        "value_flow.imported",
        "value_flow.imported_other.value",
        "value_flow.minted",
        "value_flow.minted_other.value",
        "value_flow.to_next_blk",
        "value_flow.to_next_blk_other.value",
    ]).collect();

    static ref TRANSACTION_NUMERIC_FIELDS: HashSet<&'static str> = IntoIter::new([
        "action.total_action_fees",
        "action.total_fwd_fees",
        "balance_delta",
        "balance_delta_other.value",
        "bounce.fwd_fees",
        "bounce.msg_fees",
        "bounce.req_fwd_fees",
        "compute.gas_fees",
        "compute.gas_limit",
        "compute.gas_used",
        "credit.credit",
        "credit.credit_other.value",
        "credit.due_fees_collected",
        "ext_in_msg_fee",
        "lt",
        "prev_trans_lt",
        "storage.storage_fees_collected",
        "storage.storage_fees_due",
        "total_fees",
        "total_fees_other.value",
    ]).collect();
    
    static ref BLOCKS_UNIX_TIME_FIELDS: HashSet<&'static str> = IntoIter::new([
        "gen_utime",
        "master.config.p18.utime_since",
        "master.config.p32.utime_since",
        "master.config.p32.utime_until",
        "master.config.p33.utime_since",
        "master.config.p33.utime_until",
        "master.config.p34.utime_since",
        "master.config.p34.utime_until",
        "master.config.p35.utime_since",
        "master.config.p35.utime_until",
        "master.config.p36.utime_since",
        "master.config.p36.utime_until",
        "master.config.p37.utime_since",
        "master.config.p37.utime_until",
        "master.max_shard_gen_utime",
        "master.min_shard_gen_utime",
        "master.shard_hashes.descr.gen_utime",
    ]).collect();

    static ref TRANSACTIONS_UNIX_TIME_FIELDS: HashSet<&'static str> = IntoIter::new([
        "now",
    ]).collect();
}

pub(crate) enum JsonPath<'a, 'b> {
    InitialEntity(&'static str),
    Field { parent: &'a JsonPath<'a, 'b>, field_name: &'b str },
    Index { parent: &'a JsonPath<'a, 'b>, index: usize },
}

impl<'a, 'b> JsonPath<'a, 'b> {
    pub fn new(initial_entity: &'static str) -> Self {
        Self::InitialEntity(initial_entity)
    }

    fn join_field(&'a self, field_name: &'b str) -> Self {
        JsonPath::Field { parent: self, field_name }
    }

    fn join_index(&'a self, index: usize) -> Self {
        JsonPath::Index { parent: self, index }
    }

    fn gen_flat_str(&self) -> String {
        match self {
            JsonPath::InitialEntity(_) => String::new(),
            JsonPath::Field { parent, field_name} => {
                if let JsonPath::InitialEntity(_) = parent {
                    field_name.to_string()
                } else {
                    format!("{}.{}", parent.gen_flat_str(), field_name)
                }
            }
            JsonPath::Index { parent, .. } => {
                parent.gen_flat_str()
            }
        }
    }

    fn gen_display_str(&self) -> String {
        match self {
            JsonPath::InitialEntity(name) => name.to_string(),
            JsonPath::Field { parent, field_name } => format!("{}.{}", parent, field_name),
            JsonPath::Index { parent, index} => format!("{}[{}]", parent, index),
        }
    }
}

impl Display for JsonPath<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.gen_display_str())
    }
}

pub(crate) fn compare_values(
    actual: &Value,
    expected: &Value,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> ClientResult<()> {
    match (actual, expected) {
        (Value::Null, Value::Null) => return Ok(()),
        (Value::Null, _) => return Ok(()),

        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
        => {
            if actual == expected {
                return Ok(());
            }
        }

        (Value::Number(_), Value::String(_))
            | (Value::String(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
        => {
            let is_numeric = numeric_fields.contains(path.gen_flat_str().as_str());

            if !is_numeric && actual.is_string() && expected.is_string() {
                if actual.as_str().unwrap().eq_ignore_ascii_case(expected.as_str().unwrap()) {
                    return Ok(());
                }
            } else if get_string(actual, is_numeric)
                .eq_ignore_ascii_case(&get_string(expected, is_numeric))
            {
                return Ok(());
            }
        }

        (Value::Array(vec_actual), Value::Array(vec_expected)) =>
            return compare_vectors(vec_actual, vec_expected, path, ignore_fields, numeric_fields),

        (Value::Object(map_actual), Value::Object(map_expected)) =>
            return compare_maps(map_actual, map_expected, path, ignore_fields, numeric_fields),

        _ => (),
    }

    Err(Error::data_differs_from_proven(
        format!(
            "field `{path}`: expected {expected:?}, actual {actual:?}",
            path = path,
            actual = actual,
            expected = expected,
        )
    ))
}

fn compare_maps(
    map_actual: &Map<String, Value>,
    map_expected: &Map<String, Value>,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> ClientResult<()> {
    for key in map_actual.keys()
        .filter(
            |key| !ignore_fields.contains(path.join_field(key).gen_flat_str().as_str())
        )
    {
        compare_values(
            &map_actual[key],
            map_expected.get(key).unwrap_or(&Value::Null),
            path.join_field(key),
            ignore_fields,
            numeric_fields,
        )?;
    }

    Ok(())
}

fn compare_vectors(
    vec_actual: &Vec<Value>,
    vec_expected: &Vec<Value>,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> ClientResult<()> {
    if vec_actual.len() != vec_expected.len() {
        return Err(Error::data_differs_from_proven(
            format!(
                "Field `{path}`: arrays has different lengths (expected {len_expected}, actual {len_actual})",
                path = path,
                len_actual = vec_actual.len(),
                len_expected = vec_expected.len(),
            )
        ));
    }

    for i in 0..vec_actual.len() {
        compare_values(
            &vec_actual[i],
            &vec_expected[i],
            path.join_index(i),
            ignore_fields,
            numeric_fields,
        )?;
    }

    Ok(())
}

fn get_string(value: &Value, is_numeric: bool) -> Cow<str> {
    let result = match value {
        Value::String(v) => Cow::from(v),
        _ => Cow::from(value.to_string()),
    };

    if is_numeric {
        if let Ok(value) = i128::from_str(&result) {
            if value < 0 {
                return Cow::from(format!("-0x{:x}", value.abs()));
            }
            return Cow::from(format!("0x{:x}", value));
        }
    }

    result
}

fn unix_time_to_string(value: u64) -> String {
    DateTime::<Utc>::from(std::time::UNIX_EPOCH + Duration::from_secs(value))
        .format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn add_time_strings(
    value: &mut Value,
    paths: &HashSet<&'static str>,
    path: JsonPath<'_, '_>,
) {
    match value {
        Value::Array(vec) => {
            for i in 0..vec.len() {
                add_time_strings(&mut vec[i], paths, path.join_index(i));
            }
        }
        Value::Object(map) => {
            for key in map.keys().cloned().collect::<Vec<String>>() {
                if !paths.contains(path.join_field(&key).gen_flat_str().as_str()) {
                    continue;
                }
                let unix_time = match &map[&key] {
                    Value::Number(number) => {
                        if let Some(unix_time) = number.as_u64() {
                            unix_time
                        } else {
                            continue
                        }
                    }
                    _ => continue
                };

                map.insert(format!("{}_string", key), unix_time_to_string(unix_time).into());
            }
            for (key, value) in map.iter_mut() {
                add_time_strings(value, paths, path.join_field(key));
            }
        }
        _ => return,
    }
}

pub(crate) fn serialize_block(
    id: UInt256,
    block: Block,
    boc: Vec<u8>,
) -> Result<Value> {
    let mut value = ton_block_json::db_serialize_block_ex(
        "id",
        &BlockSerializationSet {
            block,
            id,
            status: ton_block::BlockProcessingStatus::Finalized,
            boc
        },
        ton_block_json::SerializationMode::QServer,
    )?.into();

    add_time_strings(
        &mut value,
        &BLOCKS_UNIX_TIME_FIELDS,
        JsonPath::new("blocks"),
    );

    Ok(value)
}

pub(crate) fn serialize_transaction(
    id: UInt256,
    transaction: Transaction,
    block_id: UInt256,
    workchain_id: i32,
    boc: Vec<u8>,
) -> Result<Value> {
    let mut value = ton_block_json::db_serialize_transaction_ex(
        "id",
        &TransactionSerializationSet {
            transaction,
            id,
            status: ton_block::TransactionProcessingStatus::Finalized,
            block_id: Some(block_id),
            workchain_id,
            boc,
            proof: None,
        },
        ton_block_json::SerializationMode::QServer,
    )?.into();

    add_time_strings(
        &mut value,
        &TRANSACTIONS_UNIX_TIME_FIELDS,
        JsonPath::new("transactions"),
    );

    // TODO: Remove when field is added to `ton-labs-block-json`:
    if let Some(map) = value["action"].as_object_mut() {
        if let Some(status_change) = map["status_change"].as_u64() {
            let statuses = ["Unchanged", "Frozen", "Deleted"];
            if status_change < statuses.len() as u64 {
                map.insert("status_change_name".to_string(), statuses[status_change as usize].into());
            }
        }
    }

    Ok(value)
}

pub(crate) fn compare_blocks(actual: &Value, expected: &Value) -> ClientResult<()> {
    compare_values(
        actual,
        expected,
        JsonPath::new("blocks"),
        &BLOCK_IGNORE_FIELDS,
        &BLOCK_NUMERIC_FIELDS,
    )
}

pub(crate) fn compare_transactions(actual: &Value, expected: &Value) -> ClientResult<()> {
    compare_values(
        actual,
        expected,
        JsonPath::new("transactions"),
        &TRANSACTION_IGNORE_FIELDS,
        &TRANSACTION_NUMERIC_FIELDS,
    )
}
