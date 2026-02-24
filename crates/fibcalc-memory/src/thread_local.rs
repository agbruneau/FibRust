//! Generic thread-local object pool.
//!
//! Provides `tl_acquire` and `tl_release` free functions for thread-local pooling.
//! These replace duplicated pool patterns in `fastdoubling` and `matrix`.

use std::cell::RefCell;

/// Acquire an object from a thread-local pool.
///
/// If the pool has an object, it is popped and `reset` is called on it.
/// Otherwise a new object is created via `factory`.
#[inline]
pub fn tl_acquire<T>(
    pool: &RefCell<Vec<T>>,
    factory: fn() -> T,
    reset: fn(&mut T),
) -> T {
    let mut pool = pool.borrow_mut();
    match pool.pop() {
        Some(mut item) => {
            reset(&mut item);
            item
        }
        None => factory(),
    }
}

/// Return an object to a thread-local pool.
///
/// If the pool has reached `max` capacity, the object is dropped.
#[inline]
pub fn tl_release<T>(pool: &RefCell<Vec<T>>, max: usize, item: T) {
    let mut pool = pool.borrow_mut();
    if pool.len() < max {
        pool.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tl_acquire_creates_new_when_empty() {
        let pool: RefCell<Vec<i32>> = RefCell::new(Vec::new());
        let val = tl_acquire(&pool, || 42, |_| {});
        assert_eq!(val, 42);
        assert_eq!(pool.borrow().len(), 0);
    }

    #[test]
    fn tl_acquire_reuses_and_resets() {
        let pool: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
        pool.borrow_mut().push(vec![1, 2, 3]);

        let v = tl_acquire(&pool, Vec::new, |v| v.clear());
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

        let s = tl_acquire(&pool, || String::from("hello"), |s| s.clear());
        assert_eq!(s, "hello");

        tl_release(&pool, 4, s);
        assert_eq!(pool.borrow().len(), 1);

        let s2 = tl_acquire(&pool, || String::from("world"), |s| s.clear());
        assert!(s2.is_empty()); // Reused and reset
        assert_eq!(pool.borrow().len(), 0);
    }
}
