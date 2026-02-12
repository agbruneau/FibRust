# FibRust Comprehensive Code Review — DebugPlan.md

> Reviewed: 82 Rust source files, 9 Cargo.toml files, 15,174 LOC across 7 workspace crates.
> Review Date: 2026-02-12

---

## 1. Executive Summary

FibRust is a **production-quality** Fibonacci calculator with mathematically correct algorithms (Fast Doubling, Matrix Exponentiation, FFT-based), a well-designed 4-layer architecture, and strong test coverage (96.1%). The use of `BigUint` for all Fibonacci computation eliminates the primary overflow vulnerability class. However, the codebase has **three systemic issues**: (1) zero use of `pub(crate)` — every internal module, struct, and field is publicly exposed, creating an unnecessarily large API surface; (2) the `CalculationResult` struct uses a Go-style dual-`Option` pattern instead of Rust's `Result<T, E>`, causing `unwrap()` calls in library code; and (3) the release profile is missing `overflow-checks = true`, which could hide arithmetic bugs in supporting code. The documentation and README are exemplary, and the project demonstrates strong Rust patterns in many areas (`#[must_use]`, `std::mem::take/replace`, `thiserror`/`anyhow` separation, thread-local pooling).

---

## 2. Critical Issues (Must-Fix)

### C1. Missing `overflow-checks = true` in Release Profile

**File:** `Cargo.toml:113-118`

In Rust, integer overflow on primitive types panics in debug builds but **silently wraps** in release builds. For a numerical computing application, this is a significant liability.

**Before:**
```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = 3
panic = "abort"
```

**After:**
```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = 3
panic = "abort"
overflow-checks = true  # Catch silent wrapping bugs in release
```

**Impact:** ~2-5% cost on integer-heavy code. BigUint operations (which dominate runtime) are unaffected.

**Rust principle:** Rust's default release behavior wraps arithmetic for performance. Opting into overflow checks is the responsible choice for applications that advertise correctness.

---

### C2. `CalculationResult` Uses Go-Style Dual `Option` Instead of `Result`

**File:** `crates/fibcalc-orchestration/src/interfaces.rs:38-48`

This struct allows 4 states (both Some, both None, etc.) when only 2 are valid. This is the root cause of multiple `unwrap()` calls in library code.

**Before:**
```rust
pub struct CalculationResult {
    pub algorithm: String,
    pub value: Option<BigUint>,   // Go-style
    pub duration: Duration,
    pub error: Option<String>,    // Go-style
}
```

**After:**
```rust
pub struct CalculationResult {
    pub algorithm: String,
    pub outcome: Result<BigUint, String>,
    pub duration: Duration,
}
```

**Affected files:** `orchestrator.rs`, `app.rs`, `presenter.rs` — all dual-checking code becomes simple `match &result.outcome { Ok(v) => ..., Err(e) => ... }`.

**Rust principle:** Make illegal states unrepresentable. `Result<T, E>` is an algebraic sum type — exactly one of success or error exists at any time, enforced at compile time.

---

### C3. `unwrap()` in Library Production Code

**File:** `crates/fibcalc-orchestration/src/orchestrator.rs:117,119`

Library code must never panic on behalf of callers.

**Before:**
```rust
let first_value = valid_results[0].value.as_ref().unwrap();
for result in &valid_results[1..] {
    if result.value.as_ref().unwrap() != first_value {
```

**After:**
```rust
let Some(first_value) = valid_results[0].value.as_ref() else {
    return Err(FibError::Calculation("unexpected empty value".into()));
};
for result in &valid_results[1..] {
    let Some(val) = result.value.as_ref() else {
        return Err(FibError::Calculation("unexpected empty value".into()));
    };
    if val != first_value {
```

**Note:** This issue disappears entirely once C2 is fixed (refactoring to `Result`).

**Rust principle:** Library code uses `Result`/`Option` combinators or `let...else`; only application code may `unwrap`/`expect` as a last resort.

---

### C4. Zero Executable Doc Tests

**Files:** All public types across all 7 crates.

