use ton_types::Result;

use crate::client::ClientEnv;

#[async_trait::async_trait]
pub trait ProofStorage: Send + Sync {
    #[cfg(test)]
    fn in_memory(&self) -> &InMemoryProofStorage;
    async fn get_bin(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn put_bin(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn get_str(&self, key: &str) -> Result<Option<String>>;
    async fn put_str(&self, key: &str, value: &str) -> Result<()>;
}

pub struct LocalStorage {
    local_storage_path: Option<String>,
}

impl LocalStorage {
    pub const fn new(local_storage_path: Option<String>) -> Self {
        Self { local_storage_path }
    }
}

#[async_trait::async_trait]
impl ProofStorage for LocalStorage {
    #[cfg(test)]
    fn in_memory(&self) -> &InMemoryProofStorage {
        panic!("`in_memory()` is not supported for LocalStorage");
    }

    async fn get_bin(&self, key: &str) -> Result<Option<Vec<u8>>> {
        ClientEnv::bin_read_local_storage(&self.local_storage_path, key).await
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> Result<()> {
        ClientEnv::bin_write_local_storage(&self.local_storage_path, key, value).await
    }

    async fn get_str(&self, key: &str) -> Result<Option<String>> {
        ClientEnv::read_local_storage(&self.local_storage_path, key).await
    }

    async fn put_str(&self, key: &str, value: &str) -> Result<()> {
        ClientEnv::write_local_storage(&self.local_storage_path, key, value).await
    }
}

pub struct InMemoryProofStorage {
    proof_map: lockfree::map::Map<String, Vec<u8>>,
}

impl InMemoryProofStorage {
    pub fn new() -> Self {
        Self {
            proof_map: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.proof_map.iter().count()
    }

    #[cfg(test)]
    pub fn dump(&self) {
        println!("\n### Storage dump ###");
        let count = self.proof_map.iter()
            .map(|pair|
                println!(
                    "Key: {}, value (len: {}): {:?}",
                    pair.key(),
                    pair.val().len(),
                    &pair.val()[..std::cmp::min(10, pair.val().len())],
                )
            ).count();

        println!("Total records: {}", count);
    }
}

#[async_trait::async_trait]
impl ProofStorage for InMemoryProofStorage {
    #[cfg(test)]
    fn in_memory(&self) -> &InMemoryProofStorage {
        self
    }

    async fn get_bin(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(
            self.proof_map.get(key)
                .map(|guard| guard.val().clone())
        )
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> Result<()> {
        self.proof_map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn get_str(&self, key: &str) -> Result<Option<String>> {
        self.proof_map.get(key)
            .map(|guard| String::from_utf8(guard.val().clone())
                .map_err(|err| err.into()))
            .transpose()
    }

    async fn put_str(&self, key: &str, value: &str) -> Result<()> {
        self.proof_map.insert(key.to_string(), value.as_bytes().to_vec());
        Ok(())
    }
}
