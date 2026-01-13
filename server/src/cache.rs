use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{sync::RwLock, time::interval};

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub timestamp: Instant,
    pub data: T,
}

#[derive(Debug)]
pub struct Cache<K, V>
where
    K: Eq + Hash + Send + Sync + Clone,
    V: Send + Sync + Clone,
{
    store: Arc<RwLock<HashMap<K, Entry<V>>>>,
    key_fifo: Arc<RwLock<VecDeque<K>>>,
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
    K: Eq + Hash + Send + Sync + Clone,
    V: Send + Sync + Clone,
{
    pub fn new(ttl: Duration) -> Self {
        let map: HashMap<K, Entry<V>> = HashMap::new();
        let key_fifo: VecDeque<K> = VecDeque::new();

        Cache {
            store: Arc::new(RwLock::new(map)),
            key_fifo: Arc::new(RwLock::new(key_fifo)),
            ttl,
        }
    }

    pub async fn set(&self, key: K, value: V) {
        let mut store = self.store.write().await;

        if store.insert(key.clone(), Entry::new(value)).is_none() {
            let mut key_fifo = self.key_fifo.write().await;

            key_fifo.push_front(key.clone());
            drop(key_fifo);
        };
        drop(store);
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().await;

        let entry = match store.get(key) {
            Some(entry) => entry,
            None => return None,
        };

        if entry.is_stale(self.ttl) {
            drop(store); // Explicitly drop the read lock before acquiring the write lock
            let mut store = self.store.write().await;
            store.remove(key);
            return None;
        }

        Some(entry.data.clone())
    }

    async fn cache_size(&self) -> usize {
        let store = self.store.read().await;
        store.len()
    }

    async fn keyfifo_pop_back(&self) -> Option<K> {
        let mut key_fifo = self.key_fifo.write().await;

        key_fifo.pop_back()
    }

    async fn keyfifo_push_back(&self, key: K) {
        let mut key_fifo = self.key_fifo.write().await;

        key_fifo.push_back(key)
    }

    async fn get_timestamp(&self, key: &K) -> Option<Instant> {
        let store = self.store.read().await;
        store.get(key).map(|entry| entry.timestamp)
    }

    async fn store_remove(&self, key: &K) {
        let mut store = self.store.write().await;

        store.remove(key);
    }

    async fn sweep(&self) -> bool {
        log::debug!(
            "Sweeping cache...Cache size before sweep: {}",
            self.cache_size().await
        );

        if let Some(key) = self.keyfifo_pop_back().await
            && let Some(timestamp) = self.get_timestamp(&key).await
        {
            if timestamp.elapsed() > self.ttl {
                self.store_remove(&key).await;
                return true;
            } else {
                self.keyfifo_push_back(key).await;
            }
        }

        false
    }

    pub async fn sweep_task(&self) {
        let sweep_every = self.ttl.mul_f32(0.1).max(Duration::from_millis(50));
        let mut tick = interval(sweep_every);
        loop {
            tick.tick().await;
            while self.sweep().await {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_new() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        assert!(cache.store.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_cache_set() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await;
    }

    #[tokio::test]
    async fn test_cache_get() {
        let cache = Cache::<String, String>::new(Duration::from_secs(1));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await;

        let result = cache.get(&key).await;

        assert_eq!(result.unwrap(), value);
    }

    #[tokio::test]
    async fn test_cache_get_stale() {
        let cache = Cache::<String, String>::new(Duration::from_millis(100));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await;

        tokio::time::sleep(Duration::from_millis(200)).await;

        let result = cache.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_sweep() {
        let cache = Cache::<String, String>::new(Duration::from_millis(200));
        let key = "test".to_string();
        let value = "value".to_string();

        cache.set(key.clone(), value.clone()).await;
        assert_eq!(cache.cache_size().await, 1);

        let sweeped = cache.sweep().await;
        assert!(!sweeped);
        assert_eq!(cache.cache_size().await, 1);

        tokio::time::sleep(Duration::from_millis(300)).await;

        let sweeped = cache.sweep().await;
        assert!(sweeped);
        assert_eq!(cache.cache_size().await, 0);

        cache.set(key.clone(), value.clone()).await;
        assert_eq!(cache.cache_size().await, 1);

        tokio::time::sleep(Duration::from_millis(300)).await;

        let result = cache.get(&key).await;
        assert!(result.is_none());

        let sweeped = cache.sweep().await;
        assert!(!sweeped);
        assert_eq!(cache.cache_size().await, 0);
    }
}