`cargo test --doc` tests nothing. Only 2 doc examples exist, both using `/// ```ignore`. Every public trait (`Calculator`, `CoreCalculator`, `Multiplier`), type (`FibCalculator`, `CancellationToken`, `Options`), and function (`mul`, `sqr`) lacks runnable examples.

**Fix example for `Calculator` trait (`crates/fibcalc-core/src/calculator.rs`):**
```rust
/// Public trait for Fibonacci calculators.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use fibcalc_core::calculator::{Calculator, FibCalculator};
/// use fibcalc_core::fastdoubling::OptimizedFastDoubling;
/// use fibcalc_core::observers::NoOpObserver;
/// use fibcalc_core::options::Options;
/// use fibcalc_core::progress::CancellationToken;
///
/// let calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
/// let cancel = CancellationToken::new();
/// let observer = NoOpObserver::new();
/// let opts = Options::default();
/// let result = calc.calculate(&cancel, &observer, 0, 10, &opts).unwrap();
/// assert_eq!(result.to_string(), "55");
/// ```
pub trait Calculator: Send + Sync { ... }
```

**Rust principle:** Doc tests serve as both documentation AND regression tests. They verify examples compile and produce correct results. The `///` examples are executed by `cargo test`.

---

### C5. No Overflow/Bounds Safety Tests

**File:** `crates/fibcalc-core/src/calculator.rs:87`

`FIB_TABLE[n as usize]` has no guard test. If internal logic ever calls `calculate_small(94)`, it panics with index out-of-bounds.

**Missing test:**
```rust
#[test]
fn calculate_small_boundary_93() {
    let result = FibCalculator::calculate_small(93);
    assert_eq!(result, BigUint::from(12_200_160_415_121_876_738u64));
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn calculate_small_94_panics() {
    let _ = FibCalculator::calculate_small(94);
}
```

---

## 3. Major Improvements (Should-Fix)

### M1. Zero Use of `pub(crate)` — Everything Is Public

**Files:** All `lib.rs` files across all crates.

The entire workspace has zero uses of `pub(crate)`. Every module, struct, trait, field, and function is `pub`, exposing internal implementation details as stable API.

**Key fix for `crates/fibcalc-core/src/lib.rs`:**
```rust
// Public API (consumed by other crates)
pub mod calculator;
pub mod constants;
pub mod fastdoubling;
pub mod fft_based;
pub mod matrix;
pub mod memory_budget;
pub mod observer;
pub mod observers;
pub mod options;
pub mod progress;
pub mod registry;
pub mod strategy;

// Internal only (implementation details)
pub(crate) mod arena;
pub(crate) mod common;
pub(crate) mod doubling_framework;
pub(crate) mod fft_wrappers;
pub(crate) mod generator;
pub(crate) mod generator_iterative;
pub(crate) mod matrix_framework;
pub(crate) mod matrix_ops;
pub(crate) mod matrix_types;
pub(crate) mod threshold_types;
```

**Key fix for `crates/fibcalc-bigfft/src/lib.rs`:** Make all 12 internal modules `pub(crate)`; only re-export the 4 public functions (`mul`, `mul_to`, `sqr`, `sqr_to`).

**Also fix:** `fibcalc-core/src/lib.rs:36` — replace `pub use constants::*;` with explicit re-exports.

**Rust principle:** Minimize public API surface. Internal types should be `pub(crate)` to allow representation changes without breaking downstream code. Wildcard re-exports pollute namespaces.

---

### M2. Missing `unsafe_code = "forbid"` in Workspace Lints

**File:** `Cargo.toml` (workspace lints section)

The PRD requires this, but no Rust-level lints are configured (only clippy lints).

**Add:**
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```

**Rust principle:** For a pure-computation library with zero `unsafe` blocks, `forbid` prevents accidental introduction of unsafe code.

---

### M3. Clippy Cast Lints Globally Suppressed

**File:** `Cargo.toml:91-94`

```toml
cast_possible_truncation = "allow"
cast_sign_loss = "allow"
cast_precision_loss = "allow"
cast_lossless = "allow"
```

50+ `as` casts across the codebase are invisible to the linter. New dangerous casts will never be flagged.

**Fix:** Remove global suppression; add targeted `#[allow(clippy::cast_possible_truncation)]` on audited functions. At minimum, add a `// Rationale:` comment in Cargo.toml.

---

### M4. Silent Error Swallowing on Memory Limit Parse

**File:** `crates/fibcalc/src/app.rs:48,142`

```rust
memory_limit: parse_memory_limit(&config.memory_limit).unwrap_or(0),
```

A typo like `--memory-limit=8X` silently becomes "unlimited memory."

**After:**
```rust
memory_limit: parse_memory_limit(&config.memory_limit)
    .map_err(|e| anyhow::anyhow!("invalid --memory-limit: {e}"))?,
```

**Rust principle:** Don't silently swallow errors at system boundaries. Invalid user input should produce a clear error message.

---

### M5. Missing `Iterator` Trait Implementation

