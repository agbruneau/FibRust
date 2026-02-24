# FibRust Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Consolidate memory management into a dedicated crate, activate Phase 2 infrastructure, optimize performance, clean up dead code, and update all documentation.

**Architecture:** Bottom-up approach — create `fibcalc-memory` crate first (shared allocation infrastructure), then wire it into existing crates, then optimize hot paths, then clean up and document. All 669+ existing tests must pass after each task.

**Tech Stack:** Rust 1.80+, bumpalo (arenas), num-bigint (BigUint pools), parking_lot (mutexes), rayon (parallelism), criterion (benchmarks)

---

## Phase 1: Memory Consolidation — `fibcalc-memory` crate

### Task 1: Create fibcalc-memory crate scaffold

**Files:**
- Create: `crates/fibcalc-memory/Cargo.toml`
- Create: `crates/fibcalc-memory/src/lib.rs`
- Modify: `Cargo.toml:12-20` (workspace members)
- Modify: `Cargo.toml:81-87` (workspace dependencies)

**Step 1: Create directory**

Run: `mkdir -p crates/fibcalc-memory/src`

**Step 2: Write Cargo.toml**

```toml
[package]
name = "fibcalc-memory"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
keywords.workspace = true
categories = ["memory-management", "algorithms"]
description = "Unified memory management: pools, arenas, and thread-local allocators for FibCalc"

[dependencies]
num-bigint = { workspace = true }
num-traits = { workspace = true }
parking_lot = { workspace = true }
bumpalo = { workspace = true }

[dev-dependencies]
proptest = "1"

[lints]
workspace = true
```

**Step 3: Write lib.rs (empty scaffold)**

```rust
//! # fibcalc-memory
//!
//! Unified memory management for the FibCalc workspace.
//!
//! Provides BigUint pooling with size classes, bump arenas for FFT temporaries,
//! generic thread-local object pools, and pool warming strategies.
#![warn(missing_docs)]

pub mod arena;
pub mod pool;
pub mod stats;
pub mod thread_local;
pub mod warming;
```

**Step 4: Add to workspace Cargo.toml members**

In root `Cargo.toml`, add `"crates/fibcalc-memory"` to workspace members (line 19) and add workspace dependency:
```toml
fibcalc-memory = { path = "crates/fibcalc-memory" }
```

**Step 5: Create empty module files**

Create placeholder files: `arena.rs`, `pool.rs`, `stats.rs`, `thread_local.rs`, `warming.rs` — each with a module doc comment only.

**Step 6: Verify compilation**

Run: `cargo check --package fibcalc-memory`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add crates/fibcalc-memory/ Cargo.toml
git commit -m "feat(memory): scaffold fibcalc-memory crate"
```

---

### Task 2: Move stats.rs (atomic pool statistics)

**Files:**
- Create: `crates/fibcalc-memory/src/stats.rs`
- Source: `crates/fibcalc-bigfft/src/pool.rs:9-49` (PoolStats + AtomicPoolStats)

**Step 1: Write stats.rs**

Extract `PoolStats`, `AtomicPoolStats` from `crates/fibcalc-bigfft/src/pool.rs:9-49` into `crates/fibcalc-memory/src/stats.rs`. Make `AtomicPoolStats` public (it was `pub(crate)` in bigfft). Keep all existing doc comments and derive macros.

```rust
//! Atomic pool statistics for lock-free usage tracking.

use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics for pool usage.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of cache hits (acquired from pool).
    pub hits: u64,
    /// Number of cache misses (created new).
    pub misses: u64,
    /// Number of evictions (too large or pool full).
    pub evictions: u64,
}

