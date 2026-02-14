//! Generic object pool with thread-safe and thread-local variants.
//!
//! Provides `ObjectPool<T>` (mutex-based, shareable across threads) and
//! free functions `tl_acquire` / `tl_release` for thread-local pooling.
//! These replace the duplicated `CalculationStatePool` and `MatrixStatePool`
//! patterns found in `fastdoubling` and `matrix`.

use std::cell::RefCell;

use parking_lot::Mutex;

/// A thread-safe object pool backed by a `Mutex<Vec<T>>`.
///
/// Objects are acquired with a factory + reset closure, and released back
/// into the pool up to `max_size`. When the pool is empty, `acquire`
/// creates a new object via the factory; when the pool is full, `release`
/// drops the object.
pub struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
    max_size: usize,
}

impl<T: Send> ObjectPool<T> {
    /// Create a new pool with the given maximum capacity.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// Acquire an object from the pool, or create a new one via `factory`.
    /// If an object is reused from the pool, `reset` is called on it first.
    pub fn acquire(&self, factory: impl FnOnce() -> T, reset: impl FnOnce(&mut T)) -> T {
        let mut pool = self.pool.lock();
        match pool.pop() {
            Some(mut item) => {
                reset(&mut item);
                item
            }
            None => factory(),
        }
    }

    /// Return an object to the pool for reuse. If the pool is at capacity,
    /// the object is dropped.
    pub fn release(&self, item: T) {
        let mut pool = self.pool.lock();
        if pool.len() < self.max_size {
            pool.push(item);
        }
    }

    /// Get the number of objects currently available in the pool.
    #[must_use]
    pub fn available(&self) -> usize {
        self.pool.lock().len()
    }
}

/// Acquire an object from a thread-local pool.
///
/// If the pool has an object, it is popped and `reset` is called on it.
/// Otherwise a new object is created via `factory`.
pub fn tl_acquire<T>(
    pool: &RefCell<Vec<T>>,
    max: usize,
    factory: fn() -> T,
    reset: fn(&mut T),
) -> T {
    let _ = max; // max is only used by tl_release; accepted here for API symmetry
    let mut pool = pool.borrow_mut();
    match pool.pop() {
        Some(mut item) => {
            reset(&mut item);
            item
        }
        None => factory(),
    }
}

/// Return an object to a thread-local pool. If the pool has reached `max`
/// capacity, the object is dropped.
pub fn tl_release<T>(pool: &RefCell<Vec<T>>, max: usize, item: T) {
    let mut pool = pool.borrow_mut();
    if pool.len() < max {
        pool.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- ObjectPool tests --

    #[test]
    fn pool_acquire_creates_new_when_empty() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(4);
        let v = pool.acquire(|| vec![1, 2, 3], |v| v.clear());
        assert_eq!(v, vec![1, 2, 3]);
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn pool_acquire_reuses_and_resets() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(4);
        pool.release(vec![10, 20, 30]);
        assert_eq!(pool.available(), 1);

        let v = pool.acquire(|| vec![99], |v| v.clear());
        // Should have been reset (cleared), not freshly created
        assert!(v.is_empty());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn pool_release_respects_max_size() {
        let pool: ObjectPool<u32> = ObjectPool::new(2);
        pool.release(1);
        pool.release(2);
        pool.release(3); // Dropped
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn pool_acquire_release_roundtrip() {
        let pool: ObjectPool<String> = ObjectPool::new(2);
        assert_eq!(pool.available(), 0);

        let s = pool.acquire(|| String::from("hello"), |s| s.clear());
        assert_eq!(s, "hello");

        pool.release(s);
        assert_eq!(pool.available(), 1);

        let s2 = pool.acquire(|| String::from("world"), |s| s.clear());
        // Reused from pool, so reset (cleared)
        assert!(s2.is_empty());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn pool_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ObjectPool<Vec<u8>>>();
    }

    #[test]
    fn pool_multithreaded_acquire_release() {
        use std::sync::Arc;
        let pool = Arc::new(ObjectPool::<u64>::new(8));
        let mut handles = vec![];

        for i in 0..4 {
            let p = Arc::clone(&pool);
            handles.push(std::thread::spawn(move || {
                let val = p.acquire(|| i, |_| {});
                p.release(val);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Items were released back; some threads may have reused others' items
        // so the count can vary, but it should be between 1 and 4
        let avail = pool.available();
        assert!(avail >= 1 && avail <= 4, "expected 1..=4, got {avail}");
    }

    // -- Thread-local pool tests --

    #[test]
    fn tl_acquire_creates_new_when_empty() {
        let pool: RefCell<Vec<i32>> = RefCell::new(Vec::new());
        let val = tl_acquire(&pool, 4, || 42, |_| {});
        assert_eq!(val, 42);
        assert_eq!(pool.borrow().len(), 0);
    }

    #[test]
    fn tl_acquire_reuses_and_resets() {
        let pool: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
        pool.borrow_mut().push(vec![1, 2, 3]);

        let v = tl_acquire(&pool, 4, Vec::new, |v| v.clear());
        assert!(v.is_empty()); // was reset
        assert_eq!(pool.borrow().len(), 0);
    }

    #[test]
    fn tl_release_respects_max_size() {
        let pool: RefCell<Vec<u32>> = RefCell::new(Vec::new());
        tl_release(&pool, 2, 1);
        tl_release(&pool, 2, 2);
        tl_release(&pool, 2, 3); // Dropped
        assert_eq!(pool.borrow().len(), 2);
    }

    #[test]
    fn tl_roundtrip() {
        let pool: RefCell<Vec<String>> = RefCell::new(Vec::new());

        let s = tl_acquire(&pool, 4, || String::from("hello"), |s| s.clear());
        assert_eq!(s, "hello");

        tl_release(&pool, 4, s);
        assert_eq!(pool.borrow().len(), 1);

        let s2 = tl_acquire(&pool, 4, || String::from("world"), |s| s.clear());
        assert!(s2.is_empty()); // Reused and reset
        assert_eq!(pool.borrow().len(), 0);
    }
}
