use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
        }
    }

    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        Instant::now().duration_since(self.created_at) > ttl
    }
}

#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub evictions: u64,
}

pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
    ttl: Duration,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in get operation, recovering");
                poisoned.into_inner()
            }
        };

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired(self.ttl) {
                entries.remove(key);
                self.update_stats(|s| s.misses += 1);
                return None;
            }

            entry.access();
            let value = entry.value.clone();
            self.update_stats(|s| {
                s.hits += 1;
                s.size = entries.len();
            });
            return Some(value);
        }

        self.update_stats(|s| {
            s.misses += 1;
            s.size = entries.len();
        });
        None
    }

    pub fn insert(&self, key: K, value: V) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in insert operation, recovering");
                poisoned.into_inner()
            }
        };

        if entries.len() >= self.max_size {
            self.evict(&mut entries);
        }

        entries.insert(key, CacheEntry::new(value));
        self.update_stats(|s| s.size = entries.len());
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in remove operation, recovering");
                poisoned.into_inner()
            }
        };
        entries.remove(key).map(|e| {
            self.update_stats(|s| s.size = entries.len());
            e.value
        })
    }

    pub fn clear(&self) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in clear operation, recovering");
                poisoned.into_inner()
            }
        };
        entries.clear();
        self.update_stats(|s| s.size = 0);
    }

    pub fn size(&self) -> usize {
        match self.entries.read() {
            Ok(guard) => guard.len(),
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in size operation, recovering");
                poisoned.into_inner().len()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.entries.read() {
            Ok(guard) => guard.is_empty(),
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in is_empty operation, recovering");
                poisoned.into_inner().is_empty()
            }
        }
    }

    pub fn stats(&self) -> CacheStats {
        match self.stats.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in stats operation, recovering");
                poisoned.into_inner().clone()
            }
        }
    }

    fn evict(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if entries.is_empty() {
            return;
        }

        let now = Instant::now();

        let expired_keys: Vec<K> = entries
            .iter()
            .filter(|(_, entry)| {
                now.duration_since(entry.last_accessed) > self.ttl / 2 ||
                entry.access_count == 0
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            entries.remove(&key);
        }

        if entries.len() >= self.max_size {
            let keys_to_remove: Vec<K> = entries
                .iter()
                .filter(|(_, entry)| !entry.is_expired(self.ttl))
                .map(|(k, _)| k.clone())
                .collect();

            let to_remove = std::cmp::min(self.max_size / 4, keys_to_remove.len());
            let keys_to_remove = &keys_to_remove[..to_remove];

            for key in keys_to_remove {
                entries.remove(key);
                self.update_stats(|s| s.evictions += 1);
            }
        }
    }

    fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut CacheStats),
    {
        let mut stats = match self.stats.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in update_stats operation, recovering");
                poisoned.into_inner()
            }
        };
        f(&mut stats);
    }

    pub fn cleanup(&self) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Cache: RwLock poisoned in cleanup operation, recovering");
                poisoned.into_inner()
            }
        };
        let before = entries.len();

        entries.retain(|_, entry| !entry.is_expired(self.ttl));

        let removed = before.saturating_sub(entries.len());
        self.update_stats(|s| {
            s.evictions += removed as u64;
            s.size = entries.len();
        });
    }
}

impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            max_size: self.max_size,
            ttl: self.ttl,
            stats: Arc::clone(&self.stats),
        }
    }
}
