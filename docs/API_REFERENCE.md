# API Reference

> For full API docs with hyperlinked navigation, run `cargo doc --open`.

This document covers the public API of every crate in the FibCalc-rs workspace.

---

## Table of Contents

1. [fibcalc (binary)](#1-fibcalc-binary)
2. [fibcalc-core](#2-fibcalc-core)
3. [fibcalc-bigfft](#3-fibcalc-bigfft)
4. [fibcalc-orchestration](#4-fibcalc-orchestration)
5. [fibcalc-cli](#5-fibcalc-cli)
6. [fibcalc-tui](#6-fibcalc-tui)
7. [fibcalc-calibration](#7-fibcalc-calibration)

---

## 1. fibcalc (binary)

The `fibcalc` crate is the binary entry point. It parses CLI arguments, dispatches to CLI mode, TUI mode, or calibration, and exits with appropriate codes.

### CLI Arguments

| Flag | Short | Type | Default | Env Var | Description |
|------|-------|------|---------|---------|-------------|
| `--n` | `-n` | `u64` | `100000000` | `FIBCALC_N` | Fibonacci index to compute |
| `--algo` | | `String` | `all` | | Algorithm: `fast`, `matrix`, `fft`, or `all` |
| `--calculate` | `-c` | flag | | | Calculate and display the result |
| `--verbose` | `-v` | flag | | | Verbose output |
| `--details` | `-d` | flag | | | Show detailed information (bit count, digit count) |
| `--output` | `-o` | `String` | | | Write result to file |
| `--quiet` | `-q` | flag | | | Quiet mode (only output the number) |
| `--calibrate` | | flag | | | Run full calibration |
| `--auto-calibrate` | | flag | | | Run quick adaptive calibration |
| `--timeout` | | `String` | `5m` | | Timeout duration (`30s`, `5m`, `1h`) |
| `--threshold` | | `usize` | `0` | | Parallel multiplication threshold in bits |
| `--fft-threshold` | | `usize` | `0` | | FFT multiplication threshold in bits |
| `--strassen-threshold` | | `usize` | `0` | | Strassen multiplication threshold in bits |
| `--tui` | | flag | | | Launch interactive TUI dashboard |
| `--completion` | | `Shell` | | | Generate shell completion (bash, zsh, fish, etc.) |
| `--last-digits` | | `u32` | `0` | | Compute only the last K digits (0 = full) |
| `--memory-limit` | | `String` | `""` | | Memory limit (`512M`, `8G`) |

When a threshold flag is `0`, the default from calibration or static defaults is used.

### Environment Variables

| Variable | Description |
|----------|-------------|
| `FIBCALC_N` | Fibonacci index (same as `--n`) |
| `RUST_LOG` | Tracing log level filter (e.g., `warn`, `info`, `debug`) |

### Exit Codes

Defined in `fibcalc_core::constants::exit_codes`:

| Code | Constant | Meaning |
|------|----------|---------|
| `0` | `SUCCESS` | Computation completed successfully |
| `1` | `ERROR_GENERIC` | Generic error |
| `2` | `ERROR_TIMEOUT` | Computation timed out |
| `3` | `ERROR_MISMATCH` | Result mismatch between algorithms |
| `4` | `ERROR_CONFIG` | Configuration error |
| `130` | `ERROR_CANCELED` | User cancelled (Ctrl+C) |

### Configuration Precedence

CLI flags > Environment variables (`FIBCALC_*`) > Adaptive calibration > Static defaults.

---

## 2. fibcalc-core

Core library containing algorithms, traits, strategies, and progress tracking.

### Re-exports

```rust
pub use calculator::{Calculator, CoreCalculator, FibCalculator};
pub use constants::*;
pub use observer::{ProgressObserver, ProgressSubject};
pub use options::Options;
pub use progress::ProgressUpdate;
pub use registry::{CalculatorFactory, DefaultFactory};
pub use strategy::{DoublingStepExecutor, Multiplier};
```

---

### `FibError` (enum)

Error type for Fibonacci calculations. Derives `thiserror::Error`.

```rust
pub enum FibError {
    Calculation(String),            // "calculation error: {0}"
    Config(String),                 // "configuration error: {0}"
    Cancelled,                      // "calculation cancelled"
    Timeout(String),                // "calculation timed out after {0}"
    Mismatch,                       // "result mismatch between algorithms"
    Overflow(u64, &'static str),    // "overflow computing F({0}): result exceeds {1} capacity"
    InvalidInput(String),           // "invalid input: {0}"
}
```

---

### `Calculator` (trait)

Public trait consumed by orchestration. Requires `Send + Sync`.

```rust
pub trait Calculator: Send + Sync {
    /// # Errors
    /// Returns `FibError` on cancellation, timeout, or calculation failure.
    fn calculate(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    fn name(&self) -> &str;
}
```

**Parameters:**
- `cancel` -- cooperative cancellation token, checked before and during computation.
- `observer` -- receives `ProgressUpdate` notifications.
- `calc_index` -- identifies this calculator in multi-algorithm runs.
- `n` -- Fibonacci index to compute.
- `opts` -- thresholds and configuration.

---

### `CoreCalculator` (trait)

Internal trait implemented by algorithm structs (Fast Doubling, Matrix, FFT). Not used directly by consumers; wrapped by `FibCalculator`.

```rust
pub trait CoreCalculator: Send + Sync {
    /// # Errors
    /// Returns `FibError` on cancellation, timeout, or calculation failure.
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    fn name(&self) -> &str;
}
```

---

### `FibCalculator` (struct)

Decorator that wraps a `CoreCalculator` with the fast path (n <= 93 uses a precomputed lookup table) and cancellation checking.

```rust
pub struct FibCalculator { /* ... */ }

impl FibCalculator {
    pub fn new(inner: Arc<dyn CoreCalculator>) -> Self;
}

impl Calculator for FibCalculator { /* ... */ }
```

**Example:**
```rust
use fibcalc_core::{FibCalculator, Calculator};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::progress::CancellationToken;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::Options;
use std::sync::Arc;

let calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
let cancel = CancellationToken::new();
let observer = NoOpObserver::new();
let opts = Options::default();
let result = calc.calculate(&cancel, &observer, 0, 100, &opts).unwrap();
// result == F(100)
```

---

### `Multiplier` (trait)

Narrow interface for big number multiplication (Interface Segregation Principle). Requires `Send + Sync`.

```rust
pub trait Multiplier: Send + Sync {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint;
    fn square(&self, a: &BigUint) -> BigUint;  // default: self.multiply(a, a)
    fn name(&self) -> &str;
}
```

---

### `DoublingStepExecutor` (trait)

Extended interface for optimized Fast Doubling steps. Extends `Multiplier`.

```rust
pub trait DoublingStepExecutor: Multiplier {
    /// Given F(k) and F(k+1), compute (F(2k), F(2k+1)).
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint);
}
```

---

### Multiplication Strategies

All implement both `Multiplier` and `DoublingStepExecutor`.

| Struct | Description |
|--------|-------------|
| `KaratsubaStrategy` | Default strategy using `num-bigint` built-in multiplication. |
| `ParallelKaratsubaStrategy` | Parallelizes the three independent multiplications in the doubling step using `rayon::join` when operand bits exceed the parallel threshold. |
| `FFTOnlyStrategy` | Always uses `fibcalc_bigfft::mul`/`sqr` for multiplication. |
| `AdaptiveStrategy` | Selects Karatsuba or FFT based on operand bit length vs. `fft_threshold`. |

**Construction:**
```rust
KaratsubaStrategy::new()           // or ::default()
ParallelKaratsubaStrategy::new(parallel_threshold: usize)
FFTOnlyStrategy::new()             // or ::default()
AdaptiveStrategy::new(fft_threshold: usize, strassen_threshold: usize)
```

---

### `Options` (struct)

Configuration for Fibonacci calculation.

```rust
pub struct Options {
    pub parallel_threshold: usize,  // default: 4096 bits
    pub fft_threshold: usize,       // default: 500_000 bits
    pub strassen_threshold: usize,  // default: 3072 bits
    pub last_digits: u32,           // 0 = full number
    pub memory_limit: usize,        // 0 = unlimited
    pub verbose: bool,
    pub details: bool,
}

impl Options {
    pub fn normalize(self) -> Self;  // replaces 0 thresholds with defaults
}
impl Default for Options { /* ... */ }
```

---

### `ProgressObserver` (trait)

Observer trait for receiving progress updates. Requires `Send + Sync`.

```rust
pub trait ProgressObserver: Send + Sync {
    fn on_progress(&self, update: &ProgressUpdate);
    fn freeze(&self) -> FrozenObserver;
}
```

The `freeze()` method creates a `FrozenObserver` for lock-free access in hot computation loops.

---

### `FrozenObserver` (struct)

Lock-free progress observer for use in tight loops. Uses atomic operations.

```rust
pub struct FrozenObserver { /* ... */ }

impl FrozenObserver {
    pub fn new(threshold: f64) -> Self;
    pub fn should_report(&self, new_progress: f64) -> bool;
    pub fn update(&self, new_progress: f64);
    pub fn current(&self) -> f64;
}
```

---

### `ProgressSubject` (struct)

Subject that manages a collection of observers (Observer pattern).

```rust
pub struct ProgressSubject { /* ... */ }

impl ProgressSubject {
    pub fn new() -> Self;
    pub fn register(&self, observer: Arc<dyn ProgressObserver>);
    pub fn clear(&self);
    pub fn notify(&self, update: &ProgressUpdate);
    pub fn count(&self) -> usize;
}
impl Default for ProgressSubject { /* ... */ }
```

---

### `ProgressUpdate` (struct)

Progress update sent from calculators to observers.

```rust
pub struct ProgressUpdate {
    pub calc_index: usize,      // calculator index in multi-calc runs
    pub algorithm: String,       // algorithm name
    pub progress: f64,           // fraction in [0.0, 1.0]
    pub current_step: u64,
    pub total_steps: u64,
    pub done: bool,
}

impl ProgressUpdate {
    pub fn new(calc_index: usize, algorithm: &str, progress: f64, current: u64, total: u64) -> Self;
    pub fn done(calc_index: usize, algorithm: &str) -> Self;
}
```

---

### `CancellationToken` (struct)

Cooperative cancellation using atomic operations. Cloning shares the same underlying state.

```rust
pub struct CancellationToken { /* ... */ }

impl CancellationToken {
    pub fn new() -> Self;
    pub fn is_cancelled(&self) -> bool;
    pub fn cancel(&self);
    pub fn check_cancelled(&self) -> Result<(), FibError>;
}
impl Default for CancellationToken { /* ... */ }
impl Clone for CancellationToken { /* ... */ }
```

---

### `TimeoutCancellationToken` (struct)

Combines cooperative cancellation with an absolute deadline.

```rust
pub struct TimeoutCancellationToken { /* ... */ }

impl TimeoutCancellationToken {
    pub fn new(timeout: Duration) -> Self;
    pub fn is_cancelled(&self) -> bool;      // manual cancel OR timeout
    pub fn cancel(&self);
    pub fn check_cancelled(&self) -> Result<(), FibError>;  // Cancelled or Timeout error
    pub fn remaining(&self) -> Duration;
    pub fn token(&self) -> &CancellationToken;
}
```

---

### `CalculatorFactory` (trait)

Factory trait for creating calculators.

```rust
pub trait CalculatorFactory: Send + Sync {
    /// # Errors
    /// Returns `FibError` if the calculator name is unknown.
    fn get(&self, name: &str) -> Result<Arc<dyn Calculator>, FibError>;
    fn available(&self) -> Vec<&str>;
}
```

---

### `DefaultFactory` (struct)

Default factory with lazy creation and `RwLock<HashMap>` cache.

```rust
pub struct DefaultFactory { /* ... */ }

impl DefaultFactory {
    pub fn new() -> Self;
}
impl Default for DefaultFactory { /* ... */ }
impl CalculatorFactory for DefaultFactory { /* ... */ }
```

**Available calculator names:** `"fast"` (alias `"fastdoubling"`), `"matrix"`, `"fft"`.

**Example:**
```rust
use fibcalc_core::registry::{CalculatorFactory, DefaultFactory};

let factory = DefaultFactory::new();
let calc = factory.get("fast").unwrap();
assert_eq!(calc.name(), "FastDoubling");

let all_names = factory.available(); // ["fast", "matrix", "fft"]
```

---

### `DynamicThresholdManager` (struct)

Dynamically adjusts multiplication thresholds at runtime using a ring buffer of iteration metrics and hysteresis-based adjustments.

```rust
pub struct DynamicThresholdManager { /* ... */ }

impl DynamicThresholdManager {
    pub fn new(config: DynamicThresholdConfig) -> Self;
    pub fn record(&mut self, metric: IterationMetric);
    pub fn adjust(&mut self);
    pub fn metric_count(&self) -> usize;
    pub fn parallel_threshold(&self) -> usize;
    pub fn fft_threshold(&self) -> usize;
    pub fn strassen_threshold(&self) -> usize;
    pub fn snapshot(&self) -> ThresholdSnapshot;
    pub fn stats(&self) -> Option<ThresholdStats>;
    pub fn reset(&mut self);
    pub fn is_ring_full(&self) -> bool;
    pub fn adjustment_count(&self) -> usize;
    pub fn set_thresholds(&mut self, parallel: usize, fft: usize, strassen: usize);
}
impl Default for DynamicThresholdManager { /* ... */ }
```

---

### `DynamicThresholdConfig` (struct)

Configuration for `DynamicThresholdManager`.

```rust
pub struct DynamicThresholdConfig {
    pub ring_buffer_size: usize,   // default: 32
    pub hysteresis_factor: f64,    // default: 0.05
    pub max_adjustment: f64,       // default: 0.1
    pub dead_zone: f64,            // default: 0.02
}
impl Default for DynamicThresholdConfig { /* ... */ }
```

---

### `IterationMetric` (struct)

Metric collected for each multiplication iteration.

```rust
pub struct IterationMetric {
    pub bit_length: usize,
    pub fft_speedup: f64,
    pub parallel_speedup: f64,
    pub duration_ns: u64,
    pub method: MultiplicationMethod,
    pub bits_processed: u64,
    pub cache_hit: bool,
}

impl IterationMetric {
    pub fn basic(bit_length: usize, fft_speedup: f64, parallel_speedup: f64, duration_ns: u64) -> Self;
}
```

---

### `MultiplicationMethod` (enum)

```rust
pub enum MultiplicationMethod {
    Karatsuba,
    Fft,
    Strassen,
}
```

---

### `ThresholdSnapshot` (struct)

Serializable snapshot of the current threshold state.

```rust
pub struct ThresholdSnapshot {
    pub parallel_threshold: usize,
    pub fft_threshold: usize,
    pub strassen_threshold: usize,
    pub adjustment_count: usize,
    pub adjustment_history: Vec<ThresholdAdjustment>,
}
```

---

### `ThresholdAdjustment` (struct)

```rust
pub struct ThresholdAdjustment {
    pub threshold_name: String,
    pub old_value: usize,
    pub new_value: usize,
    pub trigger_benefit: f64,
}
```

---

### `ThresholdStats` (struct)

```rust
pub struct ThresholdStats {
    pub fft_benefit: f64,
    pub parallel_benefit: f64,
    pub strassen_benefit: f64,
    pub sample_count: usize,
}
```

---

### `MemoryEstimate` (struct)

Estimates memory usage for computing F(n).

```rust
pub struct MemoryEstimate {
    pub result_bytes: usize,
    pub temp_bytes: usize,
    pub total_bytes: usize,
}

impl MemoryEstimate {
    pub fn estimate(n: u64) -> Self;
    pub fn fits_in(&self, limit: usize) -> bool;  // limit=0 means unlimited
}
```

---

### `parse_memory_limit` (function)

Parses a memory limit string.

```rust
/// # Errors
/// Returns an error string if the format is invalid or the number cannot be parsed.
pub fn parse_memory_limit(s: &str) -> Result<usize, String>;
```

Accepts: `"8G"`, `"512M"`, `"1024K"`, `"1024B"`, `""` (returns 0 = unlimited).

---

### `calc_total_work` (function)

Estimates total work for a Fibonacci computation using a geometric model based on powers of 4.

```rust
#[must_use]
pub fn calc_total_work(n: u64) -> f64;
```

---

### `check_cancellation` (function)

Convenience function for use in algorithm loops.

```rust
/// # Errors
/// Returns `FibError::Cancelled` if cancellation was requested.
pub fn check_cancellation(token: &CancellationToken) -> Result<(), FibError>;
```

---

### `CalculationArena` (struct)

Arena allocator for Fibonacci calculation temporaries. Uses `bumpalo` for O(1) allocation with bulk deallocation.

```rust
pub struct CalculationArena { /* ... */ }

impl CalculationArena {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn reset(&mut self);
    pub fn allocated_bytes(&self) -> usize;
    pub fn bump(&self) -> &bumpalo::Bump;
}
impl Default for CalculationArena { /* ... */ }
```

---

### `SequenceGenerator` (trait)

Trait for generating ranges of sequential Fibonacci numbers.

```rust
pub trait SequenceGenerator: Send + Sync {
    fn generate(
        &self,
        start: u64,
        end: u64,
        cancel: &CancellationToken,
    ) -> Result<Vec<(u64, BigUint)>, FibError>;

    fn name(&self) -> &str;
}
```

**Implementations:** `IterativeGenerator` -- computes F(start) through F(end) iteratively.

---

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_PARALLEL_THRESHOLD` | `4096` | Default parallel threshold (bits) |
| `DEFAULT_FFT_THRESHOLD` | `500_000` | Default FFT threshold (bits) |
| `DEFAULT_STRASSEN_THRESHOLD` | `3072` | Default Strassen threshold (bits) |
| `PARALLEL_FFT_THRESHOLD` | `5_000_000` | Threshold for parallel FFT execution (bits) |
| `CALIBRATION_N` | `10_000_000` | Default N for calibration benchmarks |
| `PROGRESS_REPORT_THRESHOLD` | `0.01` | Minimum progress change (1%) before reporting |
| `MAX_FIB_U64` | `93` | Max Fibonacci index fitting in u64 |
| `MAX_POOLED_BIT_LEN` | `100_000_000` | Max bit length for pooled BigInts (100M bits) |
| `FIB_TABLE` | `[u64; 94]` | Precomputed F(0) through F(93) |

---

## 3. fibcalc-bigfft

FFT-based big number multiplication using Schonhage-Strassen NTT over Fermat rings.

### Re-exports

```rust
pub use fft::{mul, mul_to, sqr, sqr_to};
```

### `mul` (function)

Multiply two `BigUint` values. Uses FFT for operands above 10,000 bits, falls back to standard multiplication below.

```rust
pub fn mul(a: &BigUint, b: &BigUint) -> BigUint;
```

### `sqr` (function)

Square a `BigUint`. Uses FFT with transform reuse optimization (one forward NTT instead of two) for operands above 10,000 bits.

```rust
pub fn sqr(a: &BigUint) -> BigUint;
```

### `mul_to` (function)

Multiply and store result in destination.

```rust
pub fn mul_to(dst: &mut BigUint, a: &BigUint, b: &BigUint);
```

### `sqr_to` (function)

Square and store result in destination.

```rust
pub fn sqr_to(dst: &mut BigUint, a: &BigUint);
```

**Example:**
```rust
use num_bigint::BigUint;

let a = BigUint::from(12345u64);
let b = BigUint::from(67890u64);
let product = fibcalc_bigfft::mul(&a, &b);
assert_eq!(product, BigUint::from(838_102_050u64));

let squared = fibcalc_bigfft::sqr(&a);
assert_eq!(squared, BigUint::from(152_399_025u64));
```

---

### `FermatNum` (struct)

Fermat number representation: value = data mod (2^shift + 1). Used internally for NTT-based multiplication over Fermat rings. Arithmetic operates on u64 limbs to avoid heap allocations in hot loops.

```rust
pub struct FermatNum {
    pub data: Vec<u64>,    // little-endian u64 limbs
    pub shift: usize,      // Fermat modulus is 2^shift + 1
}

impl FermatNum {
    pub fn new(shift: usize) -> Self;
    pub fn from_biguint(value: &BigUint, shift: usize) -> Self;
    pub fn to_biguint(&self) -> BigUint;
    pub fn modulus(&self) -> BigUint;
    pub fn normalize(&mut self);
    pub fn add(&self, other: &Self) -> Self;
    pub fn sub(&self, other: &Self) -> Self;
    pub fn fermat_mul(&self, other: &Self) -> Self;
    pub fn shift_left(&mut self, s: usize);
    pub fn shift_right(&mut self, k: usize);
    pub fn is_zero(&self) -> bool;
}
```

---

### `Poly` (struct)

Polynomial representation for FFT multiplication. Coefficients are `FermatNum` values.

```rust
pub struct Poly {
    pub coeffs: Vec<FermatNum>,
    pub fermat_shift: usize,
    pub piece_bits: usize,
}

impl Poly {
    pub fn from_biguint(value: &BigUint, n: usize, piece_bits: usize, fermat_shift: usize) -> Self;
    pub fn to_biguint(&self) -> BigUint;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

---

### `FFTCache` (struct)

Thread-safe LRU cache for FFT root tables. Uses `parking_lot::Mutex`.

```rust
pub struct FFTCache { /* ... */ }

impl FFTCache {
    pub fn new(max_entries: usize) -> Self;
    pub fn get(&self, key: &CacheKey) -> Option<Vec<Vec<u64>>>;
    pub fn put(&self, key: CacheKey, value: Vec<Vec<u64>>);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn clear(&self);
}
impl Default for FFTCache { /* max_entries=64 */ }
```

---

### `BigIntPool` (struct)

Pool for `BigUint` objects organized by size class (powers of 4). Reduces allocation pressure for repeated big number operations.

```rust
pub struct BigIntPool { /* ... */ }

impl BigIntPool {
    pub fn new(max_bit_len: usize, max_per_class: usize) -> Self;
    pub fn acquire(&self, min_bits: usize) -> BigUint;
    pub fn release(&self, value: BigUint);
    pub fn total_pooled(&self) -> usize;
    pub fn stats(&self) -> PoolStats;
    pub fn reset_stats(&self);
    pub fn clear(&self);
    pub fn drain_class(&self, min_bits: usize) -> Vec<BigUint>;
    pub fn drain_all(&self) -> HashMap<usize, Vec<BigUint>>;
    pub fn warm(&self, bits: usize, count: usize);
}
impl Default for BigIntPool { /* max_bit_len=100M, max_per_class=32 */ }
```

---

### `FFTBumpAllocator` (struct)

O(1) bump allocator for FFT temporaries using `bumpalo`.

```rust
pub struct FFTBumpAllocator { /* ... */ }

impl FFTBumpAllocator {
    pub fn new() -> Self;
    pub fn with_capacity(bytes: usize) -> Self;
    pub fn alloc_slice(&self, len: usize) -> &mut [u64];
    pub fn reset(&mut self);
    pub fn allocated_bytes(&self) -> usize;
}
impl Default for FFTBumpAllocator { /* ... */ }
```

---

### `TempAllocator` (trait)

Trait for temporary allocators used in FFT operations.

```rust
pub trait TempAllocator: Send + Sync {
    fn alloc(&self, min_bits: usize) -> BigUint;
    fn free(&self, value: BigUint);
}
```

**Implementations:** `PoolAllocator` (backed by `BigIntPool`), `SimpleAllocator` (allocate/drop each time).

---

### `select_fft_params` (function)

Select optimal FFT parameters for multiplying two numbers.

```rust
pub fn select_fft_params(a_bits: usize, b_bits: usize) -> (usize, usize, usize);
// Returns (piece_bits, n, fermat_shift)
```

---

## 4. fibcalc-orchestration

Parallel execution, calculator selection, and result analysis.

### Re-exports

```rust
pub use interfaces::{ProgressReporter, ResultPresenter};
pub use orchestrator::{analyze_comparison_results, execute_calculations};
```

---

### `execute_calculations` (function)

Execute calculations with all given calculators. Single-calculator runs execute directly; multiple-calculator runs use `rayon` for parallelism.

```rust
pub fn execute_calculations(
    calculators: &[Arc<dyn Calculator>],
    n: u64,
    opts: &Options,
    cancel: &CancellationToken,
    timeout: Option<Duration>,
) -> Vec<CalculationResult>;
```

### `execute_calculations_with_observer` (function)

Same as `execute_calculations` but accepts a `ProgressObserver`.

```rust
pub fn execute_calculations_with_observer(
    calculators: &[Arc<dyn Calculator>],
    n: u64,
    opts: &Options,
    cancel: &CancellationToken,
    timeout: Option<Duration>,
    observer: &dyn ProgressObserver,
) -> Vec<CalculationResult>;
```

---

### `analyze_comparison_results` (function)

Compare results from multiple calculators. Returns `Ok(())` if all valid results match, `Err(FibError::Mismatch)` if they differ, or `Err(FibError::Calculation)` if no valid results exist.

```rust
/// # Errors
/// Returns `FibError::Calculation` if no valid results exist, or
/// `FibError::Mismatch` if results disagree.
pub fn analyze_comparison_results(results: &[CalculationResult]) -> Result<(), FibError>;
```

---

### `CalculationResult` (struct)

Result of a single calculation.

```rust
pub struct CalculationResult {
    pub algorithm: String,
    pub value: Option<BigUint>,
    pub duration: Duration,
    pub error: Option<String>,
}
```

---

### `ProgressReporter` (trait)

Trait for reporting progress to the user (used by presentation layers).

```rust
pub trait ProgressReporter: Send + Sync {
    fn report(&self, update: &ProgressUpdate);
    fn complete(&self);
}
```

---

### `ResultPresenter` (trait)

Trait for presenting results to the user.

```rust
pub trait ResultPresenter: Send + Sync {
    fn present_result(
        &self,
        algorithm: &str,
        n: u64,
        result: &BigUint,
        duration: Duration,
        details: bool,
    );
    fn present_comparison(&self, results: &[CalculationResult]);
    fn present_error(&self, error: &str);
}
```

---

### `NullProgressReporter` (struct)

No-op progress reporter.

```rust
pub struct NullProgressReporter;
impl ProgressReporter for NullProgressReporter { /* no-op */ }
```

---

### `get_calculators_to_run` (function)

Select calculators by algorithm name. Passing `"all"` returns all available calculators.

```rust
/// # Errors
/// Returns `FibError` if the requested algorithm name is unknown.
pub fn get_calculators_to_run(
    algo: &str,
    factory: &dyn CalculatorFactory,
) -> Result<Vec<Arc<dyn Calculator>>, FibError>;
```

**Example:**
```rust
use fibcalc_core::registry::DefaultFactory;
use fibcalc_orchestration::calculator_selection::get_calculators_to_run;

let factory = DefaultFactory::new();
let calcs = get_calculators_to_run("all", &factory).unwrap();
assert_eq!(calcs.len(), 3);
```

---

## 5. fibcalc-cli

CLI output, progress display, and shell completion.

### Re-exports

```rust
pub use presenter::{CLIProgressReporter, CLIResultPresenter};
```

---

### `CLIProgressReporter` (struct)

Prints progress updates to stderr with carriage returns for in-place updates.

```rust
pub struct CLIProgressReporter { /* ... */ }

impl CLIProgressReporter {
    pub fn new(quiet: bool) -> Self;
}
impl ProgressReporter for CLIProgressReporter { /* ... */ }
```

In quiet mode, all output is suppressed.

---

### `CLIResultPresenter` (struct)

Presents calculation results to stdout. In quiet mode, only the bare number is printed.

```rust
pub struct CLIResultPresenter { /* ... */ }

impl CLIResultPresenter {
    pub fn new(verbose: bool, quiet: bool) -> Self;
}
impl ResultPresenter for CLIResultPresenter { /* ... */ }
```

---

### `ETACalculator` (struct)

Tracks progress and estimates time remaining.

```rust
pub struct ETACalculator { /* ... */ }

impl ETACalculator {
    pub fn new() -> Self;
    pub fn update(&mut self, progress: f64) -> Option<Duration>;  // returns estimated remaining time
    pub fn elapsed(&self) -> Duration;
}
impl Default for ETACalculator { /* ... */ }
```

Returns `None` at progress <= 0.0 or >= 1.0. Between those bounds, estimates remaining time based on elapsed time and current progress fraction.

---

### Shell Completion

The `fibcalc_cli::completion` module provides:

```rust
pub fn generate_completion(cmd: &mut clap::Command, shell: clap_complete::Shell, out: &mut dyn Write);
```

Generates shell completion scripts for bash, zsh, fish, PowerShell, and elvish.

---

## 6. fibcalc-tui

Interactive TUI dashboard using `ratatui` with Elm architecture (Model-Update-View).

### Re-exports

```rust
pub use bridge::{TuiBridgeObserver, TUIProgressReporter, TUIResultPresenter};
pub use logs::LogScrollState;
pub use messages::{SystemMetrics, TuiMessage};
pub use metrics::MetricsCollector;
pub use model::TuiApp;
pub use sparkline::SparklineBuffer;
```

---

### `TuiApp` (struct)

Main TUI application state (Elm Model).

```rust
pub struct TuiApp {
    pub should_quit: bool,
    pub paused: bool,
    pub progress: Vec<f64>,
    pub algorithms: Vec<String>,
    pub completed: Vec<(String, Duration)>,
    pub logs: Vec<String>,
    pub sparkline_data: Vec<f64>,
    pub start_time: Option<Instant>,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub log_scroll_offset: usize,
    pub log_auto_scroll: bool,
    pub show_details: bool,
    pub show_logs: bool,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub throughput_bits_per_sec: f64,
    pub n_value: u64,
    pub errors: Vec<String>,
    pub finished_elapsed: Option<Duration>,
    /* ... private fields ... */
}

impl TuiApp {
    pub fn new(rx: Receiver<TuiMessage>) -> Self;
    pub fn set_n(&mut self, n: u64);
    pub fn generation(&self) -> u64;
    pub fn update(&mut self);                        // Elm Update: process pending messages
    pub fn handle_message(&mut self, msg: TuiMessage);
    pub fn handle_key_action(&mut self, action: KeyAction);
    pub fn elapsed(&self) -> Option<Duration>;       // frozen after Finished
    pub fn render(&self, frame: &mut ratatui::Frame); // Elm View
    pub fn run(&mut self) -> io::Result<()>;          // Full event loop
    pub fn page_up(&mut self, page_size: usize);
    pub fn page_down(&mut self, page_size: usize);
    pub fn scroll_home(&mut self);
    pub fn scroll_end(&mut self);

    // Layout helpers (static)
    pub fn compute_layout(area: Rect) -> (Rect, Rect, Rect, Rect);
    pub fn compute_info_layout(info_area: Rect) -> (Rect, Rect);
    pub fn compute_metrics_layout(metrics_area: Rect) -> (Rect, Rect);
    pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>>;
    pub fn teardown_terminal(terminal: &mut Terminal<...>) -> io::Result<()>;
}
```

---

### `TuiMessage` (enum)

Messages that drive the TUI update cycle.

```rust
pub enum TuiMessage {
    Progress { index: usize, progress: f64, algorithm: String },
    Log(String),
    SparklineData(f64),
    Started,
    Complete { algorithm: String, duration: Duration },
    Quit,
    Tick,
    Resize { width: u16, height: u16 },
    KeyPress(KeyAction),
    Error(String),
    SystemMetrics(SystemMetrics),
    Finished,
}
```

---

### `SystemMetrics` (struct)

```rust
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub throughput_bits_per_sec: f64,
}
```

---

### `TuiBridgeObserver` (struct)

Core-level `ProgressObserver` implementation that forwards progress updates to a TUI channel.

```rust
pub struct TuiBridgeObserver { /* ... */ }

impl TuiBridgeObserver {
    pub fn new(tx: Sender<TuiMessage>) -> Self;
}
impl ProgressObserver for TuiBridgeObserver { /* ... */ }
```

---

### `TUIProgressReporter` (struct)

`ProgressReporter` implementation that sends messages to the TUI.

```rust
pub struct TUIProgressReporter { /* ... */ }

impl TUIProgressReporter {
    pub fn new(tx: Sender<TuiMessage>) -> Self;
}
impl ProgressReporter for TUIProgressReporter { /* ... */ }
```

---

### `TUIResultPresenter` (struct)

`ResultPresenter` implementation that sends messages to the TUI.

```rust
pub struct TUIResultPresenter { /* ... */ }

impl TUIResultPresenter {
    pub fn new(tx: Sender<TuiMessage>) -> Self;
}
impl ResultPresenter for TUIResultPresenter { /* ... */ }
```

---

### `MetricsCollector` (struct)

Collects system metrics (CPU, memory) for display in the TUI. Runs on a background thread.

```rust
pub struct MetricsCollector { /* ... */ }

impl MetricsCollector {
    pub fn new() -> Self;
    pub fn refresh(&mut self);
    pub fn snapshot(&self) -> SystemMetrics;
}
```

---

### `SparklineBuffer` (struct)

Ring buffer for sparkline data, scaling `f64` values to `u64` (multiplied by 100).

```rust
pub struct SparklineBuffer { /* ... */ }

impl SparklineBuffer {
    pub fn new(capacity: usize) -> Self;
    pub fn push(&mut self, value: f64);
    pub fn data(&self) -> &[u64];
    pub fn capacity(&self) -> usize;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn clear(&mut self);
}
```

---

### Render Functions

The `fibcalc_tui` crate provides widget rendering functions:

| Function | Description |
|----------|-------------|
| `render_sparkline(frame, area, data, title)` | Render sparkline using ratatui's built-in widget |
| `render_braille_sparkline(frame, area, data, title)` | Render high-resolution sparkline using Braille characters |
| `render_header(frame, area, n, algo_display)` | Render TUI header |
| `render_footer(frame, area)` | Render TUI footer with key hints |
| `render_progress(frame, area, algorithms, progress)` | Render progress bars |
| `render_metrics(frame, area, elapsed, memory_mb, cpu_percent)` | Render system metrics panel |
| `render_logs(frame, area, logs, scroll_offset)` | Render scrollable log panel |

---

## 7. fibcalc-calibration

Auto-tuning, adaptive benchmarks, and calibration profiles.

### Re-exports

```rust
pub use calibration::{CalibrationEngine, CalibrationMode};
pub use profile::CalibrationProfile;
```

---

### `CalibrationMode` (enum)

```rust
pub enum CalibrationMode {
    Full,    // Run all benchmarks (slower, more accurate)
    Auto,    // Quick adaptive benchmarks
    Cached,  // Load from profile file
}
```

---

### `CalibrationEngine` (struct)

Determines optimal thresholds for the current hardware.

```rust
pub struct CalibrationEngine { /* ... */ }

impl CalibrationEngine {
    pub fn new(mode: CalibrationMode) -> Self;
    pub fn with_progress(self, cb: ProgressCallback) -> Self;
    pub fn calibrate(&self) -> CalibrationProfile;
}
```

**`ProgressCallback`:**
```rust
pub type ProgressCallback = Box<dyn Fn(CalibrationProgress) + Send>;
```

**Example:**
```rust
use fibcalc_calibration::{CalibrationEngine, CalibrationMode};

let engine = CalibrationEngine::new(CalibrationMode::Auto);
let profile = engine.calibrate();
println!("FFT threshold: {} bits", profile.fft_threshold);
```

---

### `CalibrationProgress` (struct)

```rust
pub struct CalibrationProgress {
    pub step: String,
    pub current: usize,
    pub total: usize,
}
```

---

### `CalibrationProfile` (struct)

Serializable calibration profile. Stored as JSON.

```rust
pub struct CalibrationProfile {
    pub version: u32,
    pub parallel_threshold: usize,
    pub fft_threshold: usize,
    pub strassen_threshold: usize,
    pub cpu_model: String,
    pub num_cores: usize,
    pub cpu_fingerprint: String,
    pub timestamp: String,
}

impl CalibrationProfile {
    pub fn is_compatible(&self) -> bool;
    pub fn matches_cpu(&self, current_fingerprint: &str) -> bool;
    pub fn is_valid(&self) -> bool;
}
impl Default for CalibrationProfile { /* static defaults */ }
```

---

### I/O Functions

```rust
// fibcalc_calibration::io

/// # Errors
/// Returns an I/O error if the file cannot be written.
pub fn save_profile(profile: &CalibrationProfile) -> std::io::Result<()>;

/// # Errors
/// Returns an I/O error if the file cannot be written.
pub fn save_to_path(profile: &CalibrationProfile, path: &std::path::Path) -> std::io::Result<()>;

/// # Errors
/// Returns an I/O error if the file exists but cannot be deleted.
pub fn delete_profile() -> std::io::Result<bool>;

pub fn load_profile() -> Option<CalibrationProfile>;
pub fn load_validated_profile() -> Option<CalibrationProfile>;
```

Profiles are saved to `.fibcalc_calibration.json` in the XDG config directory (or the working directory as fallback).

---

### Helper Functions

```rust
// fibcalc_calibration::profile
pub fn cpu_fingerprint() -> String;      // "cores=N"
pub fn cpu_model() -> String;            // CPU brand string via sysinfo
pub fn current_timestamp() -> String;    // "unix:EPOCH_SECS"
pub const PROFILE_VERSION: u32 = 1;
```