**Files:** `crates/fibcalc-core/src/generator.rs`, `generator_iterative.rs`

`SequenceGenerator` returns an eager `Vec<(u64, BigUint)>`. No `Iterator` trait is implemented for Fibonacci sequences.

**Suggested addition:**
```rust
pub struct FibIterator { a: BigUint, b: BigUint, index: u64 }

impl Iterator for FibIterator {
    type Item = (u64, BigUint);
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.a.clone();
        let idx = self.index;
        let next = &self.a + &self.b;
        self.a = std::mem::replace(&mut self.b, next);
        self.index += 1;
        Some((idx, val))
    }
}
```

**Rust principle:** `Iterator` enables lazy evaluation, composability with `.take()`, `.skip()`, `.collect()`, and zero-cost abstractions. The current eager approach wastes memory for large ranges.

---

### M6. Missing Monotonically Increasing Property Test

**Explicitly required by PRD (line 9103).**

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn monotonically_increasing(n in 1u64..500) {
        let fn_val = compute("fast", n);
        let fn1_val = compute("fast", n + 1);
        prop_assert!(fn1_val > fn_val, "F({}) >= F({}) violated", n + 1, n);
    }
}
```

---

### M7. Binary Crate Has No `lib.rs` — Application Logic Untestable

**File:** `crates/fibcalc/Cargo.toml`

The `fibcalc` binary crate has no `lib.rs`, forcing ~430 lines of duplicated test code (`execute_cli_logic` copies `run_cli` almost exactly at `app.rs:301-352`).

**Fix:** Add a `lib.rs` to the binary crate:
```toml
# crates/fibcalc/Cargo.toml
[lib]
name = "fibcalc_lib"
path = "src/lib.rs"
```
```rust
// crates/fibcalc/src/lib.rs (new)
pub mod app;
pub mod config;
pub mod errors;
pub mod version;
```

**Rust principle:** Binary crates with both `main.rs` and `lib.rs` allow integration tests to import the library target, eliminating test code duplication.

---

### M8. `FIB_TABLE` Bound Undocumented

**File:** `crates/fibcalc-core/src/constants.rs:29`

The table correctly stops at F(93) but there's no doc comment explaining why 94 entries.

**Add:**
```rust
/// Precomputed Fibonacci values for n = 0..=93 (fast path).
/// F(93) = 12,200,160,415,121,876,738 is the largest Fibonacci number
/// that fits in u64. F(94) overflows u64::MAX (18,446,744,073,709,551,615).
pub const FIB_TABLE: [u64; 94] = { ... };
```

---

## 4. Minor Improvements (Nice-to-Have)

### m1. `Vec::remove(0)` Instead of `VecDeque` — O(n) vs O(1)

**Files:** `model.rs:145,161`, `dynamic_threshold.rs:143`

All three are ring buffers that remove from front and push to back. `Vec::remove(0)` shifts all elements.

**Fix:** Change to `VecDeque<T>` and use `.pop_front()`.

---

### m2. `ProgressUpdate::algorithm` Is `String` — Hot-Path Allocation

**File:** `crates/fibcalc-core/src/progress.rs:16`

Algorithm names are always static literals but stored as `String`, allocating on every progress update.

**Fix:** Change to `&'static str` or `Cow<'static, str>`.

---

### m3. Sentinel Values (0, "") Instead of `Option<T>`

**Files:** `options.rs:17-19`, `config.rs:51-59,74-75`

Go-style zero-value defaults. `0` means "use default", `""` means "not provided".

**Fix:** Use `Option<u32>`, `Option<usize>`, `Option<String>`.

---

### m4. Float-to-Int Truncation Bias in Thresholds

**File:** `crates/fibcalc-core/src/dynamic_threshold.rs:74,80,88,99,112,123`

Six `((x as f64) * factor) as usize` patterns truncate toward zero.

**Fix:** Add `.round()` before casting: `((x as f64) * factor).round() as usize`.

---

### m5. Bare `use rayon;` Import

**File:** `crates/fibcalc-core/src/strategy.rs:8`

Unnecessary in Rust 2021 edition. Clippy's `single_component_path_imports` would flag this. Remove the line.

---

### m6. Missing `# Panics` Doc Sections

Zero `# Panics` sections across the entire codebase. Functions like `calculate_small()` that can panic should document it per Rustdoc conventions.

---

### m7. Missing `FibError` Specialized Variants

**File:** `crates/fibcalc-core/src/calculator.rs:17-38`

Uses catch-all `Calculation(String)` instead of typed variants.

