# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FibCalc-rs is a high-performance Fibonacci calculator ported from Go (FibGo) to Rust. It implements three algorithms (Fast Doubling, Matrix Exponentiation, FFT-Based), with CLI and interactive TUI modes, automatic calibration, dynamic thresholds, and optional GMP support via the `rug` crate.

The full specification is in `PRDRust.md` (~9300 lines, 78 tasks across 7 phases).

## Build & Test Commands

```bash
cargo build --release                                    # Release build
cargo build --release --features gmp                     # With GMP support (LGPL, needs libgmp)
cargo test                                               # All tests
cargo test -- --nocapture                                # Tests with stdout output
cargo test --lib                                         # Unit tests only
cargo test --test golden                                 # Golden file tests
cargo test --test e2e                                    # End-to-end tests
cargo test -p fibcalc-bigfft                             # Tests for a specific crate
cargo bench                                              # Criterion benchmarks
cargo bench -- "FastDoubling"                            # Specific benchmark
cargo clippy -- -W clippy::pedantic                      # Strict linting (target: 0 warnings)
cargo fmt --check                                        # Check formatting
cargo audit                                              # Security audit of dependencies
cargo deny check                                         # License compatibility check
cargo tarpaulin --out html                               # Code coverage (target: >75%)
cargo fuzz run fuzz_fast_doubling -- -max_total_time=30  # Fuzz testing
```

## Architecture

Rust 2021 edition, MSRV 1.80+. Four-layer architecture:

```
Entry point (src/main.rs)
    |
Orchestration (fibcalc-orchestration) -- parallel execution, result aggregation
    |
Core (fibcalc-core, fibcalc-bigfft)   -- algorithms, FFT multiplication
    |
Presentation (fibcalc-cli, fibcalc-tui) -- CLI output or TUI dashboard
```

### Cargo Workspace (7 crates)

| Crate | Type | Role |
|-------|------|------|
| `fibcalc` | bin | Binary entry point, app config, error handling |
| `fibcalc-core` | lib | Fibonacci algorithms, strategies, observers, dynamic thresholds, arena, memory budget |
| `fibcalc-bigfft` | lib | FFT multiplication, Fermat numbers, transform cache, bump allocator, pools |
| `fibcalc-orchestration` | lib | Parallel execution, calculator selection, result analysis |
| `fibcalc-cli` | lib | CLI output, presenter, progress/ETA, shell completion |
| `fibcalc-tui` | lib | TUI dashboard (ratatui, Elm architecture), sparklines, charts |
| `fibcalc-calibration` | lib | Auto-tuning, adaptive benchmarks, calibration profiles |

### Key Traits

- **`Calculator`** (`fibcalc-core/src/calculator.rs`): Public trait for orchestration. Methods: `calculate()`, `name()`.
- **`CoreCalculator`** (`fibcalc-core/src/calculator.rs`): Internal trait for algorithm implementations. Wrapped by `FibCalculator` decorator which adds the fast path (n <= 93) and progress reporting.
- **`Multiplier`** (`fibcalc-core/src/strategy.rs`): Narrow interface for multiply/square. Extended by `DoublingStepExecutor` for optimized steps.
- **`ProgressObserver`** (`fibcalc-core/src/observer.rs`): Observer pattern for progress updates. `Freeze()` creates lock-free snapshots for hot loops.
- **`CalculatorFactory`** (`fibcalc-core/src/registry.rs`): `DefaultFactory` with lazy creation and cache.

### Design Patterns

- **Decorator**: `FibCalculator` wraps `CoreCalculator`
- **Factory + Registry**: `DefaultFactory` with `RwLock<HashMap>` cache
- **Strategy + ISP**: `Multiplier` (narrow) and `DoublingStepExecutor` (broad)
- **Observer**: `ProgressSubject`/`ProgressObserver` with lock-free `Freeze()`
- **Arena**: `bumpalo::Bump` for FFT temporaries
- **Zero-copy**: `std::mem::take` / `std::mem::replace` for result return

## Cargo Features

- `default`: Pure Rust, no external system dependencies
- `gmp`: GMP support via `rug` (LGPL, requires libgmp)
- `simd`: Explicit SIMD optimizations (auto-detection by default)

## Code Conventions

