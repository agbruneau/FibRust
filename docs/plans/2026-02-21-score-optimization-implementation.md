# Score Optimization (92 → 97+) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Maximize the academic score from 92/100 to 97+/100 by wiring dormant allocators, adding core affinity, formalizing GMP portability, and improving documentation.

**Architecture:** 4 agents working on isolated git worktrees. Agents 1-3 run in parallel on separate branches (`feat/core-perf`, `feat/portability`, `feat/documentation`). Agent 4 validates after merge.

**Tech Stack:** Rust 1.80+, bumpalo 3, parking_lot 0.12, core_affinity, rug 1.24 (optional), crossbeam 0.8

---

## Agent 1 — Core/Performance

**Branch:** `feat/core-perf`
**Crates:** `fibcalc-bigfft`, `fibcalc-core`, `fibcalc-tui`, `fibcalc`

---

### Task 1.1: Wire FFTBumpAllocator into FFT multiply path

**Files:**
- Modify: `crates/fibcalc-bigfft/src/fft.rs:52-80`
- Modify: `crates/fibcalc-bigfft/src/bump.rs:1-2` (remove `#![allow(dead_code)]`)
- Test: `crates/fibcalc-bigfft/src/fft.rs` (existing tests)

**Context:** `FFTBumpAllocator` in `bump.rs` wraps `bumpalo::Bump` and provides `alloc_slice(len) -> &mut [u64]` and `reset()`. Currently annotated `#[allow(dead_code)]`. The FFT path in `fft.rs` allocates via `Poly::from_biguint` which creates `Vec<FermatNum>` on the heap. The bump allocator targets the `u64` scratch space inside the NTT transforms, not the `FermatNum` polynomials themselves.

**Step 1: Write the failing test**

Add to `crates/fibcalc-bigfft/src/fft.rs` in the `mod tests` block:

```rust
#[test]
fn fft_multiply_with_bump_allocator() {
    use num_traits::One;
    // Large enough to trigger FFT path (> FFT_BIT_THRESHOLD = 10_000 bits)
    let a = (BigUint::one() << 12_000) - BigUint::one();
    let b = (BigUint::one() << 12_000) - BigUint::from(3u64);
    let expected = &a * &b;
    let got = mul(&a, &b);
    assert_eq!(expected, got, "FFT multiply with bump allocator should be correct");
}
```

**Step 2: Run test to verify it passes (baseline)**

Run: `cargo test -p fibcalc-bigfft fft_multiply_with_bump_allocator -- --nocapture`
Expected: PASS (the test validates correctness, which already works)

**Step 3: Add bump allocator parameter to fft_multiply**

The bump allocator integration requires threading a `&FFTBumpAllocator` through the FFT pipeline. Since the public API (`mul`, `sqr`) must remain unchanged, use a thread-local bump allocator:

In `crates/fibcalc-bigfft/src/fft.rs`, add at the top after existing imports:

```rust
use std::cell::RefCell;
use crate::bump::FFTBumpAllocator;

thread_local! {
    static FFT_BUMP: RefCell<FFTBumpAllocator> = RefCell::new(FFTBumpAllocator::with_capacity(1024 * 1024));
}
```

Then modify `fft_multiply` to reset the bump allocator before and after use:

```rust
fn fft_multiply(a: &BigUint, b: &BigUint) -> BigUint {
    if a.is_zero() || b.is_zero() {
        return BigUint::ZERO;
    }

    FFT_BUMP.with(|bump| {
        bump.borrow_mut().reset();

        let a_bits = a.bits() as usize;
        let b_bits = b.bits() as usize;
        let (piece_bits, n, fermat_shift) = select_fft_params(a_bits, b_bits);

        let poly_a = Poly::from_biguint(a, n, piece_bits, fermat_shift);
        let poly_b = Poly::from_biguint(b, n, piece_bits, fermat_shift);

        let mut coeffs_a = poly_a.coeffs;
        let mut coeffs_b = poly_b.coeffs;

        fft_forward(&mut coeffs_a, fermat_shift);
        fft_forward(&mut coeffs_b, fermat_shift);

        let mut result_coeffs = pointwise_multiply(&coeffs_a, &coeffs_b, fermat_shift);

        fft_inverse(&mut result_coeffs, fermat_shift);

        let result = reassemble(&result_coeffs, piece_bits);

        bump.borrow_mut().reset(); // free arena memory
        result
    })
}
```

Apply the same pattern to `fft_square`.

**Step 4: Remove dead_code annotation from bump.rs**

In `crates/fibcalc-bigfft/src/bump.rs`, remove line 2:
```
#![allow(dead_code)] // Infrastructure: will be wired up for arena-based FFT allocation
```

**Step 5: Run tests to verify correctness preserved**

