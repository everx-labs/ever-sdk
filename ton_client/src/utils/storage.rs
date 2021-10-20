use crate::error::ClientResult;
use crate::utils::Error;

#[async_trait::async_trait]
pub trait KeyValueStorage: Send + Sync {
    #[cfg(test)]
    fn in_memory(&self) -> &InMemoryKeyValueStorage {
        unimplemented!()
    }

    /// Get binary value by a given key from the storage
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>>;

    /// Put binary value by a given key into the storage
    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()>;

    /// Get string value by a given key from the storage
    async fn get_str(&self, key: &str) -> ClientResult<Option<String>>;

    /// Put string value by a given key into the storage
    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()>;

    /// Remove value by a given key
    async fn remove(&self, key: &str) -> ClientResult<()>;
}

pub struct InMemoryKeyValueStorage {
    map: lockfree::map::Map<String, Vec<u8>>,
}

impl InMemoryKeyValueStorage {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.map.iter().count()
    }

    #[cfg(test)]
    pub fn dump(&self) {
        println!("\n### Storage dump ###");
        let count = self.map.iter()
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
impl KeyValueStorage for InMemoryKeyValueStorage {
    #[cfg(test)]
    fn in_memory(&self) -> &InMemoryKeyValueStorage {
        self
    }

    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        Ok(
            self.map.get(key)
                .map(|guard| guard.val().clone())
        )
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        self.map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        self.map.get(key)
            .map(|guard| String::from_utf8(guard.val().clone())
                .map_err(|err| Error::internal_error(err)))
            .transpose()
    }

    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
        self.map.insert(key.to_string(), value.as_bytes().to_vec());
        Ok(())
    }

    async fn remove(&self, key: &str) -> ClientResult<()> {
        self.map.remove(key);
        Ok(())
    }
}
