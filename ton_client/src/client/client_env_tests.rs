use ton_types::Result;

use crate::client::LocalStorage;
use crate::client::storage::KeyValueStorage;

#[cfg(not(feature = "wasm-base"))]
mod env {
    use crate::client::std_client_env::LocalStorage;

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

    #[test]
    fn test_storage_key_validation() {
        assert!(!LocalStorage::is_storage_key_correct(""));
        assert!(!LocalStorage::is_storage_key_correct("-"));
        assert!(!LocalStorage::is_storage_key_correct("A B"));
        assert!(!LocalStorage::is_storage_key_correct("  a  "));
        assert!(LocalStorage::is_storage_key_correct("123"));
        assert!(LocalStorage::is_storage_key_correct("a"));
        assert!(LocalStorage::is_storage_key_correct("a1"));
        assert!(LocalStorage::is_storage_key_correct("1a"));
        assert!(LocalStorage::is_storage_key_correct("a_"));
        assert!(LocalStorage::is_storage_key_correct("very_long_ident_232352"));
        assert!(!LocalStorage::is_storage_key_correct("directory/filename_123"));
    }
}

#[cfg(feature = "wasm-base")]
mod env {
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

#[cfg(not(feature = "wasm-base"))]
#[test]
fn test_storage_path_calculation() {
    assert_eq!(
        LocalStorage::calc_storage_path(&None, "test"),
        home::home_dir()
            .unwrap_or(std::path::PathBuf::from("/"))
            .join(crate::client::LOCAL_STORAGE_DEFAULT_DIR_NAME)
            .join("test"),
    );

    const LOCAL_STORAGE_PATH: &str = "/local-storage";
    assert_eq!(
        LocalStorage::calc_storage_path(&Some(LOCAL_STORAGE_PATH.to_string()), "test"),
        std::path::Path::new(LOCAL_STORAGE_PATH).join("test"),
    );
}

#[tokio::test]
async fn test_local_storage() -> Result<()> {
    let path = self::env::LocalStoragePathManager::new();

    let storage = LocalStorage::new(path.as_ref().clone(), "test".to_string()).await?;

    const KEY1_NAME: &str = "key1";
    const KEY2_NAME: &str = "key2";

    assert!(storage.get_str(KEY1_NAME).await?.is_none());
    assert!(storage.get_bin(KEY1_NAME).await?.is_none());
    assert!(storage.get_str(KEY2_NAME).await?.is_none());
    assert!(storage.get_bin(KEY2_NAME).await?.is_none());

    storage.put_str(KEY1_NAME, "test1").await?;

    assert_eq!(storage.get_str(KEY1_NAME).await?, Some("test1".to_string()));
    assert_eq!(storage.get_bin(KEY1_NAME).await?, Some(b"test1".to_vec()));
    assert!(storage.get_str(KEY2_NAME).await?.is_none());
    assert!(storage.get_bin(KEY2_NAME).await?.is_none());

    storage.put_bin(KEY2_NAME, b"test2").await?;

    assert_eq!(storage.get_str(KEY1_NAME).await?, Some("test1".to_string()));
    assert_eq!(storage.get_bin(KEY1_NAME).await?, Some(b"test1".to_vec()));
    assert_eq!(storage.get_str(KEY2_NAME).await?, Some("test2".to_string()));
    assert_eq!(storage.get_bin(KEY2_NAME).await?, Some(b"test2".to_vec()));

    storage.remove(KEY1_NAME).await?;

    assert!(storage.get_str(KEY1_NAME).await?.is_none());
    assert!(storage.get_bin(KEY1_NAME).await?.is_none());
    assert_eq!(storage.get_str(KEY2_NAME).await?, Some("test2".to_string()));

    Ok(())
}
