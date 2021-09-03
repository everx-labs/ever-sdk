use ton_types::Result;

use crate::client::{ClientEnv, is_storage_key_correct};

#[cfg(not(feature = "wasm"))]
mod env {
    use ton_types::Result;

    pub struct LocalStoragePathManager {
        path: Option<String>,
    }

    impl LocalStoragePathManager {
        pub fn new() -> Self {
            let temp_dir = std::env::temp_dir();
            loop {
                let path = temp_dir
                    .join(format!("tonclient-{}", rand::random::<u32>()));
                if !path.exists() {
                    return Self {
                        path: Some(path.to_string_lossy().to_string())
                    };
                }
            }
        }
    }

    impl AsRef<Option<String>> for LocalStoragePathManager {
        fn as_ref(&self) -> &Option<String> {
            &self.path
        }
    }

    impl Drop for LocalStoragePathManager {
        fn drop(&mut self) {
            if let Some(ref path) = self.path {
                let _ignore_errors = std::fs::remove_dir_all(path);
            }
        }
    }
}

#[cfg(feature = "wasm")]
mod env {
    use super::*;

    pub struct LocalStoragePathManager;

    impl LocalStoragePathManager {
        pub fn new() -> Self {
            Self
        }
    }

    impl AsRef<Option<String>> for LocalStoragePathManager {
        fn as_ref(&self) -> &Option<String> {
            &None
        }
    }
}

#[test]
fn test_storage_key_validation() {
    assert!(!is_storage_key_correct(""));
    assert!(!is_storage_key_correct("-"));
    assert!(!is_storage_key_correct("A B"));
    assert!(!is_storage_key_correct("9"));
    assert!(!is_storage_key_correct("  a  "));
    assert!(is_storage_key_correct("a"));
    assert!(is_storage_key_correct("a1"));
    assert!(is_storage_key_correct("a_"));
    assert!(is_storage_key_correct("very_long_ident_232352"));
}

#[cfg(not(feature = "wasm"))]
#[test]
fn test_storage_path_calculation() -> Result<()> {
    ClientEnv::calc_storage_path()
}

#[tokio::test]
async fn test_local_storage() -> Result<()> {
    let client = ClientEnv::new()?;

    let path = self::env::LocalStoragePathManager::new();

    const KEY1_NAME: &str = "key1";
    const KEY2_NAME: &str = "key2";

    assert!(client.read_local_storage(path.as_ref(), KEY1_NAME).await?.is_none());
    assert!(client.read_local_storage(path.as_ref(), KEY2_NAME).await?.is_none());

    client.write_local_storage(path.as_ref(), KEY1_NAME, "test1").await?;

    assert_eq!(client.read_local_storage(path.as_ref(), KEY1_NAME).await?, Some("test1".to_string()));
    assert!(client.read_local_storage(path.as_ref(), KEY2_NAME).await?.is_none());

    client.write_local_storage(path.as_ref(), KEY2_NAME, "test2").await?;

    assert_eq!(client.read_local_storage(path.as_ref(), KEY1_NAME).await?, Some("test1".to_string()));
    assert_eq!(client.read_local_storage(path.as_ref(), KEY2_NAME).await?, Some("test2".to_string()));

    client.remove_local_storage(path.as_ref(), KEY1_NAME).await?;

    assert!(client.read_local_storage(path.as_ref(), KEY1_NAME).await?.is_none());
    assert_eq!(client.read_local_storage(path.as_ref(), KEY2_NAME).await?, Some("test2".to_string()));

    Ok(())
}