**Add:**
```rust
#[error("overflow computing F({0}): result exceeds {1} capacity")]
Overflow(u64, &'static str),
#[error("invalid input: {0}")]
InvalidInput(String),
```

---

### m8. Duplicate Golden Test Locations

`tests/golden.rs` (workspace root) and `crates/fibcalc/tests/golden.rs` are largely redundant. Consolidate to one location.

---

### m9. Code Duplication: `Options` Construction

**File:** `crates/fibcalc/src/app.rs:42-52` and `137-147`

Identical 15-line blocks in `run_cli` and `run_tui`. Extract to `fn build_options(config: &AppConfig) -> Result<Options>`.

---

### m10. Missing Convenience API

No simple `fibonacci(n)` function for library consumers. Currently requires 5 imports and a complex call chain.

**Add to `crates/fibcalc-core/src/lib.rs`:**
```rust
/// Compute F(n) using the fast doubling algorithm.
#[must_use]
pub fn fibonacci(n: u64) -> BigUint {
    if n <= MAX_FIB_U64 {
        return BigUint::from(FIB_TABLE[n as usize]);
    }
    let factory = DefaultFactory::new();
    let calc = factory.get("fast").expect("fast doubling always available");
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    calc.calculate(&cancel, &observer, 0, n, &opts)
        .expect("fast doubling should not fail for valid input")
}
```

---

### m11. Fuzz Testing Only Covers FastDoubling

**File:** `fuzz/fuzz_targets/`

Matrix and FFT algorithms are not fuzzed. Add fuzz targets for cross-algorithm comparison.

---

### m12. Benchmark Missing `black_box` and Large N

**File:** `crates/fibcalc-core/benches/fibonacci.rs`

- Add explicit `black_box()` wrapping for best-practice conformance.
- Add N=1,000,000 to match README performance claims.

---

### m13. Wrong Complexity Claim in CLAUDE.md

CLAUDE.md claims Fast Doubling has "O(log n) stack" space, but the implementation is **iterative** (not recursive), using O(1) stack space. Update the documentation.

---

## 5. Positive Observations

1. **Algorithms are mathematically correct.** All 6 algorithm variants (Fast Doubling, Matrix, FFT-based, Iterative, Modular, Precomputed Table) produce correct results with no off-by-one errors, verified base cases, and consistent cross-algorithm agreement.

2. **`BigUint` architecture eliminates overflow by design.** The most common class of Fibonacci bugs — integer overflow — is impossible since all computation uses arbitrary-precision arithmetic.

3. **Excellent `#[must_use]` coverage.** Over 100 annotations across all crates, on constructors, pure functions, and getters.

4. **Masterful use of `std::mem::take/replace/swap`.** Zero-copy BigUint value rotation throughout — especially in `fastdoubling.rs`, `generator_iterative.rs`, and the pool system. This is expert-level Rust.

5. **Proper `thiserror`/`anyhow` separation.** `thiserror` for typed library errors (`FibError`), `anyhow` confined to the binary — textbook Rust error handling architecture.

6. **Zero `unsafe` blocks, zero `todo!/unimplemented!` in production.** A clean, safe codebase.

7. **Strong test infrastructure.** Golden file tests, property-based tests with `proptest`, E2E CLI tests with `assert_cmd`, fuzz testing, Criterion benchmarks — comprehensive for a calculator project.

8. **Professional workspace configuration.** Centralized dependencies, lints, profiles; all crates inherit workspace settings. `deny.toml` for license auditing.

9. **Exemplary README and documentation.** 307-line README with architecture diagrams, algorithm descriptions, complexity analysis, and 10 linked documentation files.

10. **Symmetric matrix optimization.** Reduces matrix squaring from 8 to 3 multiplications by exploiting the Q-matrix symmetry (b==c). Well-implemented.

11. **Thread-local pooling with zero-copy extraction.** `CalculationState` pooled via `thread_local!` with `std::mem::take` for result extraction — eliminates allocation in hot paths.

12. **Elm architecture in TUI.** The `model.rs` implementation correctly separates model, view, and update — a clean architecture for interactive UIs.

---

## 6. Recommended Action Plan (Prioritized by Impact)

### Phase 1: Safety & Correctness (Week 1)

