use std::path::{Path, PathBuf};

pub struct LuxmcCache {
    cache_dir: PathBuf,
}

impl LuxmcCache {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            cache_dir: dir.as_ref().to_path_buf(),
        }
    }

    pub async fn put(&self, key: &str, data: &[u8]) -> Result<(), String> {
        cacache::write(&self.cache_dir, key, data)
            .await
            .map(|_| ())
            .map_err(|e| format!("Falha ao gravar no cacache: {e}"))
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        cacache::read(&self.cache_dir, key).await.ok()
    }

    pub async fn exists(&self, key: &str) -> bool {
        cacache::metadata(&self.cache_dir, key)
            .await
            .ok()
            .flatten()
            .is_some()
    }
}