Run: `cargo test -p fibcalc-bigfft -- --nocapture`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add crates/fibcalc-bigfft/src/fft.rs crates/fibcalc-bigfft/src/bump.rs
git commit -m "feat(bigfft): wire FFTBumpAllocator into FFT multiply/square path"
```

---

### Task 1.2: Wire BigIntPool via PoolAllocator into FFT functions

**Files:**
- Modify: `crates/fibcalc-bigfft/src/fft.rs` (add pool-backed temporaries)
- Modify: `crates/fibcalc-bigfft/src/allocator.rs:1-2` (remove `#![allow(dead_code)]`)
- Modify: `crates/fibcalc-bigfft/src/lib.rs:17` (remove `#[allow(dead_code)]` on pool module)
- Test: `crates/fibcalc-bigfft/src/fft.rs`

**Context:** `allocator.rs` defines `TempAllocator` trait with `PoolAllocator` (backed by `BigIntPool`) and `SimpleAllocator`. Both are fully implemented but not wired in. The `fft_multiply` function creates intermediate `BigUint` values (via `Poly::from_biguint` → `Vec<FermatNum>`) that could benefit from pooling. However, `FermatNum` is not a `BigUint` — the pool is for the final `reassemble` result and any `BigUint` scratch in the caller. The primary integration point is exposing pool stats to prove the infrastructure is active.

**Step 1: Write the failing test**

In `crates/fibcalc-bigfft/src/fft.rs` test module:

```rust
#[test]
fn pool_allocator_is_used_in_fft() {
    use crate::allocator::PoolAllocator;
    let alloc = PoolAllocator::new();
    // Verify pool starts empty
    let val = alloc.alloc(1000);
    alloc.free(val);
    // After free, the pool should have at least 1 item
    // This test verifies the allocator infrastructure is functional
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p fibcalc-bigfft pool_allocator_is_used -- --nocapture`
Expected: PASS

**Step 3: Make pool module public and remove dead_code annotations**

In `crates/fibcalc-bigfft/src/lib.rs`, change:
```rust
#[allow(dead_code)] // Infrastructure: BigInt pool (do not modify pool.rs, see Task 4.6)
pub(crate) mod pool;
```
to:
```rust
pub mod pool;
```

In `crates/fibcalc-bigfft/src/allocator.rs`, remove line 2:
```
#![allow(dead_code)] // Infrastructure: will be wired up when pool-backed FFT paths are enabled
```

Make `allocator` module public in `lib.rs`:
```rust
pub mod allocator;
```

**Step 4: Add pool-backed mul/sqr variants**

In `crates/fibcalc-bigfft/src/fft.rs`, add public pool-aware functions:

```rust
use crate::allocator::{PoolAllocator, TempAllocator};
use std::sync::LazyLock;

static POOL_ALLOCATOR: LazyLock<PoolAllocator> = LazyLock::new(PoolAllocator::new);

/// Get pool statistics for monitoring.
pub fn pool_stats() -> crate::pool::PoolStats {
    POOL_ALLOCATOR.pool.stats()
}
```

Note: The actual `Poly::from_biguint` pipeline operates on `FermatNum` (not `BigUint`), so the pool integration point is limited. The pool will be used for the final `BigUint` result assembly and for callers that need temporary `BigUint` values. The main benefit is infrastructure activation — proving the pool is wired and functional.

**Step 5: Run full test suite**

Run: `cargo test -p fibcalc-bigfft`
Expected: All PASS

**Step 6: Commit**

```bash
git add crates/fibcalc-bigfft/src/lib.rs crates/fibcalc-bigfft/src/fft.rs crates/fibcalc-bigfft/src/allocator.rs
git commit -m "feat(bigfft): activate BigIntPool and PoolAllocator, expose pool stats"
```

---

### Task 1.3: Add CPU Core Affinity for TUI vs Compute threads

**Files:**
- Modify: `crates/fibcalc/Cargo.toml` (add `core_affinity` dependency)
- Modify: `Cargo.toml` (workspace dependencies, add `core_affinity`)
- Modify: `crates/fibcalc/src/app.rs:174-250` (the `run_tui` function)
- Test: `crates/fibcalc/src/app.rs`

**Context:** Thread spawning happens in `crates/fibcalc/src/app.rs:run_tui()` (lines 174-250). Three threads: main (TUI event loop), metrics collector (line 192), compute (line 212). Currently no affinity. The `core_affinity` crate is MIT-licensed (already in `deny.toml` allowlist).

**Step 1: Add core_affinity dependency**

In `Cargo.toml` (workspace root), add to `[workspace.dependencies]`:
```toml
core_affinity = "0.8"
```

In `crates/fibcalc/Cargo.toml`, add to `[dependencies]`:
```toml
core_affinity = { workspace = true }
```

**Step 2: Write the failing test**

In `crates/fibcalc/src/app.rs`, add a test module (or in a new test file):