| Priority | Item | Files | Effort |
|----------|------|-------|--------|
| P0 | Add `overflow-checks = true` to release profile | `Cargo.toml` | 1 line |
| P0 | Add `unsafe_code = "forbid"` to workspace lints | `Cargo.toml` | 2 lines |
| P0 | Refactor `CalculationResult` to use `Result` | `interfaces.rs`, `orchestrator.rs`, `app.rs`, `presenter.rs` | 2-3 hours |
| P0 | Remove `unwrap()` from `orchestrator.rs:117,119` | `orchestrator.rs` | 15 min |
| P1 | Add `#[should_panic]` test for `FIB_TABLE` bounds | `calculator.rs` tests | 15 min |
| P1 | Fix silent `unwrap_or(0)` on memory limit parse | `app.rs:48,142` | 15 min |

### Phase 2: API Surface & Visibility (Week 2)

| Priority | Item | Files | Effort |
|----------|------|-------|--------|
| P1 | Add `pub(crate)` to internal modules in `fibcalc-core` | `lib.rs` + affected imports | 1-2 hours |
| P1 | Add `pub(crate)` to internal modules in `fibcalc-bigfft` | `lib.rs` + affected imports | 1 hour |
| P1 | Replace `pub use constants::*` with explicit re-exports | `fibcalc-core/src/lib.rs` | 15 min |
| P1 | Make internal struct fields private | `fastdoubling.rs`, `matrix_types.rs` | 1 hour |
| P2 | Add `lib.rs` to binary crate | `fibcalc/Cargo.toml`, new `lib.rs` | 30 min |
| P2 | Remove duplicated `execute_cli_logic` test helper | `app.rs` | 30 min |

### Phase 3: Testing & Documentation (Week 3)

| Priority | Item | Files | Effort |
|----------|------|-------|--------|
| P1 | Add doc tests to `Calculator`, `CancellationToken`, `Options` | Core public types | 2-3 hours |
| P1 | Add monotonically increasing property test | `proptest.rs` or `properties.rs` | 15 min |
| P2 | Document `FIB_TABLE` bounds rationale | `constants.rs` | 5 min |
| P2 | Add `# Panics` doc sections | `calculator.rs`, others | 30 min |
| P2 | Add fuzz targets for Matrix and FFT | `fuzz/fuzz_targets/` | 1 hour |
| P3 | Fix CLAUDE.md Fast Doubling complexity claim | `CLAUDE.md` | 5 min |

### Phase 4: Idioms & Performance (Week 4)

| Priority | Item | Files | Effort |
|----------|------|-------|--------|
| P2 | Replace `Vec::remove(0)` with `VecDeque` | `model.rs`, `dynamic_threshold.rs` | 30 min |
| P2 | Change `ProgressUpdate::algorithm` to `&'static str` | `progress.rs` + callers | 1 hour |
| P2 | Add `Iterator` trait implementation | `generator.rs` | 1 hour |
| P2 | Replace sentinel values with `Option<T>` | `options.rs`, `config.rs` | 1-2 hours |
| P2 | Use targeted `#[allow]` instead of global cast lint suppression | `Cargo.toml` + individual files | 2 hours |
| P3 | Add float `.round()` to threshold casts | `dynamic_threshold.rs` | 15 min |
| P3 | Remove bare `use rayon;` import | `strategy.rs` | 1 min |
| P3 | Add convenience `fibonacci(n)` API | `fibcalc-core/src/lib.rs` | 30 min |
| P3 | Extract `build_options()` helper | `app.rs` | 30 min |

---

## 7. Learning Resources

1. **"Rust API Guidelines"** (https://rust-lang.github.io/api-guidelines/) — The authoritative reference for public API design in Rust. Directly relevant to the `pub(crate)` visibility issues (C-NEWTYPE, C-HIDDEN), `#[must_use]` (C-MUST-USE), and `Result` vs error tuple patterns (C-MEANINGFUL-RESULT). Essential reading for transitioning from Go's package system to Rust's module visibility model.

2. **"Error Handling in Rust" by Andrew Gallant (BurntSushi)** (https://blog.burntsushi.net/rust-error-handling/) — Deep dive into Rust's `Result`/`Option`/`?` error handling philosophy, with practical guidance on when to use `thiserror` vs `anyhow`, how to design error enums, and why Go's `(value, error)` pattern is an anti-pattern in Rust. Directly addresses findings C2, C3, M4.

3. **"Idiomatic Rust" by Matthias Endler** (https://corrode.dev/blog/idiomatic-rust/) — Covers iterator patterns, `std::mem::replace`, `From`/`Into` traits, and the transition from imperative (Go/C) to functional-declarative Rust style. Addresses the explicit-loop-to-iterator conversion (anti-pattern #3) and the missing `From`/`Into` implementations.
