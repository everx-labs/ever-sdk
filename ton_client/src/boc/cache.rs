/*
* Copyright 2018-2021 TON Labs LTD.
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

use super::Error;
use crate::boc::internal::{
    deserialize_cell_from_base64, deserialize_cell_from_boc, serialize_cell_to_base64,
    DeserializedBoc,
};
use crate::client::ClientContext;
use crate::error::ClientResult;

use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
#[allow(unused_imports)]
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use ton_types::{Cell, UInt256};

pub const SHA256_SIZE: usize = 32;
pub const DEPTH_SIZE: usize = 2;

fn number_of_bytes_to_fit(l: usize) -> usize {
    let mut n = 0;
    let mut l1 = l;

    while l1 != 0 {
        l1 >>= 8;
        n += 1;
    }

    n
}

fn calc_tree_cells(cell: &Cell, hashes: &mut HashSet<UInt256>) -> (usize, usize, usize) {
    let bits = cell.bit_length();
    let mut size =
        2 + if cell.store_hashes() {
            (cell.level() as usize + 1) * (SHA256_SIZE + DEPTH_SIZE)
        } else {
            0
        } + (bits / 8)
            + if bits % 8 != 0 { 1 } else { 0 };
    let mut cell_count = 1;
    let mut refs_count = cell.references_count();
    hashes.insert(cell.repr_hash());

    for i in 0..refs_count {
        let cell = cell.reference(i).unwrap();
        if hashes.contains(&cell.repr_hash()) {
            continue;
        }
        let subtree = calc_tree_cells(&cell, hashes);
        size += subtree.0;
        cell_count += subtree.1;
        refs_count += subtree.2;
    }

    (size, cell_count, refs_count)
}

fn calc_tree_size(cell: &Cell) -> usize {
    let mut hashes = HashSet::new();
    let (size, cell_count, refs_count) = calc_tree_cells(cell, &mut hashes);
    let ref_size = number_of_bytes_to_fit(cell_count);
    size + refs_count * ref_size
}

#[derive(Serialize, Deserialize, Clone, ApiType, Debug)]
#[serde(tag = "type")]
pub enum BocCacheType {
    /// Pin the BOC with `pin` name. Such BOC will not be removed from cache until it is unpinned
    /// BOCs can have several pins and each of the pins has reference counter indicating how many
    /// times the BOC was pinned with the pin. BOC is removed from cache after all references for all
    /// pins are unpinned with `cache_unpin` function calls.
    Pinned { pin: String },
    /// BOC is placed into a common BOC pool with limited size regulated by LRU
    /// (least recently used) cache lifecycle. BOC resides there until it is replaced
    /// with other BOCs if it is not used
    Unpinned,
}

impl Default for BocCacheType {
    fn default() -> Self {
        BocCacheType::Unpinned
    }
}

pub struct PinnedBoc {
    pins: HashMap<String, u32>,
    cell: Cell,
}

pub struct CachedBoc {
    size: usize,
    cell: Cell,
}

pub struct CachedBocs {
    bocs: LruCache<UInt256, CachedBoc>,
    cache_size: usize,
}

pub struct Bocs {
    pinned: RwLock<HashMap<UInt256, PinnedBoc>>,
    cached: Mutex<CachedBocs>,
    max_cache_size: usize,
}

impl Bocs {
    pub(crate) fn new(max_cache_size: u32) -> Self {
        let max_cache_size = (max_cache_size as usize)
            .checked_mul(1024) // kilobytes in config
            .unwrap_or(std::usize::MAX);
        Bocs {
            pinned: RwLock::default(),
            cached: Mutex::new(CachedBocs {
                bocs: LruCache::unbounded(),
                cache_size: 0,
            }),
            max_cache_size,
        }
    }

    fn add_new_pinned(&self, hash: UInt256, pin: String, cell: Cell) {
        let mut lock = self.pinned.write().unwrap();
        lock.entry(hash)
            .and_modify(|entry| {
                entry
                    .pins
                    .entry(pin.clone())
                    .and_modify(|refs| *refs += 1)
                    .or_insert(1);
            })
            .or_insert_with(|| PinnedBoc {
                pins: HashMap::from_iter([(pin, 1)]),
                cell,
            });
    }

    pub(crate) fn unpin(&self, pin: &str, hash: Option<UInt256>) {
        let mut to_remove = vec![];
        let mut lock = self.pinned.write().unwrap();

        if let Some(hash) = hash {
            if let Some(entry) = lock.get_mut(&hash) {
                if let Some(0) = entry.pins.get_mut(pin).map(|refs| {
                    *refs -= 1;
                    *refs
                }) {
                    entry.pins.remove(pin);
                }
                if entry.pins.is_empty() {
                    to_remove.push(hash);
                }
            }
        } else {
            for (key, entry) in lock.iter_mut() {
                if let Some(0) = entry.pins.get_mut(pin).map(|refs| {
                    *refs -= 1;
                    *refs
                }) {
                    entry.pins.remove(pin);
                }
                if entry.pins.is_empty() {
                    to_remove.push(key.clone());
                }
            }
        }

        for key in to_remove {
            lock.remove(&key);
        }
    }

    fn add_cached(&self, hash: UInt256, cell: Cell, size: usize) -> ClientResult<()> {
        if size > self.max_cache_size as usize {
            return Err(Error::insufficient_cache_size(self.max_cache_size, size));
        }
        let mut lock = self.cached.lock().unwrap();

        if let Some(_) = lock.bocs.get(&hash) {
            return Ok(());
        }

        while lock.cache_size + size > self.max_cache_size as usize {
            let (_, entry) = lock
                .bocs
                .pop_lru()
                .ok_or(Error::insufficient_cache_size(self.max_cache_size, size))?;
            lock.cache_size -= entry.size;
        }
        lock.bocs.put(hash.clone(), CachedBoc { cell, size });
        lock.cache_size += size;

        Ok(())
    }

    pub(crate) fn deserialize_cell(
        &self,
        boc: &str,
        name: &str,
    ) -> ClientResult<(DeserializedBoc, Cell)> {
        if boc.starts_with("*") {
            let hash = UInt256::from_str(&boc[1..]).map_err(|err| {
                Error::invalid_boc(format!(
                    "BOC start with `*` but contains invalid hash: {}",
                    err
                ))
            })?;

            let cell = self.get(&hash).ok_or(Error::boc_ref_not_found(boc))?;
            Ok((DeserializedBoc::Cell(cell.clone()), cell))
        } else {
            deserialize_cell_from_base64(boc, name)
                .map(|(bytes, cell)| (DeserializedBoc::Bytes(bytes), cell))
        }
    }

    pub(crate) fn resolve_boc_with_hash(
        &self,
        boc: &str,
        name: &str,
    ) -> ClientResult<(UInt256, String)> {
        let (source, cell) = self.deserialize_cell(boc, name)?;
        let hash = cell.repr_hash();
        let boc = if let DeserializedBoc::Bytes(_) = source {
            boc.to_string()
        } else {
            serialize_cell_to_base64(&cell, name)?
        };
        Ok((hash, boc))
    }

    fn get_pinned(&self, hash: &UInt256) -> Option<Cell> {
        self.pinned
            .read()
            .unwrap()
            .get(hash)
            .map(|entry| entry.cell.clone())
    }

    fn get_cached(&self, hash: &UInt256) -> Option<Cell> {
        self.cached
            .lock()
            .unwrap()
            .bocs
            .get(hash)
            .map(|entry| entry.cell.clone())
    }

    pub(crate) fn get(&self, hash: &UInt256) -> Option<Cell> {
        if let Some(cell) = self.get_pinned(&hash) {
            return Some(cell);
        }

        if let Some(cell) = self.get_cached(&hash) {
            return Some(cell);
        }

        None
    }

    pub(crate) fn add(
        &self,
        cache_type: BocCacheType,
        cell: Cell,
        size: Option<usize>,
    ) -> ClientResult<UInt256> {
        let hash = cell.repr_hash();
        log::debug!("Bocs::add {:x}", hash);
        match cache_type {
            BocCacheType::Pinned { pin } => self.add_new_pinned(hash.clone(), pin, cell),
            BocCacheType::Unpinned => {
                if let Some(_) = self.get_cached(&hash) {
                    return Ok(hash);
                }
                let size = size.unwrap_or_else(|| calc_tree_size(&cell));
                self.add_cached(hash.clone(), cell, size)?;
            }
        }
        Ok(hash)
    }
}

fn parse_boc_ref(boc_ref: &str) -> ClientResult<UInt256> {
    if !boc_ref.starts_with("*") {
        return Err(Error::invalid_boc_ref(
            "reference doesn't start with `*`. Did you use the BOC itself instead of reference?",
            boc_ref,
        ));
    }

    UInt256::from_str(&boc_ref[1..]).map_err(|err| {
        Error::invalid_boc_ref(format!("reference contains invalid hash: {}", err), boc_ref)
    })
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfBocCacheSet {
    /// BOC encoded as base64 or BOC reference
    pub boc: String,
    /// Cache type
    pub cache_type: BocCacheType,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfBocCacheSet {
    /// Reference to the cached BOC
    pub boc_ref: String,
}

/// Save BOC into cache or increase pin counter for existing pinned BOC
#[api_function]
pub async fn cache_set(
    context: Arc<ClientContext>,
    params: ParamsOfBocCacheSet,
) -> ClientResult<ResultOfBocCacheSet> {
    let (bytes, cell) = deserialize_cell_from_boc(&context, &params.boc, "BOC").await?;
    let size = match bytes {
        DeserializedBoc::Bytes(bytes) => Some(bytes.len()),
        _ => None,
    };
    context
        .bocs
        .add(params.cache_type, cell, size)
        .map(|hash| ResultOfBocCacheSet {
            boc_ref: format!("*{:x}", hash),
        })
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfBocCacheGet {
    /// Reference to the cached BOC
    pub boc_ref: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfBocCacheGet {
    /// BOC encoded as base64.
    pub boc: Option<String>,
}

/// Get BOC from cache
#[api_function]
pub async fn cache_get(
    context: Arc<ClientContext>,
    params: ParamsOfBocCacheGet,
) -> ClientResult<ResultOfBocCacheGet> {
    let hash = parse_boc_ref(&params.boc_ref)?;

    let boc = context
        .bocs
        .get(&hash)
        .map(|cell| serialize_cell_to_base64(&cell, "BOC"))
        .transpose()?;

    Ok(ResultOfBocCacheGet { boc })
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfBocCacheUnpin {
    /// Pinned name
    pub pin: String,
    /// Reference to the cached BOC. If it is provided then only referenced BOC is unpinned
    pub boc_ref: Option<String>,
}
/// Unpin BOCs with specified pin defined in the `cache_set`.

/// Decrease pin reference counter for BOCs with specified pin defined in the `cache_set`.
/// BOCs which have only 1 pin and its reference counter become 0 will be removed from cache
#[api_function]
pub async fn cache_unpin(
    context: Arc<ClientContext>,
    params: ParamsOfBocCacheUnpin,
) -> ClientResult<()> {
    let hash = params
        .boc_ref
        .map(|string| parse_boc_ref(&string))
        .transpose()?;
    context.bocs.unpin(&params.pin, hash);
    Ok(())
}
