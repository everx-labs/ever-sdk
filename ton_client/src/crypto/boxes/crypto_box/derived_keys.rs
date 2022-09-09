use crate::client::ClientEnv;
use crate::crypto;
use crate::crypto::internal::SecretBuf;
use crate::error::ClientResult;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

struct SecretHash(u64);

impl Drop for SecretHash {
    fn drop(&mut self) {
        self.0 = 0;
    }
}

struct DerivedKey {
    hash: SecretHash,
    ttl_ms: u64,
    expired_at: u64,
    key: SecretBuf,
}

impl DerivedKey {
    fn calc_hash(password: &[u8], salt: &str) -> SecretHash {
        let crc = crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182);
        let mut digest = crc.digest();
        digest.update(password);
        digest.update(salt.as_bytes());
        SecretHash(digest.finalize())
    }

    fn calc_key(password: &[u8], salt: &str) -> ClientResult<SecretBuf> {
        let scrypt_params = scrypt::Params::new(14, 8, 1).expect("Scrypt params setup failed");
        let mut key = SecretBuf(vec![0; 32]);
        scrypt::scrypt(password, salt.as_bytes(), &scrypt_params, &mut key.0)
            .map_err(|err| crypto::Error::scrypt_failed(err))?;
        Ok(key)
    }
}

struct DerivedKeysCache {
    keys: Vec<DerivedKey>,
    env: Arc<ClientEnv>,
}

impl DerivedKeysCache {
    fn touch(&mut self, hash: &SecretHash) -> Option<&SecretBuf> {
        for key in &mut self.keys {
            if key.hash.0 == hash.0 {
                println!("use cached key");
                key.expired_at = self.env.now_ms() + key.ttl_ms;
                return Some(&key.key);
            }
        }
        None
    }

    // Ensure that key is present in cache and returns `true` if the clean timer must be started
    fn put_and_check_start_timer(
        &mut self,
        hash: &SecretHash,
        key: &SecretBuf,
        calculation_time: u64,
    ) -> bool {
        if self.touch(hash).is_some() {
            return false;
        }
        let ttl_ms = calculation_time * 2;
        self.keys.push(DerivedKey {
            hash: SecretHash(hash.0),
            key: key.clone(),
            ttl_ms,
            expired_at: self.env.now_ms() + ttl_ms,
        });
        println!("put key to cache with ttl {}", ttl_ms);
        self.keys.len() == 1
    }

    fn clean_and_check_stop_timer(&mut self) -> bool {
        let now = self.env.now_ms();
        for i in (0..self.keys.len()).rev() {
            if self.keys[i].expired_at <= now {
                println!("remove expired key");
                self.keys.remove(i);
            }
        }
        self.keys.is_empty()
    }
}

#[derive(Clone)]
pub(crate) struct DerivedKeys {
    cache: Arc<RwLock<DerivedKeysCache>>,
}

impl DerivedKeys {
    pub(crate) fn new(env: Arc<ClientEnv>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(DerivedKeysCache {
                keys: Vec::new(),
                env,
            })),
        }
    }

    pub(crate) fn derive(&self, password: &[u8], salt: &str) -> ClientResult<SecretBuf> {
        let hash = DerivedKey::calc_hash(password, salt);
        if let Some(existing) = { self.write_cache().touch(&hash).map(|x| x.clone()) } {
            return Ok(existing);
        }
        let calculation_start = { self.read_cache().env.now_ms() };
        let key = DerivedKey::calc_key(password, salt)?;
        let calculation_time = { self.read_cache().env.now_ms() - calculation_start };
        let start_timer = {
            self.write_cache()
                .put_and_check_start_timer(&hash, &key, calculation_time)
        };
        if start_timer {
            let keys = self.clone();
            let env = { self.read_cache().env.clone() };
            let inner_env = env.clone();
            env.spawn(async move {
                let mut stop_timer = false;
                println!("start clean timer");
                while !stop_timer {
                    let _ = inner_env.set_timer(1000u64).await;
                    stop_timer = keys.write_cache().clean_and_check_stop_timer();
                }
                println!("stop clean timer");
            });
        }
        Ok(key)
    }

    fn read_cache(&self) -> RwLockReadGuard<DerivedKeysCache> {
        self.cache.read().unwrap()
    }

    fn write_cache(&self) -> RwLockWriteGuard<DerivedKeysCache> {
        self.cache.write().unwrap()
    }
}
