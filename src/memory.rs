use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::Result;

#[async_trait]
pub trait MemoryBackend: Send + Sync + std::fmt::Debug {
    async fn store(&mut self, key: &str, value: &Value) -> Result<()>;
    async fn retrieve(&mut self, key: &str) -> Result<Option<Value>>;
    async fn delete(&mut self, key: &str) -> Result<bool>;
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>>;
    async fn clear(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct InMemoryBackend {
    storage: Arc<Mutex<HashMap<String, Value>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryBackend for InMemoryBackend {
    async fn store(&mut self, key: &str, value: &Value) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key.to_string(), value.clone());
        Ok(())
    }

    async fn retrieve(&mut self, key: &str) -> Result<Option<Value>> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.get(key).cloned())
    }

    async fn delete(&mut self, key: &str) -> Result<bool> {
        let mut storage = self.storage.lock().unwrap();
        Ok(storage.remove(key).is_some())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let storage = self.storage.lock().unwrap();
        let keys: Vec<String> = match prefix {
            Some(p) => storage.keys()
                .filter(|k| k.starts_with(p))
                .cloned()
                .collect(),
            None => storage.keys().cloned().collect(),
        };
        Ok(keys)
    }

    async fn clear(&mut self) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.clear();
        Ok(())
    }
}

#[cfg(feature = "persistence")]
pub mod persistent {
    use super::*;
    use std::path::Path;
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[derive(Debug)]
    pub struct FileBackend {
        base_path: std::path::PathBuf,
        in_memory: InMemoryBackend,
    }

    impl FileBackend {
        pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
            let base_path = base_path.as_ref().to_path_buf();
            
            if !base_path.exists() {
                fs::create_dir_all(&base_path).await
                    .map_err(|e| crate::Error::Io(e))?;
            }

            let mut backend = Self {
                base_path,
                in_memory: InMemoryBackend::new(),
            };

            // Load existing data
            backend.load_from_disk().await?;
            Ok(backend)
        }

        async fn load_from_disk(&mut self) -> Result<()> {
            let mut entries = fs::read_dir(&self.base_path).await
                .map_err(|e| crate::Error::Io(e))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| crate::Error::Io(e))? {
                
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let key = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    let mut file = fs::File::open(&path).await
                        .map_err(|e| crate::Error::Io(e))?;
                    
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).await
                        .map_err(|e| crate::Error::Io(e))?;

                    let value: Value = serde_json::from_str(&contents)?;
                    self.in_memory.store(&key, &value).await?;
                }
            }

            Ok(())
        }

        async fn save_to_disk(&self, key: &str, value: &Value) -> Result<()> {
            let file_path = self.base_path.join(format!("{}.json", key));
            let content = serde_json::to_string_pretty(value)?;
            
            let mut file = fs::File::create(file_path).await
                .map_err(|e| crate::Error::Io(e))?;
            
            file.write_all(content.as_bytes()).await
                .map_err(|e| crate::Error::Io(e))?;
            
            Ok(())
        }

        async fn remove_from_disk(&self, key: &str) -> Result<()> {
            let file_path = self.base_path.join(format!("{}.json", key));
            if file_path.exists() {
                fs::remove_file(file_path).await
                    .map_err(|e| crate::Error::Io(e))?;
            }
            Ok(())
        }
    }

    #[async_trait]
    impl MemoryBackend for FileBackend {
        async fn store(&mut self, key: &str, value: &Value) -> Result<()> {
            self.in_memory.store(key, value).await?;
            self.save_to_disk(key, value).await?;
            Ok(())
        }

        async fn retrieve(&mut self, key: &str) -> Result<Option<Value>> {
            self.in_memory.retrieve(key).await
        }

        async fn delete(&mut self, key: &str) -> Result<bool> {
            let deleted = self.in_memory.delete(key).await?;
            if deleted {
                self.remove_from_disk(key).await?;
            }
            Ok(deleted)
        }

        async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
            self.in_memory.list_keys(prefix).await
        }

        async fn clear(&mut self) -> Result<()> {
            let keys = self.list_keys(None).await?;
            for key in keys {
                self.remove_from_disk(&key).await?;
            }
            self.in_memory.clear().await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[cfg(feature = "nats")]
    #[tokio::test]
    async fn test_in_memory_backend() {
        let mut backend = InMemoryBackend::new();
        let test_value = json!({"test": "data"});

        // Test store and retrieve
        backend.store("test_key", &test_value).await.unwrap();
        let retrieved = backend.retrieve("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_value.clone()));

        // Test list keys
        let keys = backend.list_keys(None).await.unwrap();
        assert!(keys.contains(&"test_key".to_string()));

        // Test delete
        let deleted = backend.delete("test_key").await.unwrap();
        assert!(deleted);
        
        let retrieved_after_delete = backend.retrieve("test_key").await.unwrap();
        assert_eq!(retrieved_after_delete, None);
    }

    #[cfg(feature = "nats")]
    #[tokio::test]
    async fn test_prefix_filtering() {
        let mut backend = InMemoryBackend::new();
        let test_value = json!({"test": "data"});

        backend.store("agent1:state", &test_value).await.unwrap();
        backend.store("agent2:state", &test_value).await.unwrap();
        backend.store("system:config", &test_value).await.unwrap();

        let agent_keys = backend.list_keys(Some("agent")).await.unwrap();
        assert_eq!(agent_keys.len(), 2);
        assert!(agent_keys.contains(&"agent1:state".to_string()));
        assert!(agent_keys.contains(&"agent2:state".to_string()));

        let system_keys = backend.list_keys(Some("system")).await.unwrap();
        assert_eq!(system_keys.len(), 1);
        assert!(system_keys.contains(&"system:config".to_string()));
    }

    #[cfg(feature = "persistence")]
    mod persistent_tests {
        use super::*;
        use tempfile::tempdir;

        #[tokio::test]
        async fn test_file_backend() {
            let temp_dir = tempdir().unwrap();
            let mut backend = persistent::FileBackend::new(temp_dir.path()).await.unwrap();
            
            let test_value = json!({"test": "data"});
            backend.store("test_key", &test_value).await.unwrap();
            
            let retrieved = backend.retrieve("test_key").await.unwrap();
            assert_eq!(retrieved, Some(test_value));
        }
    }
}