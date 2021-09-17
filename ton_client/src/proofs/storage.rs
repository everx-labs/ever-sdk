use crate::client::ClientEnv;
use crate::error::ClientResult;

#[async_trait::async_trait]
pub trait ProofStorage {
    async fn get(&self, key: &str) -> ClientResult<Option<Vec<u8>>>;
    async fn put(&self, key: &str, value: &[u8]) -> ClientResult<()>;
}

pub struct LocalProofStorage {
    local_storage_path: Option<String>,
}

impl LocalProofStorage {
    pub const fn new(local_storage_path: Option<String>) -> Self {
        Self { local_storage_path }
    }
}

#[async_trait::async_trait]
impl ProofStorage for LocalProofStorage {
    async fn get(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        ClientEnv::read_local_storage(&self.local_storage_path, key).await
    }

    async fn put(&self, key: &str, value: &[u8]) -> ClientResult<()> {
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
}

#[async_trait::async_trait]
impl ProofStorage for InMemoryProofStorage {
    async fn get(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        Ok(
            self.proof_map.get(key)
                .map(|guard| guard.val().clone())
        )
    }

    async fn put(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        self.proof_map.insert(key.to_string(), value.to_vec());
        Ok(())
    }
}
