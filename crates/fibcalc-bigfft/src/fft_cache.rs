//! Thread-safe LRU cache for FFT transforms.

use std::collections::HashMap;

use parking_lot::Mutex;

/// Cache key for FFT transforms.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub size: usize,
    pub shift: usize,
}

/// Thread-safe LRU cache for FFT root tables.
pub struct FFTCache {
    cache: Mutex<HashMap<CacheKey, Vec<Vec<u64>>>>,
    max_entries: usize,
}

impl FFTCache {
    /// Create a new FFT cache with the given maximum entries.
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            max_entries,
        }
    }

    /// Get a cached transform, if available.
    pub fn get(&self, key: &CacheKey) -> Option<Vec<Vec<u64>>> {
        self.cache.lock().get(key).cloned()
    }

    /// Store a transform in the cache.
    pub fn put(&self, key: CacheKey, value: Vec<Vec<u64>>) {
        let mut cache = self.cache.lock();
        if cache.len() >= self.max_entries {
            // Simple eviction: clear all (LRU would be more sophisticated)
            cache.clear();
        }
        cache.insert(key, value);
    }

    /// Get the number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.lock().len()
    }

    /// Check if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.lock().is_empty()
    }

    /// Clear the cache.
    pub fn clear(&self) {
        self.cache.lock().clear();
    }
}

impl Default for FFTCache {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_put_get() {
        let cache = FFTCache::new(10);
        let key = CacheKey { size: 8, shift: 64 };
        cache.put(key.clone(), vec![vec![1, 2, 3]]);
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn cache_eviction() {
        let cache = FFTCache::new(2);
        for i in 0..3 {
            cache.put(CacheKey { size: i, shift: 64 }, vec![]);
        }
        // After exceeding max, cache should have been cleared + 1 new entry
        assert!(cache.len() <= 2);
    }

    #[test]
    fn cache_miss() {
        let cache = FFTCache::new(10);
        let key = CacheKey {
            size: 99,
            shift: 128,
        };
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn cache_default() {
        let cache = FFTCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_is_empty_after_creation() {
        let cache = FFTCache::new(5);
        assert!(cache.is_empty());
    }

    #[test]
    fn cache_not_empty_after_put() {
        let cache = FFTCache::new(5);
        cache.put(CacheKey { size: 1, shift: 1 }, vec![vec![1]]);
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_clear() {
        let cache = FFTCache::new(10);
        cache.put(CacheKey { size: 1, shift: 1 }, vec![vec![1]]);
        cache.put(CacheKey { size: 2, shift: 2 }, vec![vec![2]]);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_get_returns_correct_value() {
        let cache = FFTCache::new(10);
        let key = CacheKey { size: 4, shift: 32 };
        let value = vec![vec![10, 20], vec![30, 40]];
        cache.put(key.clone(), value.clone());

        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn cache_overwrite_same_key() {
        let cache = FFTCache::new(10);
        let key = CacheKey { size: 4, shift: 32 };
        cache.put(key.clone(), vec![vec![1]]);
        cache.put(key.clone(), vec![vec![2]]);

        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved, vec![vec![2]]);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_eviction_clears_all_then_inserts() {
        let cache = FFTCache::new(2);
        let key1 = CacheKey { size: 1, shift: 1 };
        let key2 = CacheKey { size: 2, shift: 2 };
        cache.put(key1.clone(), vec![vec![1]]);
        cache.put(key2.clone(), vec![vec![2]]);
        assert_eq!(cache.len(), 2);

        // This triggers eviction (clear) then insert
        let key3 = CacheKey { size: 3, shift: 3 };
        cache.put(key3.clone(), vec![vec![3]]);

        // Old entries should be gone
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_none());
        // New entry should be present
        assert!(cache.get(&key3).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_multiple_different_keys() {
        let cache = FFTCache::new(100);
        for i in 0..50 {
            cache.put(
                CacheKey {
                    size: i,
                    shift: i * 2,
                },
                vec![vec![i as u64]],
            );
        }
        assert_eq!(cache.len(), 50);

        for i in 0..50 {
            let val = cache.get(&CacheKey {
                size: i,
                shift: i * 2,
            });
            assert!(val.is_some());
            assert_eq!(val.unwrap(), vec![vec![i as u64]]);
        }
    }

    #[test]
    fn cache_key_equality() {
        let k1 = CacheKey { size: 8, shift: 64 };
        let k2 = CacheKey { size: 8, shift: 64 };
        let k3 = CacheKey {
            size: 8,
            shift: 128,
        };
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn cache_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(FFTCache::new(1000));

        let mut handles = vec![];
        for t in 0..4 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    let key = CacheKey {
                        size: t * 100 + i,
                        shift: 64,
                    };
                    cache.put(key.clone(), vec![vec![i as u64]]);
                    let _ = cache.get(&key);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Cache should have entries and not have panicked
        assert!(cache.len() > 0);
    }

    #[test]
    fn cache_max_entries_one() {
        let cache = FFTCache::new(1);
        cache.put(CacheKey { size: 1, shift: 1 }, vec![]);
        assert_eq!(cache.len(), 1);

        // Adding second entry should trigger eviction
        cache.put(CacheKey { size: 2, shift: 2 }, vec![]);
        assert_eq!(cache.len(), 1);
    }
}
