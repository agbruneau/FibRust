# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FibCalc-rs is a high-performance Fibonacci calculator ported from Go ([FibGo](https://github.com/agbruneau/Fibonacci)) to Rust. It implements three algorithms (Fast Doubling, Matrix Exponentiation, FFT-Based) with CLI and interactive TUI modes, automatic calibration, dynamic thresholds, and optional GMP support via the `rug` crate.

The full specification is in `docs/PRD.md` (~9,500 lines, 78 tasks across 7 phases).

## Build & Test Commands

```bash
# Build
cargo build --release                                    # Release build (LTO, strip, native CPU)
cargo build --release --features gmp                     # With GMP support (LGPL, needs libgmp)

# Run
cargo run --release -p fibcalc -- -n 1000 --algo fast -c # Compute F(1000)
cargo run --release -p fibcalc -- --tui                  # Interactive TUI

# Test (689 tests total)
cargo test --workspace                                   # All tests
cargo test --lib                                         # Unit tests only
cargo test --test golden                                 # Golden file tests (workspace-level)
cargo test --test e2e                                    # End-to-end CLI tests (crates/fibcalc/tests/)
cargo test -p fibcalc-bigfft                             # Tests for a specific crate
cargo test -- --nocapture                                # Tests with stdout output

# Quality
cargo clippy -- -W clippy::pedantic                      # Strict linting (0 warnings target)
cargo fmt --check                                        # Formatting
cargo audit                                              # Security vulnerabilities
cargo deny check                                         # License compatibility (deny.toml)

# Benchmarks & Coverage
cargo bench                                              # Criterion benchmarks (HTML reports)
cargo bench -- "FastDoubling"                            # Specific benchmark
cargo llvm-cov --workspace --html                        # Coverage report (target: >96%)

# Fuzz testing (4 targets: fast_doubling, matrix, fft, cross_algorithm)
cargo fuzz run fuzz_fast_doubling -- -max_total_time=30
```

## Architecture

Rust 2021 edition, MSRV 1.80+, resolver v2. Four-layer architecture:

```
fibcalc (binary)              -- entry point, config, error handling
    |
fibcalc-orchestration         -- parallel execution, result aggregation
    |
fibcalc-core + fibcalc-bigfft -- algorithms, FFT multiplication
    |
fibcalc-cli / fibcalc-tui     -- CLI output / TUI dashboard
fibcalc-calibration            -- auto-tuning, adaptive benchmarks
```

### Workspace (7 crates in `crates/`)

| Crate | Type | Lines | Role |
|-------|------|-------|------|
| `fibcalc` | bin | ~950 | Entry point, clap config, app orchestration, error handling |
| `fibcalc-core` | lib | ~5,000 | Fibonacci algorithms, traits, strategies, observers, thresholds, arena |
| `fibcalc-bigfft` | lib | ~2,500 | FFT multiplication, Fermat numbers, transform cache, bump allocator |
| `fibcalc-orchestration` | lib | ~550 | Parallel execution, calculator selection, result analysis |
| `fibcalc-cli` | lib | ~700 | CLI output, progress bars (indicatif), shell completion |
| `fibcalc-tui` | lib | ~3,400 | TUI dashboard (ratatui, Elm architecture), sparklines, charts |
| `fibcalc-calibration` | lib | ~1,200 | Auto-tuning, adaptive benchmarks, calibration profiles |

### Key Traits

- **`Calculator`** (`fibcalc-core/src/calculator.rs`): Public trait for orchestration. Methods: `calculate()`, `name()`.
- **`CoreCalculator`** (`fibcalc-core/src/calculator.rs`): Internal trait for algorithm implementations. Wrapped by `FibCalculator` decorator which adds the fast path (n <= 93) and progress reporting.
- **`Multiplier`** (`fibcalc-core/src/strategy.rs`): Narrow interface for multiply/square. Extended by `DoublingStepExecutor` for optimized steps.
- **`ProgressObserver`** (`fibcalc-core/src/observer.rs`): Observer pattern for progress updates. `Freeze()` creates lock-free snapshots for hot loops.
- **`CalculatorFactory`** (`fibcalc-core/src/registry.rs`): `DefaultFactory` with lazy creation and `RwLock<HashMap>` cache.

### Design Patterns

