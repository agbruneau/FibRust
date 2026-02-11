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
}