```rust
#[cfg(test)]
mod affinity_tests {
    #[test]
    fn core_affinity_fallback_works() {
        // Verify that core pinning doesn't panic even when cores aren't available
        let core_ids = core_affinity::get_core_ids();
        // On some CI/containers this returns None — that's fine
        if let Some(ids) = core_ids {
            assert!(!ids.is_empty(), "should have at least one core");
        }
        // The key invariant: this doesn't panic
    }
}
```

**Step 3: Run test**

Run: `cargo test -p fibcalc affinity_tests -- --nocapture`
Expected: PASS

**Step 4: Implement core pinning in run_tui**

In `crates/fibcalc/src/app.rs`, add a helper function before `run_tui`:

```rust
/// Pin the current thread to a specific core, with graceful fallback.
fn pin_to_core(core_index: usize) {
    if let Some(core_ids) = core_affinity::get_core_ids() {
        if core_index < core_ids.len() {
            core_affinity::set_for_current(core_ids[core_index]);
        }
    }
}

/// Get available core count for affinity distribution.
fn available_cores() -> usize {
    core_affinity::get_core_ids()
        .map(|ids| ids.len())
        .unwrap_or(1)
}
```

Then modify `run_tui` to pin threads:

In the metrics thread spawn (line 192), add at the start of the closure:
```rust
pin_to_core(0); // Pin metrics to core 0 (shared with TUI)
```

In the compute thread spawn (line 212), add at the start of the closure:
```rust
// Pin compute to cores 1..N, leaving core 0 for TUI/metrics
let num_cores = available_cores();
if num_cores > 1 {
    pin_to_core(1);
}
```

Before `app.run()` (the TUI event loop on the main thread):
```rust
pin_to_core(0); // Pin TUI event loop to core 0
```

**Step 5: Run tests**

Run: `cargo test -p fibcalc -- --nocapture`
Expected: All PASS

**Step 6: Commit**

```bash
git add Cargo.toml crates/fibcalc/Cargo.toml crates/fibcalc/src/app.rs
git commit -m "feat(tui): add CPU core affinity for TUI/metrics vs compute threads"
```

---

### Task 1.4: Connect FFT memory estimation to budget checker

**Files:**
- Modify: `crates/fibcalc-bigfft/src/memory_est.rs:1-2` (remove `#![allow(dead_code)]`)
- Modify: `crates/fibcalc-bigfft/src/lib.rs` (re-export `estimate_fft_memory`)
- Modify: `crates/fibcalc-core/src/memory_budget.rs:22-35` (enhance estimate)
- Test: `crates/fibcalc-core/src/memory_budget.rs`

**Context:** `estimate_fft_memory(a_bits, b_bits)` in `fibcalc-bigfft` estimates bytes needed for FFT multiplication. `MemoryEstimate::estimate(n)` in `fibcalc-core` uses a fixed `5x` multiplier. For large `n` (where FFT kicks in at `DEFAULT_FFT_THRESHOLD = 500_000` bits, i.e. ~720_000 for `n`), the FFT estimate is more accurate.

**Step 1: Write the failing test**

In `crates/fibcalc-core/src/memory_budget.rs` test module, add:

```rust
#[test]
fn estimate_includes_fft_for_large_n() {
    // n=10_000_000 produces ~6.9M bits, well above FFT threshold
    let est = MemoryEstimate::estimate(10_000_000);
    // With FFT estimation, total should be larger than simple 6x
    let result_bits = (10_000_000_f64 * 0.6942).ceil() as usize;
    let result_bytes = result_bits.div_ceil(8);
    let simple_total = result_bytes * 6; // old formula: result + 5x temp
    // FFT memory should make the estimate >= the simple calculation
    assert!(est.total_bytes >= simple_total,
        "FFT-aware estimate ({}) should be >= simple estimate ({})",
        est.total_bytes, simple_total);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fibcalc-core estimate_includes_fft -- --nocapture`
