use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to acquire write lock")]
    FailedToAcquireWriteLock,
    #[error("Failed to acquire read lock")]
    FailedToAcquireReadLock,
}

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub timestamp: Instant,
    pub data: T,
}

type Result<T> = std::result::Result<T, CacheError>;

pub struct Cache<K, V>
where
    K: Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    store: Arc<RwLock<HashMap<K, Entry<V>>>>,
    ttl: Duration,
}

impl<T> Entry<T> {
    pub fn new(data: T) -> Self {
        Entry {
            timestamp: Instant::now(),
            data,
        }
    }

    pub fn is_stale(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Send + Sync,
    V: Send + Sync + Clone,
{
    pub fn new(ttl: Duration) -> Self {
        let map: HashMap<K, Entry<V>> = HashMap::new();

        Cache {
            store: Arc::new(RwLock::new(map)),
            ttl,
        }
    }

    pub async fn set(&self, key: K, value: V) -> Result<()> {
        let mut store = self
            .store
            .write()
            .map_err(|_| CacheError::FailedToAcquireWriteLock)?;

        store.insert(key, Entry::new(value));

        Ok(())
    }

    pub async fn get(&self, key: &K) -> Result<Option<Entry<V>>> {
        let store = self
            .store
            .read()
            .map_err(|_| CacheError::FailedToAcquireReadLock)?;

        let entry = match store.get(key) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        if entry.is_stale(self.ttl) {
            drop(store); // Explicitly drop the read lock before acquiring the write lock
            let mut store = self
                .store
                .write()
                .map_err(|_| CacheError::FailedToAcquireWriteLock)?;
            store.remove(key);
            return Ok(None);
        }

        Ok(Some(entry.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        assert!(cache.store.read().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cache_set() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_get() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await.unwrap();

        let result = cache.get(&key).await.unwrap();

        assert_eq!(result.unwrap().data, value);
    }

    #[tokio::test]
    async fn test_cache_get_stale() {
        let cache = Cache::<String, String>::new(Duration::from_millis(100));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }
}
