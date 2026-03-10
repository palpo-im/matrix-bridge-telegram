use std::time::{Duration, Instant};

pub struct TimedCache<T> {
    data: Option<T>,
    expires_at: Option<Instant>,
    ttl: Duration,
}

impl<T> TimedCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: None,
            expires_at: None,
            ttl,
        }
    }

    pub fn get(&self) -> Option<&T> {
        if let Some(expires_at) = self.expires_at {
            if Instant::now() < expires_at {
                return self.data.as_ref();
            }
        }
        None
    }

    pub fn set(&mut self, value: T) {
        self.data = Some(value);
        self.expires_at = Some(Instant::now() + self.ttl);
    }

    pub fn invalidate(&mut self) {
        self.data = None;
        self.expires_at = None;
    }
}

impl<T: Clone> TimedCache<T> {
    pub fn get_or_update<F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(ref data) = self.data {
            if let Some(expires_at) = self.expires_at {
                if Instant::now() < expires_at {
                    return data.clone();
                }
            }
        }
        let value = f();
        self.set(value.clone());
        value
    }
}

use std::collections::HashMap;
use std::hash::Hash;
use tokio::sync::RwLock;

/// Async version of TimedCache using tokio RwLock.
pub struct AsyncTimedCache<K, V> {
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + Hash + Clone, V: Clone> AsyncTimedCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().await;
        entries.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub async fn set(&self, key: K, value: V) {
        let mut entries = self.entries.write().await;
        entries.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|e| e.value)
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Remove expired entries.
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        entries.retain(|_, entry| now < entry.expires_at);
    }

    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    pub async fn is_empty(&self) -> bool {
        let entries = self.entries.read().await;
        entries.is_empty()
    }
}

/// Portal mapping cache: Telegram chat_id -> Matrix room_id
pub type PortalCache = AsyncTimedCache<i64, String>;

/// User info cache: Telegram user_id -> display name
pub type UserInfoCache = AsyncTimedCache<i64, String>;

/// Create a portal cache with default 1-hour TTL.
pub fn new_portal_cache() -> PortalCache {
    AsyncTimedCache::new(Duration::from_secs(3600))
}

/// Create a user info cache with default 30-minute TTL.
pub fn new_user_info_cache() -> UserInfoCache {
    AsyncTimedCache::new(Duration::from_secs(1800))
}