- **Decorator**: `FibCalculator` wraps `CoreCalculator`
- **Factory + Registry**: `DefaultFactory` with `RwLock<HashMap>` cache
- **Strategy + ISP**: `Multiplier` (narrow) and `DoublingStepExecutor` (broad)
- **Observer**: `ProgressSubject`/`ProgressObserver` with lock-free `Freeze()`
- **Arena**: `bumpalo::Bump` for FFT temporaries
- **Zero-copy**: `std::mem::take` / `std::mem::replace` for result return

## Build Configuration

### Release Profile (`Cargo.toml`)

```toml
[profile.release]
lto = true              # Link-time optimization (full)
codegen-units = 1       # Single codegen unit for max LTO
strip = true            # Strip symbols
opt-level = 3           # Maximum optimization
panic = "abort"         # Abort on panic (smaller binary)
overflow-checks = true  # Keep overflow checks even in release
```

### Native CPU (`.cargo/config.toml`)

```toml
rustflags = ["-C", "target-cpu=native"]
```

This means builds are optimized for the local CPU and are **not portable** across different CPU architectures.

### Workspace Lints (`Cargo.toml`)

```toml
[workspace.lints.rust]
unsafe_code = "forbid"   # No unsafe code anywhere

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
# Allowed exceptions:
module_name_repetitions = "allow"  # Crate-prefixed types are fine
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
similar_names = "allow"            # Mathematical variable names (a, b, c)
struct_excessive_bools = "allow"
items_after_statements = "allow"
unused_self = "allow"
if_not_else = "allow"
redundant_else = "allow"
```

## Cargo Features

- `default = []`: Pure Rust, no external system dependencies
- `gmp`: GMP support via `rug` crate (`dep:rug` syntax). LGPL, requires libgmp installed on the system.

## Code Conventions

**Imports**: Group as (1) std, (2) external crates, (3) workspace crates.

**Error handling**: `thiserror` for library errors, `anyhow` in main. Enum `FibError` with variants: Calculation, Config, Cancelled, Timeout, Mismatch, Overflow, InvalidInput.

**Concurrency**: `rayon` for CPU-bound parallelism. `crossbeam::channel` for communication. No tokio (CPU-bound synchronous workload).

**Tests**: Table-driven with `#[test]`. Golden files in `tests/testdata/fibonacci_golden.json`. Property-based via `proptest`. Fuzz targets in `fuzz/fuzz_targets/`. Benchmarks in `crates/fibcalc-core/benches/fibonacci.rs` via `criterion`.

**Linting**: `cargo clippy -- -W clippy::pedantic`. Cyclomatic complexity < 15 per function. `unsafe_code = "forbid"` at workspace level.

**Commits**: Conventional Commits -- `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.

## Configuration Precedence

CLI flags > Environment variables (`FIBCALC_*`) > Adaptive calibration > Static defaults.

Default thresholds: ParallelThreshold=4,096 bits, FFTThreshold=500K bits, StrassenThreshold=3,072 bits.

Calibration profile stored in `.fibcalc_calibration.json` (gitignored).

## Test Structure

| Location | Type | Count |
|----------|------|-------|
| `crates/*/src/**/*.rs` (inline) | Unit tests | ~625 |
| `tests/golden.rs` + `tests/testdata/` | Golden file integration | 17 |
| `crates/fibcalc/tests/e2e.rs` | E2E CLI tests (assert_cmd) | 23 |
| `crates/fibcalc/tests/proptest.rs` | Property-based | 4 |
| `crates/fibcalc-core/tests/properties.rs` | Algorithm properties | 20 |
| `fuzz/fuzz_targets/` | Fuzz (4 targets) | -- |
| `crates/fibcalc-core/benches/fibonacci.rs` | Criterion benchmarks | -- |

## License Policy (`deny.toml`)

Allowed: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode, Zlib, BSL-1.0, CC0-1.0, MPL-2.0. Unlicensed dependencies are denied. Copyleft is warned.

## Implementation Priority

Sprint order: fibcalc-core (types, traits, fast path) -> fibcalc-bigfft (Fermat, FFT, pools) -> algorithms with golden tests -> fibcalc-orchestration -> fibcalc-cli -> fibcalc-tui -> integration & cross-validation.