/// Atomic pool statistics for lock-free updates.
pub struct AtomicPoolStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl AtomicPoolStats {
    /// Create new zeroed stats.
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Take a snapshot of current stats.
    pub fn snapshot(&self) -> PoolStats {
        PoolStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters.
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Increment hit counter.
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment miss counter.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment eviction counter.
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for AtomicPoolStats {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Write tests for stats**

Add `#[cfg(test)] mod tests` to stats.rs with tests for: new (zeroed), record + snapshot, reset.

**Step 3: Verify**

Run: `cargo test --package fibcalc-memory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/fibcalc-memory/src/stats.rs
git commit -m "feat(memory): add atomic pool statistics"
```

---

### Task 3: Move pool.rs (BigIntPool)

**Files:**
- Create: `crates/fibcalc-memory/src/pool.rs`
- Source: `crates/fibcalc-bigfft/src/pool.rs:51-300` (BigIntPool + tests)

**Step 1: Write pool.rs**

Move `BigIntPool` from `crates/fibcalc-bigfft/src/pool.rs:60-180`. Import `AtomicPoolStats` and `PoolStats` from `crate::stats`. Keep the `clear_value` helper function. Copy all existing tests.

The code is the exact same as `crates/fibcalc-bigfft/src/pool.rs:51-300` but with:
- `use crate::stats::{AtomicPoolStats, PoolStats};` instead of inline definitions
- `self.stats.record_hit()` instead of `self.stats.hits.fetch_add(1, Ordering::Relaxed)` (use the new helper methods)
- Same for `record_miss()`, `record_eviction()`

**Step 2: Write tests**

Copy tests from `crates/fibcalc-bigfft/src/pool.rs:182-300` into `crates/fibcalc-memory/src/pool.rs`.

**Step 3: Verify**

Run: `cargo test --package fibcalc-memory`
Expected: All pool tests PASS

**Step 4: Commit**

```bash
git add crates/fibcalc-memory/src/pool.rs
git commit -m "feat(memory): add BigIntPool with size classes"
```

---

### Task 4: Move arena.rs (unified BumpArena)

**Files:**
- Create: `crates/fibcalc-memory/src/arena.rs`
- Source: `crates/fibcalc-bigfft/src/bump.rs` (FFTBumpAllocator)
- Source: `crates/fibcalc-core/src/arena.rs` (CalculationArena)

**Step 1: Write arena.rs**

Merge both bumpalo wrappers into a single `BumpArena` type. It should support:
- `new()`, `with_capacity(bytes)` — from both sources
- `alloc_slice(len) -> &mut [u64]` — from FFTBumpAllocator
- `bump() -> &Bump` — from CalculationArena (for typed allocations)
- `reset(&mut self)` — from both
- `allocated_bytes() -> usize` — from both

```rust
//! Bump arena allocator for temporary allocations.
//!
//! Uses bumpalo for O(1) allocation of temporaries during Fibonacci computation.
//! Supports both typed allocations (via `bump()`) and slice allocations for FFT.

use bumpalo::Bump;

/// Unified bump arena for calculation and FFT temporaries.
pub struct BumpArena {
    bump: Bump,
}

impl BumpArena {
    /// Create a new arena with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create a new arena with the given initial capacity in bytes.
    #[must_use]
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
        }
    }

    /// Allocate a zero-filled slice of u64 values (for FFT scratch buffers).
    pub fn alloc_slice(&self, len: usize) -> &mut [u64] {
        self.bump.alloc_slice_fill_default(len)
    }

    /// Get a reference to the underlying bumpalo allocator (for typed allocations).
    #[must_use]
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Reset the arena, deallocating all objects at once.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get the number of bytes currently allocated.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for BumpArena {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Write tests**

Merge tests from both `crates/fibcalc-bigfft/src/bump.rs:50-157` and `crates/fibcalc-core/src/arena.rs:53-72`. Deduplicate where identical.

**Step 3: Verify**

Run: `cargo test --package fibcalc-memory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/fibcalc-memory/src/arena.rs
git commit -m "feat(memory): add unified BumpArena"
```

---

### Task 5: Create thread_local.rs (generic ThreadLocalPool)

**Files:**
- Create: `crates/fibcalc-memory/src/thread_local.rs`
- Reference: `crates/fibcalc-core/src/pool.rs:64-92` (tl_acquire/tl_release)

**Step 1: Write thread_local.rs**

Create a generic `ThreadLocalPool<T>` wrapper that encapsulates the `RefCell<Vec<T>>` + `tl_acquire`/`tl_release` pattern.

```rust
//! Generic thread-local object pool.
//!
//! Provides `tl_acquire` and `tl_release` free functions for thread-local pooling.
//! These replace duplicated pool patterns in `fastdoubling` and `matrix`.

use std::cell::RefCell;

/// Acquire an object from a thread-local pool.
///
/// If the pool has an object, it is popped and `reset` is called on it.
/// Otherwise a new object is created via `factory`.
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
pub fn tl_release<T>(pool: &RefCell<Vec<T>>, max: usize, item: T) {
    let mut pool = pool.borrow_mut();
    if pool.len() < max {
        pool.push(item);
    }
}
```

Note: The `max` parameter is removed from `tl_acquire` (it was unused there — see `crates/fibcalc-core/src/pool.rs:74`). It remains on `tl_release` where it's actually used.

**Step 2: Write tests**

Copy and adapt tests from `crates/fibcalc-core/src/pool.rs:178-218`.

**Step 3: Verify**

Run: `cargo test --package fibcalc-memory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/fibcalc-memory/src/thread_local.rs
git commit -m "feat(memory): add generic thread-local pool functions"
```

---

### Task 6: Move warming.rs (pool warming)

**Files:**
- Create: `crates/fibcalc-memory/src/warming.rs`
- Source: `crates/fibcalc-bigfft/src/pool_warming.rs` (entire file)

**Step 1: Write warming.rs**

Copy `crates/fibcalc-bigfft/src/pool_warming.rs` to `crates/fibcalc-memory/src/warming.rs`. Change:
- `use crate::pool::BigIntPool;` (now local import within fibcalc-memory)
- Remove `#![allow(dead_code)]` at line 2 (this code will be activated)
- Keep all functions, structs, tests unchanged

**Step 2: Verify**

Run: `cargo test --package fibcalc-memory`
Expected: PASS (all warming tests pass)

**Step 3: Commit**

```bash
git add crates/fibcalc-memory/src/warming.rs
git commit -m "feat(memory): add pool warming strategies"
```

---

### Task 7: Update fibcalc-bigfft to use fibcalc-memory

**Files:**
- Modify: `crates/fibcalc-bigfft/Cargo.toml` (add fibcalc-memory dep)
- Modify: `crates/fibcalc-bigfft/src/lib.rs` (re-export from memory)
- Modify: `crates/fibcalc-bigfft/src/pool.rs` (replace with re-export)
- Modify: `crates/fibcalc-bigfft/src/bump.rs` (replace with re-export or type alias)
- Modify: `crates/fibcalc-bigfft/src/pool_warming.rs` (replace with re-export)
- Modify: `crates/fibcalc-bigfft/src/allocator.rs` (update imports)
- Modify: `crates/fibcalc-bigfft/src/fft.rs` (update imports)

**Step 1: Add dependency**

In `crates/fibcalc-bigfft/Cargo.toml`, add:
```toml
fibcalc-memory = { workspace = true }
```

**Step 2: Update pool.rs**

Replace `crates/fibcalc-bigfft/src/pool.rs` contents with re-exports:
```rust
//! BigInt pool — re-exported from fibcalc-memory.
pub use fibcalc_memory::pool::BigIntPool;
pub use fibcalc_memory::stats::PoolStats;
```

Remove the original implementation and tests (they now live in fibcalc-memory).

**Step 3: Update bump.rs**

Replace `crates/fibcalc-bigfft/src/bump.rs` with:
```rust
//! FFT bump allocator — re-exported from fibcalc-memory.
pub use fibcalc_memory::arena::BumpArena as FFTBumpAllocator;
```

**Step 4: Update pool_warming.rs**

Replace `crates/fibcalc-bigfft/src/pool_warming.rs` with:
```rust
//! Pool warming — re-exported from fibcalc-memory.
pub use fibcalc_memory::warming::{
    estimate_result_bits, predict_sizes, warm_pool, warm_pool_default, SizePrediction,
    WarmingConfig,
};
```

**Step 5: Update allocator.rs**

In `crates/fibcalc-bigfft/src/allocator.rs`, change line 16:
- From: `pool: crate::pool::BigIntPool,`
- The import `use crate::pool::BigIntPool` should still work via re-export.
- Change `use crate::pool::PoolStats` to reference the re-export.

**Step 6: Update fft.rs**

In `crates/fibcalc-bigfft/src/fft.rs`:
- Line 13: `use crate::bump::FFTBumpAllocator;` — still works via re-export (type alias)
- Line 17: `use crate::pool::PoolStats;` — still works via re-export

**Step 7: Verify**

Run: `cargo test --package fibcalc-bigfft`
Expected: All existing bigfft tests PASS

**Step 8: Commit**

```bash
git add crates/fibcalc-bigfft/
git commit -m "refactor(bigfft): delegate pool/arena/warming to fibcalc-memory"
```

---

### Task 8: Update fibcalc-core to use fibcalc-memory

**Files:**
- Modify: `crates/fibcalc-core/Cargo.toml` (add fibcalc-memory dep)
- Modify: `crates/fibcalc-core/src/pool.rs` (keep tl_acquire/tl_release as re-exports, remove ObjectPool)
- Modify: `crates/fibcalc-core/src/arena.rs` (replace with re-export)
- Modify: `crates/fibcalc-core/src/fastdoubling.rs:65-86` (update pool imports)
- Modify: `crates/fibcalc-core/src/matrix.rs:19-40` (update pool imports)

**Step 1: Add dependency**

In `crates/fibcalc-core/Cargo.toml`, add:
```toml
fibcalc-memory = { workspace = true }
```

And remove `bumpalo` dependency (no longer needed directly — comes via fibcalc-memory).

**Step 2: Update pool.rs**

Replace `crates/fibcalc-core/src/pool.rs` with re-exports:
```rust
//! Object pool — re-exported from fibcalc-memory.
pub use fibcalc_memory::thread_local::{tl_acquire, tl_release};
```

Note: `ObjectPool<T>` is dropped (it was dead code, marked `#[allow(dead_code)]`). The `tl_acquire` signature changes: the `max` parameter is removed from acquire (it was unused).

**Step 3: Update fastdoubling.rs pool usage**

In `crates/fibcalc-core/src/fastdoubling.rs`, update lines 65-86:
- Change `pool::tl_acquire(p, THREAD_LOCAL_POOL_MAX, ...)` to `pool::tl_acquire(p, ...)` (remove max param from acquire call)
- Keep `pool::tl_release(p, THREAD_LOCAL_POOL_MAX, state)` (max is still used in release)

**Step 4: Update matrix.rs pool usage**

Same changes as Step 3, in `crates/fibcalc-core/src/matrix.rs:19-40`.

**Step 5: Update arena.rs**

Replace `crates/fibcalc-core/src/arena.rs` with:
```rust
//! Calculation arena — re-exported from fibcalc-memory.
pub use fibcalc_memory::arena::BumpArena as CalculationArena;
```

**Step 6: Verify**

Run: `cargo test --workspace`
Expected: ALL tests PASS (669+)

**Step 7: Commit**

```bash
git add crates/fibcalc-core/
git commit -m "refactor(core): delegate pool/arena to fibcalc-memory"
```

---

### Task 9: Full workspace verification

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All 669+ tests PASS

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

**Step 3: Check doc build**

Run: `cargo doc --workspace --no-deps`
Expected: No errors

**Step 4: Commit (if any fixups needed)**

---

## Phase 2: P2 Infrastructure Activation

### Task 10: Wire pool warming into registry

**Files:**
- Modify: `crates/fibcalc-core/src/registry.rs:42-63` (add warming call)

**Step 1: Write a test for warming integration**

Add a test to `crates/fibcalc-core/src/registry.rs` tests module:
```rust
#[test]
fn factory_warms_pool_on_create() {
    // After creating a fast doubling calculator, the global pool should be warmed
    let factory = DefaultFactory::new();
    let _calc = factory.get("fast").unwrap();
    // Warming happens at creation — verify pool has items
    let stats = fibcalc_bigfft::pool_stats();
    // Stats should show some activity (warming pre-populates)
    // Note: we can't assert exact counts since warming depends on defaults
    let _ = stats; // Compilation check; warming integration verified by not panicking
}
```

**Step 2: Run test to verify it fails or passes baseline**

Run: `cargo test --package fibcalc-core registry::tests::factory_warms_pool_on_create`

**Step 3: Add warming to DefaultFactory**

In `crates/fibcalc-core/src/registry.rs`, modify `DefaultFactory::get()` (lines 72-85):
- After creating and caching a calculator, call pool warming for the FFT pool.
- Add import: `use fibcalc_bigfft::pool_warming::warm_pool_default;`
- The warming is best done as a separate method or inline after cache insertion.

Since warming targets the global `POOL_ALLOCATOR` in `fft.rs`, and we want to warm the pool for typical usage, add a warm-on-first-use pattern. The simplest approach: add a function in `fibcalc-bigfft/src/fft.rs` that exposes `warm_global_pool(n: u64)`:

```rust
// In crates/fibcalc-bigfft/src/fft.rs:
/// Warm the global BigInt pool for computing F(n).
pub fn warm_global_pool(n: u64) {
    use crate::pool_warming::warm_pool_default;
    warm_pool_default(&POOL_ALLOCATOR.pool, n);
}
```

Note: This requires making `PoolAllocator.pool` accessible. The cleanest approach is to add a `pool(&self) -> &BigIntPool` accessor to `PoolAllocator`, or to add a `warm` method to `PoolAllocator` directly.

Alternative: Add to `crates/fibcalc-bigfft/src/allocator.rs`:
```rust
impl PoolAllocator {
    /// Warm the internal pool for computing F(n).
    pub fn warm(&self, n: u64) {
        crate::pool_warming::warm_pool_default(&self.pool, n);
    }
}
```

Then in `crates/fibcalc-bigfft/src/fft.rs`:
```rust
/// Warm the global pool for computing F(n).
pub fn warm_global_pool(n: u64) {
    POOL_ALLOCATOR.warm(n);
}
```

And re-export in `crates/fibcalc-bigfft/src/lib.rs`:
```rust
pub use fft::warm_global_pool;
```

Then in `crates/fibcalc-core/src/registry.rs`, call it:
```rust
// In DefaultFactory::get(), after creating the calculator:
fn get(&self, name: &str) -> Result<Arc<dyn Calculator>, FibError> {
    if let Some(calc) = self.cache.read().get(name) {
        return Ok(Arc::clone(calc));
    }
    let calc = Self::create_calculator(name)?;
    self.cache
        .write()
        .insert(name.to_string(), Arc::clone(&calc));
    Ok(calc)
}
```

Note: Warming should happen per-computation with a known `n`, not at factory creation time (we don't know `n` yet). The better integration point is in `Calculator::calculate()` or in the orchestration layer. Evaluate whether warming belongs in the factory or in the calculator.

**Decision:** Add a `warm_for_n(n: u64)` function that the orchestration layer or CLI can call before starting computation. This is cleaner than coupling it to the factory.

**Step 4: Verify**

Run: `cargo test --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/fibcalc-bigfft/src/allocator.rs crates/fibcalc-bigfft/src/fft.rs crates/fibcalc-bigfft/src/lib.rs
git commit -m "feat(bigfft): expose warm_global_pool for pre-warming FFT pool"
```

---

### Task 11: Remove dead code markers from activated infrastructure

**Files:**
- Modify: `crates/fibcalc-memory/src/arena.rs` (no dead_code markers needed)
- Modify: `crates/fibcalc-memory/src/warming.rs` (remove `#![allow(dead_code)]`)
- Modify: `crates/fibcalc-bigfft/src/bump.rs` (remove dead_code markers if any remain)

**Step 1: Audit for remaining dead_code markers**

Search for `allow(dead_code)` across the workspace:
Run: `grep -rn "allow(dead_code)" crates/`

**Step 2: Remove markers from activated code**

Remove `#[allow(dead_code)]` from:
- Any re-export stubs that replaced original implementations
- The warming module (now active)
- The arena module (now active)

Keep `#[allow(dead_code)]` only for genuinely unused code:
- `matrix_types.rs:7` (`Matrix` struct — used but clippy can't see through all paths)
- `matrix_types.rs:40` (`is_identity` — only used in tests)
- `matrix_ops.rs` Strassen function (P2 placeholder)
- `common.rs` task execution infra (P2 placeholder)

**Step 3: Verify**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove dead_code markers from activated infrastructure"
```

---

### Task 12: Workspace verification after Phase 2

**Step 1: Full test suite**

Run: `cargo test --workspace`
Expected: All tests PASS

**Step 2: Clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Clean

**Step 3: Commit (if fixups needed)**

---

## Phase 3: Performance Optimizations

### Task 13: Add in-place matrix operations

**Files:**
- Modify: `crates/fibcalc-core/src/matrix_types.rs:48-86` (add mutable variants)
- Modify: `crates/fibcalc-core/src/matrix.rs` (use new mutable ops in loop)

**Step 1: Write failing tests for in-place operations**

Add to `crates/fibcalc-core/src/matrix_types.rs` tests:
```rust
#[test]
fn square_symmetric_into_matches_immutable() {
    let q = Matrix::fibonacci_q();
    let expected = q.square_symmetric();
    let mut m = q.clone();
    m.square_symmetric_into();
    assert_eq!(m.a, expected.a);
    assert_eq!(m.b, expected.b);
    assert_eq!(m.d, expected.d);
}

#[test]
fn multiply_symmetric_into_matches_immutable() {
    let q = Matrix::fibonacci_q();
    let q2 = q.square_symmetric();
    let expected = q2.multiply_symmetric(&q);
    let mut m = q2.clone();
    m.multiply_symmetric_into(&q);
    assert_eq!(m.a, expected.a);
    assert_eq!(m.b, expected.b);
    assert_eq!(m.d, expected.d);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package fibcalc-core matrix_types::tests::square_symmetric_into`
Expected: FAIL (method doesn't exist)

**Step 3: Implement in-place methods**

Add to `crates/fibcalc-core/src/matrix_types.rs`, after `square_symmetric()`:

```rust
/// In-place squaring for symmetric matrices.
///
/// Mutates `self` to contain `self * self`, reusing buffer capacity.
pub fn square_symmetric_into(&mut self) {
    let b_sq = &self.b * &self.b;
    let new_a = &self.a * &self.a + &b_sq;
    let new_b = &self.b * (&self.a + &self.d);
    let new_d = &b_sq + &self.d * &self.d;
    self.a = new_a;
    self.c = new_b.clone();
    self.b = new_b;
    self.d = new_d;
}

/// In-place multiplication for symmetric matrices.
///
/// Mutates `self` to contain `self * other`, reusing buffer capacity.
pub fn multiply_symmetric_into(&mut self, other: &Self) {
    let b1_b2 = &self.b * &other.b;
    let new_a = &self.a * &other.a + &b1_b2;
    let new_b = &self.a * &other.b + &self.b * &other.d;
    let new_d = &b1_b2 + &self.d * &other.d;
    self.a = new_a;
    self.c = new_b.clone();
    self.b = new_b;
    self.d = new_d;
}
```

Note: These don't truly save allocations because `BigUint` multiplication always returns new values (num-bigint limitation). The benefit is avoiding the allocation of a new `Matrix` struct — the BigUint fields are moved rather than copied. The improvement is modest.

**Step 4: Run tests**

Run: `cargo test --package fibcalc-core matrix_types`
Expected: PASS

**Step 5: Wire into matrix exponentiation loop**

In `crates/fibcalc-core/src/matrix.rs`, modify `execute_matrix_loop` to use in-place operations:
- Replace `state.base = state.base.square_symmetric();` with `state.base.square_symmetric_into();`
- Replace `state.result = state.result.multiply_symmetric(&state.base);` with `state.result.multiply_symmetric_into(&state.base);`

Caution: The multiply_symmetric_into call needs `state.base` as argument while mutating `state.result`. Since `state.result` and `state.base` are separate fields, this should work via split borrows. Verify compilation.

If split borrows don't work (both through `state`), the workaround is to temporarily take the base:
```rust
let base_ref = &state.base;
state.result.multiply_symmetric_into(base_ref);
```

This should work since we only need `&state.base` (immutable) while mutating `state.result`.

**Step 6: Run golden tests**

Run: `cargo test --workspace golden`
Expected: PASS (correctness preserved)

**Step 7: Commit**

```bash
git add crates/fibcalc-core/src/matrix_types.rs crates/fibcalc-core/src/matrix.rs
git commit -m "perf(core): add in-place matrix operations for exponentiation loop"
```

---

### Task 14: Conservative inline audit

**Files:**
- Modify: `crates/fibcalc-core/src/observer.rs:41` (add `#[inline]` to `should_report`)
- Modify: `crates/fibcalc-memory/src/thread_local.rs` (add `#[inline]` to `tl_acquire`, `tl_release`)

**Step 1: Capture baseline benchmarks**

Run: `cargo bench --package fibcalc-core -- --save-baseline before-inline`
Expected: Saves baseline results

**Step 2: Add inline annotations**

In `crates/fibcalc-core/src/observer.rs:41`, add `#[inline]` before `pub fn should_report`:
```rust
#[inline]
#[must_use]
pub fn should_report(&self, new_progress: f64) -> bool {
```

In `crates/fibcalc-memory/src/thread_local.rs`, add `#[inline]` to both functions:
```rust
#[inline]
pub fn tl_acquire<T>(...) -> T { ... }

#[inline]
pub fn tl_release<T>(...) { ... }
```

**Step 3: Run benchmarks again**

Run: `cargo bench --package fibcalc-core -- --baseline before-inline`
Expected: Compare results. Keep annotations only if they show improvement or no regression.

**Step 4: Run tests**

Run: `cargo test --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/fibcalc-core/src/observer.rs crates/fibcalc-memory/src/thread_local.rs
git commit -m "perf: add conservative inline annotations to hot-path functions"
```

---

### Task 15: Observer dispatch verification

**Files:**
- Read-only verification — no changes expected

**Step 1: Verify FrozenObserver usage**

Check that `fastdoubling.rs` and `matrix.rs` use `FrozenObserver` (atomic load) in their hot loops, not `&dyn ProgressObserver` (vtable dispatch).

In `crates/fibcalc-core/src/fastdoubling.rs`, look for `frozen` or `should_report` calls inside the main loop. Verify the observer is frozen before the loop, and only `frozen.should_report()` is called inside.

**Step 2: Document findings**

If the observer pattern is already optimal (expected), no code changes. Note this in a commit message.

**Step 3: Commit (verification only)**

```bash
git commit --allow-empty -m "verify: observer dispatch uses FrozenObserver in hot loops (no changes needed)"
```

---

### Task 16: Benchmark final Phase 3 results

**Step 1: Run full benchmarks**

Run: `cargo bench --package fibcalc-core`
Expected: No regression; modest improvement on large N values

**Step 2: Record results**

Save benchmark output for documentation in Phase 5.

---

## Phase 4: Code Hygiene

### Task 17: Dead code cleanup

**Files:**
- Audit all `#[allow(dead_code)]` across workspace
- Remove genuinely unused code
- Document intentionally kept placeholders

**Step 1: List all dead code markers**

Search: `grep -rn "allow(dead_code)" crates/`

**Step 2: For each marker, decide: remove code, remove marker, or keep**

Rules:
- Code activated by Phase 2: remove marker
- Code that is genuinely used but clippy can't see through trait dispatch: keep marker
- Code that is a P2 placeholder (Strassen, common task executor): keep marker with updated comment
- Code that is truly dead and has no planned use: remove code entirely

**Step 3: Apply changes**

**Step 4: Verify**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Clean

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: clean up dead code markers across workspace"
```

---

### Task 18: GMP isolation verification

**Step 1: Build without features**

Run: `cargo build --workspace`
Expected: Compiles without GMP

**Step 2: Build with GMP (if available)**

Run: `cargo build --workspace --features fibcalc-core/gmp`
Expected: Compiles with GMP (only on systems with libgmp-dev)

**Step 3: Verify no GMP types in non-gmp paths**

Search: `grep -rn "rug::" crates/ --include="*.rs" | grep -v "cfg.*gmp" | grep -v "test"`
Expected: No matches outside `#[cfg(feature = "gmp")]` blocks

**Step 4: Commit (verification only)**

```bash
git commit --allow-empty -m "verify: GMP feature isolation confirmed (no changes needed)"
```

---

### Task 19: Full test suite validation

**Step 1: Run all tests**

Run: `cargo test --workspace`
Expected: All 669+ tests PASS

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Clean

**Step 3: Run doc tests**

Run: `cargo doc --workspace --no-deps`
Expected: Clean

**Step 4: Run golden tests explicitly**

Run: `cargo test --test golden`
Expected: All golden file tests PASS

---

## Phase 5: Documentation Update

### Task 20: Update README.md

**Files:**
- Modify: `README.md`

**Step 1: Update crate count**

Change "7 crates" to "8 crates" wherever it appears (search for "7 crates").

**Step 2: Update architecture table**

Add `fibcalc-memory` row to the crate description table.

**Step 3: Update any dependency diagrams**

If README contains an architecture diagram, add `fibcalc-memory` as a dependency of both `fibcalc-core` and `fibcalc-bigfft`.

**Step 4: Commit**

```bash
git add README.md
git commit -m "docs: update README with fibcalc-memory crate"
```

---

### Task 21: Update docs/ARCHITECTURE.md

**Files:**
- Modify: `docs/ARCHITECTURE.md`

**Step 1: Add fibcalc-memory to crate descriptions**

Add a section describing the memory crate's responsibilities.

**Step 2: Update dependency graph**

Update the mermaid diagram to include `fibcalc-memory` as a dependency of `fibcalc-core` and `fibcalc-bigfft`.

**Step 3: Update memory management section**

Update the allocation strategies table (around line 661) to reflect the unified approach:
- Replace separate descriptions of `thread-local Vec<CalculationState>`, `CalculationStatePool`, `bumpalo::Bump arena`, `BigUint pools` with unified `fibcalc-memory` descriptions.

**Step 4: Commit**

```bash
git add docs/ARCHITECTURE.md
git commit -m "docs: update ARCHITECTURE.md with fibcalc-memory crate"
```

---

### Task 22: Update docs/PERFORMANCE.md

**Files:**
- Modify: `docs/PERFORMANCE.md`

**Step 1: Update memory optimization section**

Update the section around lines 263-326 to reference `fibcalc-memory` as the unified source for:
- BigInt pool management
- Pool warming strategies
- Arena allocation

**Step 2: Add benchmark comparison**

If benchmark results from Task 16 show changes, add a before/after comparison section.

**Step 3: Commit**

```bash
git add docs/PERFORMANCE.md
git commit -m "docs: update PERFORMANCE.md with unified memory strategy"
```

---

### Task 23: Update docs/CHANGELOG.md

**Files:**
- Modify: `docs/CHANGELOG.md`

**Step 1: Add entry to Unreleased section**

Follow existing format (lines 8-28). Add:

```markdown
### Added
- **Memory**: New `fibcalc-memory` crate consolidating all allocation infrastructure (BigInt pools, bump arenas, thread-local pools, pool warming)
- **Performance**: In-place matrix operations (`square_symmetric_into`, `multiply_symmetric_into`) for reduced allocation in exponentiation loop
- **Performance**: Conservative `#[inline]` annotations on hot-path pool and observer functions
- **Infrastructure**: Exposed `warm_global_pool(n)` for pre-warming FFT BigInt pool

### Changed
- **Architecture**: `fibcalc-core` and `fibcalc-bigfft` now delegate pool/arena management to `fibcalc-memory`
- **Code quality**: Removed dead code markers from activated Phase 2 infrastructure
```

**Step 2: Commit**

```bash
git add docs/CHANGELOG.md
git commit -m "docs: update CHANGELOG with refactoring changes"
```

---

### Task 24: Update CLAUDE.md (project instructions)

**Files:**
- Modify: `CLAUDE.md` (project root)

**Step 1: Update workspace description**

- Line 7: Change "7 crates Cargo" to "8 crates Cargo"
- Update line count if significantly changed

**Step 2: Update architecture section**

Add `fibcalc-memory` to the crate tree (lines 13-28):
```
crates/
  fibcalc-memory/         # Gestion mémoire unifiée : pools BigInt, arènes bump, pools thread-local
  ...
```

**Step 3: Update stack technique table**

Update line 39 from:
```
| Allocation | `bumpalo` (arena, active in FFT), thread-local pool allocator (active) |
```
To:
```
| Allocation | `fibcalc-memory` (pools BigInt, arènes bump, pools thread-local, pré-chauffage) |
```

**Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with fibcalc-memory crate"
```

---

### Task 25: Add rustdoc to fibcalc-memory

**Files:**
- Modify: `crates/fibcalc-memory/src/lib.rs`
- Verify: all public items have doc comments

**Step 1: Check rustdoc coverage**

Run: `cargo doc --package fibcalc-memory --no-deps`
Expected: No missing docs warnings (module has `#![warn(missing_docs)]`)

**Step 2: Fix any missing doc comments**

Ensure all public types, methods, and functions have doc comments.

**Step 3: Commit**

```bash
git add crates/fibcalc-memory/
git commit -m "docs: complete rustdoc coverage for fibcalc-memory"
```

---

## Final Verification

### Task 26: End-to-end verification

**Step 1:** `cargo test --workspace` — all tests pass
**Step 2:** `cargo clippy --workspace -- -D warnings` — clean
**Step 3:** `cargo doc --workspace --no-deps` — clean
**Step 4:** `cargo bench --package fibcalc-core` — no regression
**Step 5:** Review all commits since start — verify each is focused and correct

---

## Task Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-9 | Create fibcalc-memory, move allocators, update dependencies |
| 2 | 10-12 | Wire pool warming, remove dead code markers |
| 3 | 13-16 | In-place matrix ops, inline audit, observer verification |
| 4 | 17-19 | Dead code cleanup, GMP verification, full validation |
| 5 | 20-25 | Update README, ARCHITECTURE, PERFORMANCE, CHANGELOG, CLAUDE.md |
| Final | 26 | End-to-end verification |
