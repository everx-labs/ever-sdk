use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};

// use ed25519_dalek::Digest;
use failure::bail;
use sha2::Digest;
use ton_block::{
    CatchainConfig, ConfigParams, UnixTime32, ValidatorDescr, ValidatorSet, WorkchainDescr,
    Workchains,
};
use ton_types::Result;

use crate::proofs::Signatures;

pub(crate) fn calc_workchain_id(descr: &ValidatorDescr) -> i32 {
    calc_workchain_id_by_adnl_id(descr.compute_node_id_short().as_slice())
}

pub(crate) fn calc_workchain_id_by_adnl_id(adnl_id: &[u8]) -> i32 {
    (adnl_id[0] % 32) as i32 - 1
}

lazy_static::lazy_static! {
    static ref SINGLE_WORKCHAIN: Workchains = {
        let mut workchains = Workchains::default();
        workchains.set(&0, &WorkchainDescr::default()).unwrap();
        workchains
    };
}

pub fn try_calc_subset_for_workchain(
    validator_set: &ValidatorSet,
    config: &ConfigParams,
    cc_config: &CatchainConfig,
    shard_pfx: u64,
    workchain_id: i32,
    cc_seqno: u32,
    _time: UnixTime32,
) -> Result<Option<(Vec<ValidatorDescr>, u32)>> {
    // In a case of old block proof it doesn't contain workchains in config, so 1 workchain by default
    let workchains = config
        .workchains()
        .unwrap_or_else(|_| SINGLE_WORKCHAIN.clone());
    match workchains.len()? as i32 {
        0 => bail!("workchain's description is empty"),
        1 => validator_set
            .calc_subset(cc_config, shard_pfx, workchain_id, cc_seqno, _time)
            .map(|e| Some(e)),
        count => {
            let mut list = Vec::new();
            for descr in validator_set.list() {
                let id = calc_workchain_id(descr);
                if (id == workchain_id) || (id >= count) {
                    list.push(descr.clone());
                }
            }
            if list.len() >= cc_config.shard_validators_num as usize {
                let validator_set = ValidatorSet::new(
                    validator_set.utime_since(),
                    validator_set.utime_until(),
                    validator_set.main(),
                    list,
                )?;
                validator_set
                    .calc_subset(cc_config, shard_pfx, workchain_id, cc_seqno, _time)
                    .map(|e| Some(e))
            } else {
                // Not enough validators -- config is ok, but we can't validate the shard at the moment
                Ok(None)
            }
        }
    }
}

pub fn calc_subset_for_workchain(
    validator_set: &ValidatorSet,
    config: &ConfigParams,
    cc_config: &CatchainConfig,
    shard_pfx: u64,
    workchain_id: i32,
    cc_seqno: u32,
    time: UnixTime32,
) -> Result<(Vec<ValidatorDescr>, u32)> {
    match try_calc_subset_for_workchain(
        validator_set,
        config,
        cc_config,
        shard_pfx,
        workchain_id,
        cc_seqno,
        time,
    )? {
        Some(x) => Ok(x),
        None => bail!(
            "Not enough validators from total {} for workchain {}:{:016X} cc_seqno: {}",
            validator_set.list().len(),
            workchain_id,
            shard_pfx,
            cc_seqno,
        ),
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct AdnlKeyId([u8; 32]);

impl AdnlKeyId {
    pub const KEY_ED25519: i32 = 1209251014;

    /// Create from type and public key
    pub fn from_type_and_public_key(type_id: i32, pub_key: &[u8; 32]) -> Self {
        Self::calc_id(type_id, pub_key)
    }

    /// Calculate key ID
    fn calc_id(type_id: i32, pub_key: &[u8; 32]) -> Self {
        let mut sha = sha2::Sha256::new();
        sha.update(&type_id.to_le_bytes());
        sha.update(pub_key);
        let buf = sha.finalize();
        let src = buf.as_slice();

        Self(src.try_into().unwrap())
    }

    pub fn data(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for AdnlKeyId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", base64::encode(self.data()))
    }
}

pub(crate) fn check_crypto_signatures(
    signatures: &Signatures,
    validators_list: &[ValidatorDescr],
    data: &[u8],
) -> Result<u64> {
    // Calc validators short ids
    let validators_map = validators_list
        .iter()
        .map(|desc| {
            let key = AdnlKeyId::from_type_and_public_key(
                AdnlKeyId::KEY_ED25519,
                desc.public_key.as_slice(),
            );
            (key, desc)
        })
        .collect::<HashMap<_, _>>();
    // Check signatures
    let mut weight = 0;
    for sign in signatures.pure_signatures() {
        let key = AdnlKeyId(sign.node_id_short.as_array().clone());
        if let Some(vd) = validators_map.get(&key) {
            if !vd.verify_signature(data, &sign.sign) {
                bail!("bad signature from validator with pub_key {}", key)
            }
            weight += vd.weight;
        }
    }

    Ok(weight)
}
