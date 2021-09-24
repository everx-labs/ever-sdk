use ton_types::Result;

use crate::client::{ClientEnv, is_storage_key_correct};

#[cfg(not(feature = "wasm"))]
mod env {
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
    assert!(!is_storage_key_correct("  a  "));
    assert!(is_storage_key_correct("123"));
    assert!(is_storage_key_correct("a"));
    assert!(is_storage_key_correct("a1"));
    assert!(is_storage_key_correct("1a"));
    assert!(is_storage_key_correct("a_"));
    assert!(is_storage_key_correct("very_long_ident_232352"));
    assert!(is_storage_key_correct("directory/filename_123"));
}

#[cfg(not(feature = "wasm"))]
#[test]
fn test_storage_path_calculation() {
    assert_eq!(
        ClientEnv::calc_storage_path(&None, "test"),
        home::home_dir()
            .unwrap_or(std::path::PathBuf::from("/"))
            .join(crate::client::LOCAL_STORAGE_DEFAULT_DIR_NAME)
            .join("test"),
    );

    const LOCAL_STORAGE_PATH: &str = "/local-storage";
    assert_eq!(
        ClientEnv::calc_storage_path(&Some(LOCAL_STORAGE_PATH.to_string()), "test"),
        std::path::Path::new(LOCAL_STORAGE_PATH).join("test"),
    );
}

#[tokio::test]
async fn test_local_storage() -> Result<()> {
    let path = self::env::LocalStoragePathManager::new();

    const KEY1_NAME: &str = "key1";
    const KEY2_NAME: &str = "key2";

    assert!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY1_NAME).await?.is_none());
    assert!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY2_NAME).await?.is_none());

    ClientEnv::bin_write_local_storage(path.as_ref(), KEY1_NAME, b"test1").await?;

    assert_eq!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY1_NAME).await?, Some(b"test1".to_vec()));
    assert!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY2_NAME).await?.is_none());

    ClientEnv::bin_write_local_storage(path.as_ref(), KEY2_NAME, b"test2").await?;

    assert_eq!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY1_NAME).await?, Some(b"test1".to_vec()));
    assert_eq!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY2_NAME).await?, Some(b"test2".to_vec()));

    ClientEnv::remove_local_storage(path.as_ref(), KEY1_NAME).await?;

    assert!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY1_NAME).await?.is_none());
    assert_eq!(ClientEnv::bin_read_local_storage(path.as_ref(), KEY2_NAME).await?, Some(b"test2".to_vec()));

    Ok(())
}
