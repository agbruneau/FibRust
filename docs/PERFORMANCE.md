# Performance and Benchmarking Guide

This document covers FibCalc-rs performance characteristics, how to run and interpret benchmarks, the auto-calibration system, threshold tuning, memory optimizations, and compilation flags that affect performance.

## Table of Contents

- [Performance Overview](#performance-overview)
- [Running Benchmarks](#running-benchmarks)
- [Profiling](#profiling)
- [Auto-Calibration System](#auto-calibration-system)
- [Threshold Tuning](#threshold-tuning)
- [Memory Optimization](#memory-optimization)
- [Compilation Optimizations](#compilation-optimizations)
- [GMP vs Pure Rust](#gmp-vs-pure-rust)
- [Parallelism Tuning](#parallelism-tuning)
- [Hardware Recommendations](#hardware-recommendations)

---

## Performance Overview

Benchmarked on a single run (Windows 11, release mode with LTO, `target-cpu=native`):

| N | Fast Doubling | Matrix Exp. | FFT-Based | Result Digits |
|---|---------------|-------------|-----------|---------------|
| 1,000 | 21 us | - | - | 209 |
| 10,000 | 124 us | 120 us | 68 us | 2,090 |
| 1,000,000 | 5.8 ms | 26.7 ms | 11.5 ms | 208,988 |

### Algorithm Selection Summary

- **Fast Doubling** is generally the fastest algorithm for most values of N. It uses O(log N) big-integer multiplications via the doubling identities F(2k) = F(k)[2F(k+1) - F(k)] and F(2k+1) = F(k)^2 + F(k+1)^2.
- **Matrix Exponentiation** uses O(log N) 2x2 matrix multiplications. Slower due to more multiply operations per step.
- **FFT-Based** uses Fast Doubling with FFT-accelerated big-number multiplication (Fermat Number Transform). Becomes faster than plain Fast Doubling for very large N where multiplication cost dominates.

### Fast Path

For N <= 93, the result fits in a `u64` and is returned from a precomputed lookup table (`FIB_TABLE`) in constant time. This avoids all big-integer allocation.

---

## Running Benchmarks

FibCalc-rs uses [Criterion](https://bheisler.github.io/criterion.rs/book/) for statistical benchmarks.

### Basic Commands

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark group
cargo bench -- "FastDoubling"
cargo bench -- "MatrixExponentiation"
cargo bench -- "FFTBased"

# Run benchmarks for a specific N value
cargo bench -- "FastDoubling/100000"
```

### Benchmark Structure

The benchmarks are defined in `crates/fibcalc-core/benches/fibonacci.rs`. Three benchmark groups are run, each testing N = 100, 1,000, 10,000, and 100,000:

| Group | Algorithm | What It Measures |
|-------|-----------|------------------|
| `FastDoubling` | `OptimizedFastDoubling` | Pure fast doubling performance |
| `MatrixExponentiation` | `MatrixExponentiation` | Matrix binary exponentiation |
| `FFTBased` | `FFTBasedCalculator` | Fast doubling with FFT multiplication |

### Interpreting Results

Criterion outputs statistical data for each benchmark:

```
FastDoubling/1000       time:   [20.5 us 21.0 us 21.6 us]
                        change: [-1.2% +0.5% +2.1%] (p = 0.58 > 0.05)
                        No change in performance detected.
```

- **time**: [lower bound, estimate, upper bound] of mean execution time.
- **change**: Percentage change compared to the last saved baseline.
- **p-value**: Statistical significance of the change.

### Saving Baselines

```bash
# Save a baseline for future comparison
cargo bench -- --save-baseline my_baseline

# Compare against a saved baseline
cargo bench -- --baseline my_baseline
```

### HTML Reports

Criterion generates HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` in a browser to see plots and detailed statistics.

---

## Profiling

### Using cargo-flamegraph

[cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) generates flamegraph SVGs from profiling data.

```bash
# Install
cargo install flamegraph

# Generate flamegraph for a computation
cargo flamegraph --release --bin fibcalc -- -n 1000000 --algo fast -c

# Output: flamegraph.svg in the current directory
```

### Using perf (Linux)

```bash
# Record with perf
perf record --call-graph dwarf cargo run --release --bin fibcalc -- -n 1000000 --algo fast -c

# Generate report
perf report

# Or convert to flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

### Using Windows Performance Toolkit (Windows)

```powershell
# Use the built-in ETW tracing
cargo build --release
xperf -on DiagEasy
.\target\release\fibcalc.exe -n 1000000 --algo fast -c
xperf -d trace.etl
# Open trace.etl in Windows Performance Analyzer
```

### Debug Symbols in Benchmarks

The `[profile.bench]` configuration inherits from release but enables debug symbols:

```toml
[profile.bench]
inherits = "release"
debug = true
```

This allows profiling tools to resolve function names in optimized benchmark binaries.

### Tracing

FibCalc-rs uses the `tracing` crate for structured logging. Enable detailed computation traces with:

```bash
RUST_LOG=trace cargo run --release --bin fibcalc -- -n 100000 --algo fast -c
```

---

## Auto-Calibration System

The `fibcalc-calibration` crate provides automatic threshold tuning that adapts to your hardware.

### Calibration Modes

| Mode | CLI Flag | Description | Speed |
|------|----------|-------------|-------|
| **Full** | `--calibrate` | Runs comprehensive benchmarks at many bit lengths. Tests Karatsuba multiplication from 1K to 256K bits, finds FFT crossover, measures parallel overhead at 6 sizes, computes Strassen threshold. | ~30-60s |
| **Auto** | `--auto-calibrate` | Quick adaptive estimation using binary search. Samples at exponentially-spaced points (1K to 1M bits) then binary searches the crossover region. | ~5-10s |
| **Cached** | Automatic | Loads previously saved profile from `.fibcalc_calibration.json`. Validates CPU fingerprint and profile version. Falls back to defaults on mismatch. | Instant |

### How Calibration Works

1. **Karatsuba vs FFT crossover**: Benchmarks big-integer multiplication at increasing bit lengths to find where FFT-accelerated multiplication outperforms Karatsuba.
2. **Parallel overhead**: Measures sequential vs parallel (rayon) execution at various operand sizes to find where parallelism becomes beneficial (requires >10% speedup).
3. **Strassen threshold**: Derived as 60% of the FFT threshold, clamped to the default minimum of 3,072 bits.

### Calibration Profiles

Profiles are saved to `.fibcalc_calibration.json` with this structure:

```json
{
  "version": 1,
  "parallel_threshold": 4096,
  "fft_threshold": 500000,
  "strassen_threshold": 3072,
  "cpu_model": "AMD Ryzen 9 7950X",
  "num_cores": 32,
  "cpu_fingerprint": "cores=32",
  "timestamp": "unix:1707600000"
}
```

A profile is invalidated if:
- The `version` field does not match the current `PROFILE_VERSION` (1).
- The `cpu_fingerprint` does not match the current machine.
- Any threshold value is zero.
- The FFT threshold is less than the Strassen threshold.

### Running Calibration

```bash
# Full calibration (recommended for first run on new hardware)
cargo run --release --bin fibcalc -- --calibrate

# Quick auto-calibration
cargo run --release --bin fibcalc -- --auto-calibrate

# Subsequent runs will use the cached profile automatically
cargo run --release --bin fibcalc -- -n 10000000 --algo fast -c
```

---

## Threshold Tuning

FibCalc-rs uses three thresholds to choose multiplication strategies at runtime.

### Threshold Reference

| Threshold | Default | Unit | Controls |
|-----------|---------|------|----------|
| `parallel_threshold` | 4,096 | bits | Minimum operand bit-length to use parallel (rayon) multiplication. Below this, the overhead of spawning tasks exceeds the benefit. |
| `fft_threshold` | 500,000 | bits | Minimum operand bit-length to switch from Karatsuba to FFT-based multiplication. FFT multiplication is asymptotically faster but has higher constant overhead. |
| `strassen_threshold` | 3,072 | bits | Minimum operand bit-length for Strassen-style optimized multiplication. Applies in the range between `strassen_threshold` and `fft_threshold`. |

### Dynamic Threshold Manager

The `DynamicThresholdManager` in `fibcalc-core` adjusts thresholds at runtime based on observed performance:

- **Ring buffer**: Stores the last 32 iteration metrics (configurable via `ring_buffer_size`).
- **Hysteresis**: Requires a minimum benefit of 5% (`hysteresis_factor = 0.05`) before adjusting.
- **Dead zone**: Ignores benefit values within 2% (`dead_zone = 0.02`) to avoid oscillation.
- **Max adjustment**: Limits each adjustment to 10% (`max_adjustment = 0.1`) of the current value.
- **Floors**: Prevents thresholds from dropping below 1,024 bits (FFT) or 512 bits (parallel/Strassen).

Each iteration records:
- Operand bit length
- FFT speedup factor
- Parallel speedup factor
- Duration in nanoseconds
- Which multiplication method was actually used
- Whether a cache hit occurred

### Manual Override

Configuration precedence: **CLI flags > Environment variables > Calibration profile > Static defaults**.

```bash
# Set thresholds via environment variables
FIBCALC_PARALLEL_THRESHOLD=8192 \
FIBCALC_FFT_THRESHOLD=250000 \
FIBCALC_STRASSEN_THRESHOLD=6144 \
cargo run --release --bin fibcalc -- -n 10000000 --algo fast -c
```

---

## Memory Optimization

### Memory Budget System

The `MemoryEstimate` in `fibcalc-core` predicts memory usage before computation starts:

- **Result size**: F(N) has approximately N * log2(phi) bits, or N * 0.6942 / 8 bytes.
- **Temporaries**: Fast Doubling needs ~5x the result size for working variables (FK, FK+1, T1, T2, T3).
- **Total**: result_bytes + temp_bytes.

| N | Estimated Result | Estimated Total |
|---|-----------------|-----------------|
| 1,000,000 | ~87 KB | ~520 KB |
| 10,000,000 | ~868 KB | ~5.2 MB |
| 100,000,000 | ~8.7 MB | ~52 MB |
| 1,000,000,000 | ~87 MB | ~520 MB |

Set a memory limit to prevent out-of-memory conditions:

```bash
# Limit to 4 GB
cargo run --release --bin fibcalc -- -n 100000000 --memory-limit 4G

# Supported suffixes: G (GiB), M (MiB), K (KiB), B (bytes)
```

The computation will refuse to start if the estimated memory exceeds the limit.

### BigInt Pool

The `BigIntPool` in `fibcalc-memory` (re-exported by `fibcalc-bigfft`) reuses `BigUint` allocations to reduce allocation pressure:

- **Size classes**: Organized by powers of 4 (64, 256, 1024, 4096, ...). An acquired value is drawn from the matching or next-higher size class.
- **Max pooled bit length**: 100,000,000 bits. Values larger than this are not pooled.
- **Max per class**: 32 objects per size class by default.
- **Statistics**: Tracks hits, misses, and evictions via lock-free atomic counters.

```
Pool lifecycle:
  acquire(min_bits) -> returns a zeroed BigUint from the pool, or allocates new
  release(value)    -> returns a BigUint to the pool (evicted if too large or pool full)
  clear()           -> drops all pooled objects
  warm(bits, count) -> pre-populates a size class
```

### Pool Warming

The `warming` module in `fibcalc-memory` (exposed via `fibcalc_bigfft::warm_global_pool(n)`) pre-allocates pool entries based on predicted computation sizes:

| N Range | Strategy | Allocations |
|---------|----------|-------------|
| < 1,000 | No warming | - |
| 1,000 - 99,999 | Medium | 6 full-size + 4 half-size temporaries |
| 100,000 - 999,999 | Medium | Same as above |
| >= 1,000,000 | Aggressive (2x scale) | 16 full + 12 half + 8 quarter-size temporaries |

### Arena Allocation

The `BumpArena` in `fibcalc-memory` (re-exported as `CalculationArena` in core and `FFTBumpAllocator` in bigfft) wraps `bumpalo::Bump` for O(1) allocation of FFT temporaries:

- All temporaries are allocated in a contiguous arena.
- A single `reset()` call frees all temporaries at once instead of individual deallocations.
- Pre-size the arena with `with_capacity(bytes)` to avoid resizing during computation.

### In-Place Matrix Operations

`MatrixExponentiation` uses in-place `square_symmetric_into()` and `multiply_symmetric_into()` methods that mutate the matrix directly, avoiding allocation of new `Matrix` structs in the exponentiation hot loop.

---

## Compilation Optimizations

### Release Profile

The workspace `Cargo.toml` defines an aggressive release profile:

```toml
[profile.release]
lto = true           # Link-Time Optimization (full, cross-crate)
codegen-units = 1    # Single codegen unit for maximum optimization
strip = true         # Strip symbols to reduce binary size
opt-level = 3        # Maximum optimization level
panic = "abort"      # No unwinding overhead
```

| Setting | Effect | Trade-off |
|---------|--------|-----------|
| `lto = true` | Enables whole-program optimization across all crates. Allows inlining and dead-code elimination across crate boundaries. | Significantly increases compile time. |
| `codegen-units = 1` | LLVM optimizes the entire crate as a single unit, enabling better inlining and register allocation. | Increases compile time and disables parallel codegen. |
| `strip = true` | Removes debug symbols and section headers from the binary. | No profiling with the release binary (use bench profile instead). |
| `opt-level = 3` | Maximum LLVM optimization including auto-vectorization and aggressive inlining. | Larger binary, longer compile time. |
| `panic = "abort"` | Eliminates unwinding tables and landing pads. | Cannot catch panics; process terminates immediately. |

### Target CPU

The `.cargo/config.toml` configures native CPU targeting:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

This enables architecture-specific instruction sets (AVX2, AVX-512, BMI2, etc.) for the build machine. Binaries built this way are **not portable** to machines with older CPUs.

For portable builds, remove this flag or use a specific target:

```bash
# Portable x86-64 build (baseline SSE2)
RUSTFLAGS="-C target-cpu=x86-64" cargo build --release

# Target a specific microarchitecture
RUSTFLAGS="-C target-cpu=haswell" cargo build --release
RUSTFLAGS="-C target-cpu=znver3" cargo build --release
```

### Profile-Guided Optimization (PGO)

PGO uses runtime profiling data to optimize hot paths. This can provide an additional 5-15% speedup on top of LTO.

```bash
# Step 1: Build instrumented binary
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# Step 2: Run representative workload
./target/release/fibcalc -n 1000000 --algo fast -c
./target/release/fibcalc -n 10000000 --algo all -c

# Step 3: Merge profiling data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data/

# Step 4: Build optimized binary
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" cargo build --release
```

### BOLT (Binary Optimization and Layout Tool)

For further optimization on Linux, BOLT can reorder the binary layout based on profiling data:

```bash
# Requires llvm-bolt installed
perf record -e cycles:u -j any,u -- ./target/release/fibcalc -n 10000000 --algo fast -c
perf2bolt -p perf.data -o perf.fdata ./target/release/fibcalc
llvm-bolt ./target/release/fibcalc -o ./target/release/fibcalc.bolt \
    -data=perf.fdata -reorder-blocks=ext-tsp -reorder-functions=hfsort
```

---

## GMP vs Pure Rust

The optional `gmp` feature links against libgmp via the `rug` crate for significantly faster big-integer arithmetic.

### Building with GMP

```bash
# Install libgmp (varies by platform)
# Ubuntu/Debian: sudo apt install libgmp-dev
# macOS: brew install gmp
# Windows: use MSYS2 or vcpkg

# Build with GMP support
cargo build --release --features gmp
```

### Performance Comparison

GMP's hand-tuned assembly routines (using platform-specific SIMD instructions) are significantly faster than the pure-Rust `num-bigint` for large operands. Expected speedup ranges:

| Operand Size | Expected GMP Speedup |
|-------------|---------------------|
| < 1,000 bits | ~1x (negligible) |
| 10,000 bits | ~2-3x |
| 100,000 bits | ~3-5x |
| 1,000,000+ bits | ~5-10x |

The crossover point where GMP becomes faster than `num-bigint` is typically around 256-512 bits.

### License Implications

GMP is licensed under LGPL. When building with `--features gmp`:
- The combined binary must comply with LGPL terms.
- You must allow users to re-link against a different version of libgmp.
- The default (pure Rust) build has no LGPL obligations (Apache-2.0 only).

---

## Parallelism Tuning

### Rayon Thread Pool

FibCalc-rs uses [rayon](https://docs.rs/rayon/) for CPU-bound parallelism. Rayon's work-stealing thread pool is initialized once and reused.

```bash
# Control the number of worker threads
RAYON_NUM_THREADS=8 cargo run --release --bin fibcalc -- -n 10000000 --algo all -c

# Disable parallelism entirely (single-threaded)
RAYON_NUM_THREADS=1 cargo run --release --bin fibcalc -- -n 10000000 --algo fast -c
```

### Parallel Threshold

The `parallel_threshold` (default: 4,096 bits) controls when rayon is used for multiplication:

- **Below threshold**: Single-threaded multiplication. Avoids the overhead of task spawning, which is typically 1-10 microseconds per `rayon::join`.
- **Above threshold**: Parallel multiplication splits work across threads. Beneficial when multiplication takes long enough to amortize the spawning overhead.

### Parallel FFT Threshold

A separate constant `PARALLEL_FFT_THRESHOLD` (5,000,000 bits) controls when the FFT transform itself is parallelized. FFT transforms below this size are run sequentially within each thread.

### When to Run All Algorithms in Parallel

The `--algo all` flag runs all three algorithms in parallel using `rayon::scope` for cross-validation. This is useful for:
- Verifying correctness by comparing results from independent implementations.
- Utilizing all CPU cores when a single algorithm cannot fully saturate them.

---

## Hardware Recommendations

### CPU

| Workload | Recommended |
|----------|-------------|
| N < 100,000 | Any modern CPU; computation completes in < 1 ms |
| N = 1,000,000 | Modern x86-64 or ARM with fast integer multiply |
| N = 10,000,000 | High-clock-speed CPU with large L3 cache (>= 16 MB) |
| N = 100,000,000+ | High-core-count CPU; benefits from AVX2/AVX-512 with GMP |

### Memory

| N | Minimum RAM |
|---|-------------|
| 1,000,000 | ~1 MB |
| 10,000,000 | ~8 MB |
| 100,000,000 | ~64 MB |
| 1,000,000,000 | ~600 MB |

### Storage

Results for very large N can be enormous when written to disk:
- F(1,000,000) is ~209,000 decimal digits (~209 KB as text).
- F(100,000,000) is ~20,900,000 decimal digits (~20 MB as text).
- F(1,000,000,000) is ~209,000,000 decimal digits (~209 MB as text).

Use an SSD for faster output, or use `--last-digits K` to compute only the last K digits.

### Operating System

All major platforms are supported. Performance is best on Linux due to:
- Faster memory allocation (jemalloc/mmap vs Windows heap).
- Better thread scheduling for rayon workloads.
- Easier access to perf and flamegraph profiling tools.
- GMP builds are simpler (libgmp readily available in package managers).

Windows and macOS performance is within 5-10% of Linux for most workloads when using the same compilation flags.