**Imports**: Group as (1) std, (2) external crates, (3) workspace crates.

**Error handling**: `thiserror` for library errors, `anyhow` in main. Enum `FibError` with variants: Calculation, Config, Cancelled, Timeout, Mismatch.

**Concurrency**: `rayon` for CPU-bound parallelism. `crossbeam::channel` for communication. Semaphore via `rayon::ThreadPool` with limited size. No tokio for computation (CPU-bound synchronous).

**Tests**: Table-driven with `#[test]`. Golden files in `tests/testdata/fibonacci_golden.json`. Property-based via `proptest`. Fuzz targets in `fuzz/fuzz_targets/`. Benchmarks via `criterion`.

**Linting**: `cargo clippy -- -W clippy::pedantic`. Cyclomatic complexity < 15 per function.

**unsafe**: Maximum 5 blocks, all with `// SAFETY:` comments. Audit with `cargo geiger`.

**Commits**: Conventional Commits -- `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.

## Configuration Precedence

CLI flags > Environment variables (`FIBCALC_*`) > Adaptive calibration > Static defaults.

Default thresholds: ParallelThreshold=4096 bits, FFTThreshold=500K bits, StrassenThreshold=3072 bits.

## Key Dependencies

| Crate | Role |
|-------|------|
| `num-bigint` | Big number arithmetic (default, pure Rust) |
| `rug` | GMP bindings (optional `gmp` feature, LGPL) |
| `rayon` | Work-stealing parallelism |
| `crossbeam` | Channels and concurrent primitives |
| `ratatui` + `crossterm` | Interactive TUI |
| `clap` + `clap_complete` | CLI parsing + shell completion |
| `bumpalo` | Bump allocator for FFT |
| `parking_lot` | Fast mutexes/rwlocks |
| `tracing` | Structured logging |
| `thiserror` / `anyhow` | Error derivation / contextual errors |
| `serde` + `serde_json` | Serialization (calibration profiles) |
| `criterion` | Benchmarks (dev) |
| `proptest` | Property-based testing (dev) |
| `insta` | Snapshot/golden testing (dev) |
| `assert_cmd` | E2E CLI testing (dev) |

## Cross-Compilation Targets

| Target | Priority |
|--------|----------|
| `x86_64-unknown-linux-gnu` | P0 |
| `x86_64-unknown-linux-musl` | P1 |
| `x86_64-pc-windows-msvc` | P1 |
| `x86_64-apple-darwin` | P1 |
| `aarch64-apple-darwin` | P1 |

## Go-to-Rust Idiom Reference

| Go Pattern | Rust Equivalent | Crate |
|------------|----------------|-------|
| `interface{}` | `dyn Trait` / `impl Trait` | -- |
| `goroutine` + `errgroup` | `rayon::join` / `rayon::scope` | `rayon` |
| `chan T` | `crossbeam::channel` | `crossbeam` |
| `sync.Pool` | Stack alloc / `bumpalo` | `bumpalo` |
| `context.Context` | `CancellationToken` / `Arc<AtomicBool>` | `tokio-util` |
| `(T, error)` | `Result<T, E>` | `thiserror` |
| `sync.RWMutex` | `parking_lot::RwLock` | `parking_lot` |
| `big.Int` | `BigUint` / `rug::Integer` | `num-bigint` / `rug` |
| `go:build tag` | `#[cfg(feature = "x")]` | Cargo features |
| `go:linkname` asm | `std::arch::x86_64` | -- |
| `init()` auto-register | `inventory::submit!` | `inventory` |
| `sync.Once` | `std::sync::OnceLock` | -- |
| `defer` | `Drop` trait / `scopeguard` | `scopeguard` |
| `select {}` multi-channel | `crossbeam::select!` | `crossbeam` |

## Implementation Priority

Sprint order: fibcalc-core (types, traits, fast path) -> fibcalc-bigfft (Fermat, FFT, pools) -> algorithms with golden tests -> fibcalc-orchestration -> fibcalc-cli -> fibcalc-tui -> integration & cross-validation.

P0 files (35): Core algorithms, interfaces, frameworks, entry point.
P1 files (38): Observers, adaptive calibration, formatting, critical tests.
P2 files (30): TUI, full calibration, shell completion, GMP.