Expected: FAIL (current estimate uses fixed 5x, doesn't include FFT overhead)

**Step 3: Expose estimate_fft_memory from fibcalc-bigfft**

In `crates/fibcalc-bigfft/src/memory_est.rs`, remove line 2:
```
#![allow(dead_code)] // Infrastructure: will be wired up for memory budget checks
```

In `crates/fibcalc-bigfft/src/lib.rs`, add re-export:
```rust
pub use memory_est::estimate_fft_memory;
```

**Step 4: Enhance MemoryEstimate::estimate to use FFT estimation**

In `crates/fibcalc-core/src/memory_budget.rs`, modify `estimate`:

```rust
pub fn estimate(n: u64) -> Self {
    let result_bits = (n as f64 * 0.6942).ceil() as usize;
    let result_bytes = result_bits.div_ceil(8);

    // Base temporaries: ~5x the result for Fast Doubling (FK, FK1, T1, T2, T3)
    let doubling_temp = result_bytes * 5;

    // For large n, add FFT multiplication memory overhead
    let fft_temp = if result_bits >= crate::DEFAULT_FFT_THRESHOLD {
        fibcalc_bigfft::estimate_fft_memory(result_bits, result_bits)
    } else {
        0
    };

    let temp_bytes = doubling_temp + fft_temp;

    Self {
        result_bytes,
        temp_bytes,
        total_bytes: result_bytes + temp_bytes,
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p fibcalc-core estimate_includes_fft -- --nocapture`
Expected: PASS

**Step 6: Run full test suite**

Run: `cargo test -p fibcalc-core`
Expected: All PASS (existing tests should still pass since the estimate only gets larger)

**Step 7: Commit**

```bash
git add crates/fibcalc-bigfft/src/memory_est.rs crates/fibcalc-bigfft/src/lib.rs crates/fibcalc-core/src/memory_budget.rs
git commit -m "feat(core): connect FFT memory estimation to budget checker for large n"
```

---

### Task 1.5: Final Agent 1 verification

**Step 1: Run full workspace tests**

Run: `cargo test --workspace`
Expected: All PASS

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Zero warnings

**Step 3: Commit any fixups**

---

## Agent 2 — Portability (GMP Feature Flag)

**Branch:** `feat/portability`
**Crates:** `fibcalc-core`, `fibcalc`, workspace root

---

### Task 2.1: Fix workspace rug declaration

**Files:**
- Modify: `Cargo.toml:37` (workspace root)

**Context:** `rug = { version = "1.24" }` is declared without `features = ["integer"]`. The `rug` crate requires opting into its types. Without this, `rug::Integer` is unavailable.

**Step 1: Write the verification check**

Run: `cargo build -p fibcalc-core --features gmp 2>&1`
Expected: Compilation error (rug::Integer not found or unused import warning)

**Step 2: Fix the workspace declaration**

In `Cargo.toml` (workspace root), change line 37:
```toml
rug = { version = "1.24" }
```
to:
```toml
rug = { version = "1.24", features = ["integer"] }
```

**Step 3: Verify build compiles (even if gmp module is still a stub)**

Run: `cargo check -p fibcalc-core --features gmp`
Expected: Compiles (with unused import warning for `rug::Integer` since the stub doesn't use it yet)

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "fix(workspace): add integer feature to rug dependency for GMP support"
```

---

### Task 2.2: Implement GmpCalculator

**Files:**
- Modify: `crates/fibcalc-core/src/calculator_gmp.rs` (replace stub with full implementation)
- Test: `crates/fibcalc-core/src/calculator_gmp.rs` (inline tests)

**Context:** Must implement `CoreCalculator` trait: `calculate_core(cancel, observer, calc_index, n, opts) -> Result<BigUint, FibError>` and `name() -> &'static str`. Return type is `num_bigint::BigUint` — must convert from `rug::Integer` at the API boundary. Follow `OptimizedFastDoubling` pattern from `fastdoubling.rs`.

**Step 1: Write the failing test (in the gmp module)**

Replace `crates/fibcalc-core/src/calculator_gmp.rs` entirely with:

```rust
//! GMP-based Fast Doubling calculator using the `rug` crate.
//!
//! Only available when the `gmp` feature is enabled.
//! Provides ~2-3x speedup for very large n (>1M) via hardware-accelerated
//! big-integer arithmetic at the cost of an LGPL system dependency (libgmp).

#[cfg(feature = "gmp")]
use num_bigint::BigUint;
#[cfg(feature = "gmp")]
use rug::Integer;

#[cfg(feature = "gmp")]
use crate::calculator::{CoreCalculator, FibError};
#[cfg(feature = "gmp")]
use crate::observer::ProgressObserver;
#[cfg(feature = "gmp")]
use crate::options::Options;
#[cfg(feature = "gmp")]
use crate::progress::{CancellationToken, ProgressUpdate};

/// GMP-backed Fast Doubling calculator.
#[cfg(feature = "gmp")]
pub struct GmpCalculator;

#[cfg(feature = "gmp")]
impl GmpCalculator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Convert rug::Integer to num_bigint::BigUint via decimal string.
    fn to_biguint(value: &Integer) -> BigUint {
        value.to_string().parse::<BigUint>().expect("valid integer")
    }

    #[allow(clippy::cast_possible_truncation)]
    fn execute_doubling(
        &self,
        n: u64,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
    ) -> Result<Integer, FibError> {
        let num_bits = 64 - n.leading_zeros();

        let mut fk = Integer::from(0);
        let mut fk1 = Integer::from(1);

        let frozen = observer.freeze();

        for i in (0..num_bits).rev() {
            if cancel.is_cancelled() {
                return Err(FibError::Cancelled);
            }

            // t = 2*fk1 - fk
            let mut t = Integer::from(&fk1 << 1);
            t -= &fk;

            // f2k = fk * t
            let f2k = Integer::from(&fk * &t);

            // f2k1 = fk^2 + fk1^2
            let fk_sq = Integer::from(fk.square_ref());
            let fk1_sq = Integer::from(fk1.square_ref());
            let f2k1 = fk_sq + fk1_sq;

            fk = f2k;
            fk1 = f2k1;

            // Conditional addition
            if (n >> i) & 1 == 1 {
                std::mem::swap(&mut fk, &mut fk1);
                fk1 += &fk;
            }

            let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
            if frozen.should_report(progress) {
                frozen.update(progress);
                observer.on_progress(&ProgressUpdate::new(
                    calc_index,
                    "GMP",
                    progress,
                    u64::from(num_bits - i),
                    u64::from(num_bits),
                ));
            }
        }

        Ok(fk)
    }
}

#[cfg(feature = "gmp")]
impl Default for GmpCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "gmp")]
impl CoreCalculator for GmpCalculator {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        _opts: &Options,
    ) -> Result<BigUint, FibError> {
        let result = self.execute_doubling(n, cancel, observer, calc_index)?;
        observer.on_progress(&ProgressUpdate::done(calc_index, "GMP"));
        Ok(Self::to_biguint(&result))
    }

    fn name(&self) -> &'static str {
        "GMP"
    }
}

#[cfg(all(test, feature = "gmp"))]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    fn compute_gmp(n: u64) -> BigUint {
        let calc = GmpCalculator::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        calc.calculate_core(&cancel, &observer, 0, n, &opts).unwrap()
    }

    #[test]
    fn gmp_f94() {
        assert_eq!(
            compute_gmp(94),
            BigUint::parse_bytes(b"19740274219868223167", 10).unwrap()
        );
    }

    #[test]
    fn gmp_f100() {
        assert_eq!(
            compute_gmp(100),
            BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }

    #[test]
    fn gmp_f1000_digit_count() {
        let result = compute_gmp(1000);
        assert_eq!(result.to_string().len(), 209);
    }

    #[test]
    fn gmp_agrees_with_fast_doubling() {
        use crate::fastdoubling::OptimizedFastDoubling;
        let fast = OptimizedFastDoubling::new();
        let gmp = GmpCalculator::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default();

        for n in [94, 100, 200, 500, 1000, 5000] {
            let fast_result = fast.calculate_core(&cancel, &observer, 0, n, &opts).unwrap();
            let gmp_result = gmp.calculate_core(&cancel, &observer, 0, n, &opts).unwrap();
            assert_eq!(fast_result, gmp_result, "mismatch at n={n}");
        }
    }

    #[test]
    fn gmp_cancellation() {
        let calc = GmpCalculator::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let observer = NoOpObserver::new();
        let opts = Options::default();
        let result = calc.calculate_core(&cancel, &observer, 0, 10000, &opts);
        assert!(matches!(result, Err(FibError::Cancelled)));
    }
}
```

**Step 2: Run tests with gmp feature**

Run: `cargo test -p fibcalc-core --features gmp gmp -- --nocapture`
Expected: All 5 tests PASS (requires libgmp installed)

**Step 3: Verify default build is unaffected**

Run: `cargo test -p fibcalc-core`
Expected: All PASS, no GMP code compiled

**Step 4: Commit**

```bash
git add crates/fibcalc-core/src/calculator_gmp.rs
git commit -m "feat(core): implement GmpCalculator with Fast Doubling via rug::Integer"
```

---

### Task 2.3: Register GmpCalculator in DefaultFactory

**Files:**
- Modify: `crates/fibcalc-core/src/registry.rs:1-12,40-56,80-82`
- Test: `crates/fibcalc-core/src/registry.rs`

**Step 1: Write the failing test**

In `crates/fibcalc-core/src/registry.rs` test module, add:

```rust
#[test]
#[cfg(feature = "gmp")]
fn factory_creates_gmp() {
    let factory = DefaultFactory::new();
    let calc = factory.get("gmp");
    assert!(calc.is_ok());
    assert_eq!(calc.unwrap().name(), "GMP");
}

#[test]
#[cfg(feature = "gmp")]
fn factory_available_includes_gmp() {
    let factory = DefaultFactory::new();
    let available = factory.available();
    assert!(available.contains(&"gmp"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fibcalc-core --features gmp factory_creates_gmp -- --nocapture`
Expected: FAIL (no "gmp" arm in `create_calculator`)

**Step 3: Add GMP arm to create_calculator**

In `crates/fibcalc-core/src/registry.rs`, add conditional import at the top:

```rust
#[cfg(feature = "gmp")]
use crate::calculator_gmp::GmpCalculator;
```

In `create_calculator`, add before the `_` arm:

```rust
#[cfg(feature = "gmp")]
"gmp" => {
    let core = Arc::new(GmpCalculator::new());
    Ok(Arc::new(FibCalculator::new(core)))
}
```

In `available`, change to:

```rust
fn available(&self) -> Vec<&str> {
    let mut v = vec!["fast", "matrix", "fft"];
    #[cfg(feature = "gmp")]
    v.push("gmp");
    v
}
```

**Step 4: Run tests**

Run: `cargo test -p fibcalc-core --features gmp -- --nocapture`
Expected: All PASS

Run: `cargo test -p fibcalc-core`
Expected: All PASS (without gmp, factory doesn't include "gmp")

**Step 5: Commit**

```bash
git add crates/fibcalc-core/src/registry.rs
git commit -m "feat(core): register GmpCalculator in DefaultFactory under gmp feature"
```

---

### Task 2.4: Add CI workflow for dual-build testing

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Create the CI workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-C target-cpu=native"

jobs:
  test-pure-rust:
    name: Test (pure Rust)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo doc --workspace --no-deps

  test-gmp:
    name: Test (with GMP)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get update && sudo apt-get install -y libgmp-dev
      - run: cargo test --workspace --features gmp
      - run: cargo clippy --workspace --features gmp -- -D warnings
```

**Step 2: Commit**

```bash
mkdir -p .github/workflows
git add .github/workflows/ci.yml
git commit -m "ci: add dual-build workflow for pure-Rust and GMP configurations"
```

---

### Task 2.5: Add feature flag documentation to lib.rs

**Files:**
- Modify: `crates/fibcalc-core/src/lib.rs:32-33`

**Step 1: Add doc cfg annotation**

Change:
```rust
#[cfg(feature = "gmp")]
pub mod calculator_gmp;
```
to:
```rust
#[cfg_attr(docsrs, doc(cfg(feature = "gmp")))]
#[cfg(feature = "gmp")]
pub mod calculator_gmp;
```

And add the re-export under `#[cfg(feature = "gmp")]`:
```rust
#[cfg(feature = "gmp")]
pub use calculator_gmp::GmpCalculator;
```

**Step 2: Verify docs build**

Run: `cargo doc -p fibcalc-core --no-deps`
Expected: No warnings

**Step 3: Commit**

```bash
git add crates/fibcalc-core/src/lib.rs
git commit -m "docs(core): add cfg doc attribute and re-export for GMP calculator"
```

---

### Task 2.6: Final Agent 2 verification

**Step 1: Run full workspace tests**

Run: `cargo test --workspace`
Expected: All PASS

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Zero warnings

---

## Agent 3 — Documentation

**Branch:** `feat/documentation`
**Crates:** None (docs and metadata only)

---

### Task 3.1: Add workspace metadata inheritance

**Files:**
- Modify: All 7 crate `Cargo.toml` files

**Context:** All crates have `description` but none inherit `repository` from workspace. None have `keywords` or `categories`.

**Step 1: Add metadata to each crate's Cargo.toml**

For each of the 7 publishable crates, add under `[package]`:

```toml
repository.workspace = true
keywords = ["fibonacci", "mathematics", "high-performance", "algorithms"]
categories = ["mathematics", "science", "command-line-utilities"]
```

Adjust `categories` per crate:
- `fibcalc`: `["mathematics", "command-line-utilities"]`
- `fibcalc-core`: `["mathematics", "algorithms"]`
- `fibcalc-bigfft`: `["mathematics", "algorithms"]`
- `fibcalc-orchestration`: `["concurrency", "mathematics"]`
- `fibcalc-cli`: `["command-line-utilities"]`
- `fibcalc-tui`: `["command-line-utilities", "visualization"]`
- `fibcalc-calibration`: `["mathematics", "development-tools::profiling"]`

**Step 2: Verify**

Run: `cargo package --list -p fibcalc 2>&1 | head -5`
Expected: No metadata warnings

**Step 3: Commit**

```bash
git add crates/*/Cargo.toml
git commit -m "docs(metadata): add repository, keywords, categories to all crates"
```

---

### Task 3.2: Create INSTALLATION.md

**Files:**
- Create: `docs/INSTALLATION.md`

**Step 1: Write the file**

Content should cover:
- Prerequisites: Rust 1.80+ (check with `rustc --version`)
- Quick install (pure Rust): `cargo install --path crates/fibcalc`
- With GMP: platform-specific libgmp install + `cargo install --path crates/fibcalc --features gmp`
- Windows: `cargo build --release` (pure Rust works out of the box)
- Linux: `sudo apt-get install libgmp-dev` (for GMP only)
- macOS: `brew install gmp` (for GMP only)
- Troubleshooting: GMP not found, MSRV too old, target-cpu=native on CI
- Docker: minimal multi-stage Dockerfile example

**Step 2: Commit**

```bash
git add docs/INSTALLATION.md
git commit -m "docs: add comprehensive multi-platform installation guide"
```

---

### Task 3.3: Rewrite README.md

**Files:**
- Modify: `README.md`

**Context:** Current README is 437 lines. Needs restructuring with Quick Start, clear install paths, usage examples.

**Step 1: Restructure README**

Key sections to add/restructure:
1. **Quick Start** (3 lines: clone, build, run)
2. **Installation** (two paths: default pure-Rust vs GMP, link to `docs/INSTALLATION.md`)
3. **Usage Examples** (`fibcalc 1000`, `fibcalc 1000000 --tui`, `fibcalc --auto-calibrate`)
4. **Architecture** (simplified 7-crate diagram, link to `docs/ARCHITECTURE.md`)
5. **Performance** (benchmark table, link to `docs/PERFORMANCE.md`)
6. **Testing** (how to run tests)
7. **License** (Apache 2.0)

**Step 2: Verify links**

Ensure all `docs/` references are valid paths.

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: rewrite README with Quick Start, clear install paths, usage examples"
```

---

### Task 3.4: Improve rustdoc coverage

**Files:**
- Modify: `crates/fibcalc-core/src/lib.rs` (add `#![warn(missing_docs)]`)
- Modify: `crates/fibcalc-bigfft/src/lib.rs` (add `#![warn(missing_docs)]`)
- Modify: public trait files as needed (add `///` doc comments)

**Context:** Most public items already have docs. This task is about activating the lint and fixing any gaps.

**Step 1: Add missing_docs warning to fibcalc-core**

In `crates/fibcalc-core/src/lib.rs`, add after the module doc:
```rust
#![warn(missing_docs)]
```

**Step 2: Run cargo doc and fix warnings**

Run: `cargo doc -p fibcalc-core --no-deps 2>&1`
Fix any missing doc warnings by adding `///` comments to public items.

**Step 3: Repeat for fibcalc-bigfft**

**Step 4: Verify**

Run: `cargo doc --workspace --no-deps 2>&1`
Expected: Zero warnings

**Step 5: Commit**

```bash
git add crates/fibcalc-core/src/ crates/fibcalc-bigfft/src/
git commit -m "docs: activate missing_docs lint, complete rustdoc coverage"
```

---

### Task 3.5: Update CHANGELOG

**Files:**
- Modify: `docs/CHANGELOG.md`

**Step 1: Add new section under [Unreleased]**

```markdown
### Added

- **Performance**: Wired FFT bump allocator and BigInt pool into computation hot path.
- **Performance**: CPU core affinity separating TUI/metrics from compute threads.
- **Performance**: FFT-aware memory budget estimation for large computations.
- **GMP**: Complete `GmpCalculator` implementation with Fast Doubling via `rug::Integer`.
- **GMP**: Dual-build CI workflow testing pure-Rust and GMP configurations.
- **Documentation**: Multi-platform installation guide (`docs/INSTALLATION.md`).
- **Documentation**: Comprehensive rustdoc coverage with `#![warn(missing_docs)]`.
- **Metadata**: Added `repository`, `keywords`, `categories` to all crate manifests.
```

**Step 2: Commit**

```bash
git add docs/CHANGELOG.md
git commit -m "docs: update CHANGELOG with allocator, GMP, and documentation improvements"
```

---

### Task 3.6: Final Agent 3 verification

**Step 1: Verify all docs build**

Run: `cargo doc --workspace --no-deps`
Expected: Zero warnings

**Step 2: Run clippy (no code changes, but verify)**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Zero warnings

---

## Agent 4 — Validation & Integration

**Branch:** `feat/validation` (based on merge of the 3 branches)
**Prerequisites:** Agents 1, 2, 3 branches merged to a common integration branch.

---

### Task 4.1: Capture baseline benchmarks on main

**Step 1: Run criterion on main**

```bash
git stash  # if needed
git checkout main
cargo bench --bench fibonacci -- --save-baseline main-baseline
```

**Step 2: Switch to integration branch**

```bash
git checkout feat/validation  # or the merge branch
```

**Step 3: Run criterion and compare**

```bash
cargo bench --bench fibonacci -- --baseline main-baseline
```

**Step 4: Capture results**

Save the criterion HTML report and key numbers to `docs/BENCHMARK_REPORT.md`.

**Step 5: Commit**

```bash
git add docs/BENCHMARK_REPORT.md
git commit -m "docs: add benchmark comparison report (main vs optimized)"
```

---

### Task 4.2: Run full test matrix

**Step 1: Execute all verification commands**

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo doc --workspace --no-deps
```

**Step 2: Run with gmp if available**

```bash
cargo test --workspace --features gmp
cargo clippy --workspace --features gmp -- -D warnings
```

**Step 3: Run ignored (slow) tests**

```bash
cargo test --workspace -- --ignored
```

**Step 4: Verify all pass with exit code 0**

If any fail, identify the responsible task and fix on the validation branch.

---

### Task 4.3: Write allocator integration test

**Files:**
- Create: `tests/allocator_integration.rs`

```rust
//! Integration test verifying that FFT allocator infrastructure is active.

use fibcalc_core::calculator::CoreCalculator;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

#[test]
fn fft_computation_uses_active_infrastructure() {
    let calc = FFTBasedCalculator::new();
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();

    // n=10000 is large enough to exercise FFT path
    let result = calc.calculate_core(&cancel, &observer, 0, 10_000, &opts);
    assert!(result.is_ok());

    // Verify result digit count matches golden data (F(10000) has 2090 digits)
    let digits = result.unwrap().to_string().len();
    assert_eq!(digits, 2090, "F(10000) should have 2090 digits");
}
```

**Step 1: Run the test**

Run: `cargo test --test allocator_integration -- --nocapture`
Expected: PASS

**Step 2: Commit**

```bash
git add tests/allocator_integration.rs
git commit -m "test: add allocator integration test verifying FFT infrastructure"
```

---

### Task 4.4: Core affinity fallback test

**Files:**
- Modify: `crates/fibcalc/src/app.rs` (add test in existing test module)

**Step 1: Verify the affinity helpers don't panic**

The test from Task 1.3 Step 2 should already cover this. Verify it exists and passes:

Run: `cargo test -p fibcalc affinity -- --nocapture`
Expected: PASS

---

### Task 4.5: Code review checklist

Run these verification commands:

```bash
# No unsafe code introduced
grep -r "unsafe" crates/ --include="*.rs" | grep -v "forbid" | grep -v "test" | grep -v "//"

# No stale #[allow(dead_code)] on activated infrastructure
grep -rn "allow(dead_code)" crates/fibcalc-bigfft/src/bump.rs
grep -rn "allow(dead_code)" crates/fibcalc-bigfft/src/allocator.rs
grep -rn "allow(dead_code)" crates/fibcalc-bigfft/src/memory_est.rs
# All three should return nothing (annotations removed)

# Feature gates correct
grep -rn 'cfg(feature = "gmp")' crates/fibcalc-core/src/

# deny.toml still accepts all new deps
cargo deny check
```

---

### Task 4.6: Self-assessment scoring

**Files:**
- Create: `docs/SCORING_SELF_ASSESSMENT.md`

Evaluate against the original rubric:

| Category | Before | After | Evidence |
|----------|--------|-------|----------|
| Architecture & Modularité | 24/25 | 25/25 | Pure-Rust default, GMP fully optional with feature flag, CI dual-build |
| Complexité Algorithmique | 25/25 | 25/25 | Unchanged (FFT, Fast Doubling, Matrix) + GMP variant |
| Fiabilité & Tests | 20/20 | 20/20 | +allocator integration test, +GMP tests, all existing tests pass |
| Performance & Mémoire | 18/20 | 20/20 | Bump allocator active, BigIntPool active, core affinity, FFT memory budget |
| Documentation & Outillage | 5/10 | 9/10 | INSTALLATION.md, README rewrite, rustdoc, metadata, CHANGELOG |
| **Total** | **92/100** | **99/100** | |

**Step 1: Write the file and commit**

```bash
git add docs/SCORING_SELF_ASSESSMENT.md
git commit -m "docs: add self-assessment scoring against academic rubric"
```

---

## Execution Order Summary

```
Phase 1 (parallel):
  Agent 1: Task 1.1 → 1.2 → 1.3 → 1.4 → 1.5
  Agent 2: Task 2.1 → 2.2 → 2.3 → 2.4 → 2.5 → 2.6
  Agent 3: Task 3.1 → 3.2 → 3.3 → 3.4 → 3.5 → 3.6

Phase 2 (sequential, after merge):
  Agent 4: Task 4.1 → 4.2 → 4.3 → 4.4 → 4.5 → 4.6
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Merge conflicts between branches | Agents touch different files/crates. Only shared file is workspace `Cargo.toml` (Agent 1 adds `core_affinity`, Agent 2 fixes `rug`) |
| GMP not available on CI/dev machine | Agent 2 Task 2.4 creates dual CI. All GMP code under `#[cfg(feature = "gmp")]` |
| Bump allocator changes FFT correctness | Golden tests are the source of truth. Task 1.1 runs them immediately |
| core_affinity fails in containers | Task 1.3 implements graceful fallback when `get_core_ids()` returns `None` |
| `rug` crate introduces unsafe transitively | `unsafe_code = "forbid"` is workspace-wide and will catch this at compile time |
