use crate::client::{ClientEnv, Error};
use crate::error::ClientResult;

#[async_trait::async_trait]
pub trait ProofStorage {
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>>;
    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()>;
    async fn get_str(&self, key: &str) -> ClientResult<Option<String>>;
    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()>;
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
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        ClientEnv::bin_read_local_storage(&self.local_storage_path, key).await
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        ClientEnv::bin_write_local_storage(&self.local_storage_path, key, value).await
    }

    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        ClientEnv::read_local_storage(&self.local_storage_path, key).await
    }

    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
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
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        Ok(
            self.proof_map.get(key)
                .map(|guard| guard.val().clone())
        )
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        self.proof_map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        self.proof_map.get(key)
            .map(|guard| String::from_utf8(guard.val().clone())
                .map_err(|err| Error::internal_error(err)))
            .transpose()
    }

    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
        self.proof_map.insert(key.to_string(), value.as_bytes().to_vec());
        Ok(())
    }
}
