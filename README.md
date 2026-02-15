# FibCalc-rs

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![MSRV](https://img.shields.io/badge/MSRV-1.80%2B-orange)
![Tests](https://img.shields.io/badge/tests-669%2B-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-96.1%25-brightgreen)

This academic prototype in software engineering, is an High-performance Fibonacci calculator written in Rust. Computes arbitrarily large Fibonacci numbers using three algorithms with automatic cross-validation. Ported from [FibGo](https://github.com/agbruneau/Fibonacci) (Go).

## Table of Contents

- [Academic Context](#academic-context)
- [Features](#features)
- [Quick Start](#quick-start)
- [Performance](#performance)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Design Patterns](#design-patterns)
- [Algorithms](#algorithms)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Configuration](#configuration)
- [Documentation](#documentation)
- [Development](#development)
- [Cross-Compilation](#cross-compilation)
- [References](#references)
- [License](#license)

## Academic Context

This project is a port of [FibGo](https://github.com/agbruneau/Fibonacci) (Go) to Rust, designed to demonstrate the exploitation of Rust's type system and performance guarantees for high-performance numerical computing. The port showcases:

- **Ownership and borrowing** for zero-copy result passing and arena-based FFT allocation
- **Trait-based polymorphism** with static dispatch (`impl Trait`) and dynamic dispatch (`dyn Trait`) for algorithm selection
- **Zero-cost abstractions** via generics, iterators, and inlining that produce optimal machine code
- **Fearless concurrency** with `rayon` for data parallelism and `crossbeam` channels for communication, without data races

The complete specification is documented in [docs/PRD.md](docs/PRD.md) (~9,500 lines, 78 tasks across 7 phases).

## Features

- **Three algorithms**: Fast Doubling, Matrix Exponentiation, FFT-Based multiplication
- **Cross-validation**: Run all algorithms and verify results match
- **Interactive TUI**: Terminal dashboard with live progress, sparklines, and charts (powered by [ratatui](https://ratatui.rs))
- **Auto-calibration**: Adaptive threshold tuning based on your hardware
- **Massive scale**: Compute F(100,000,000)+ with configurable memory limits and timeouts
- **Optional GMP support**: Link against libgmp via the `rug` crate for even faster arithmetic
- **Shell completion**: Generated for bash, zsh, fish, PowerShell, and elvish
- **Zero unsafe code**: `unsafe_code = "forbid"` enforced at workspace level

## Quick Start

```bash
# Build
cargo build --release

# Compute F(1000)
cargo run --release -p fibcalc -- -n 1000 --algo fast -c

# Compare all algorithms on F(10,000)
cargo run --release -p fibcalc -- -n 10000 --algo all -d

# Launch interactive TUI
cargo run --release -p fibcalc -- --tui
```

## Performance

Benchmarked on a single run (Windows 11, release mode with LTO, native CPU):

| N         | Fast Doubling | Matrix Exp. | FFT-Based | Digits  |
| --------- | ------------- | ----------- | --------- | ------- |
| 1,000     | 21 us         | -           | -         | 209     |
| 10,000    | 124 us        | 120 us      | 68 us     | 2,090   |
| 1,000,000 | 5.8 ms        | 26.7 ms     | 11.5 ms   | 208,988 |

> **Note**: Builds use `-C target-cpu=native` via `.cargo/config.toml`, so binaries are optimized for the local CPU and may not be portable.

## Installation

### Prerequisites

- Rust 1.80+ (MSRV)
- (Optional) libgmp for GMP support

### Build

```bash
# Default (pure Rust, no system dependencies)
cargo build --release

# With GMP support (requires libgmp installed)
cargo build --release --features gmp
```

### Cargo Features

| Feature     | Description                                                         |
| ----------- | ------------------------------------------------------------------- |
| `default` | Pure Rust, no external system dependencies                          |
| `gmp`     | GMP support via `rug` crate (`dep:rug`). LGPL, requires libgmp. |

## Usage

```
fibcalc [OPTIONS]

Options:
  -n, --n <N>                 Fibonacci number to compute [default: 100000000]
      --algo <ALGO>           Algorithm: fast, matrix, fft, or all [default: all]
  -c, --calculate             Calculate and display the result
  -v, --verbose               Verbose output
  -d, --details               Show detailed information
  -o, --output <OUTPUT>       Output file path
  -q, --quiet                 Quiet mode (only output the number)
      --tui                   Launch interactive TUI
      --calibrate             Run full calibration
      --auto-calibrate        Run automatic calibration
      --timeout <TIMEOUT>     Timeout duration (e.g., "5m", "1h") [default: 5m]
      --threshold <BITS>      Parallel multiplication threshold in bits
      --fft-threshold <BITS>  FFT multiplication threshold in bits
      --strassen-threshold <BITS>  Strassen multiplication threshold in bits
      --last-digits <K>       Compute only last K digits
      --memory-limit <LIM>    Memory limit (e.g., "8G", "512M")
      --completion <SHELL>    Generate shell completion (bash, zsh, fish, powershell, elvish)
```

### Examples

```bash
# Calculate and print F(500)
fibcalc -n 500 -c

# Compare all algorithms with timing details
fibcalc -n 50000 --algo all -d

# Save result to file
fibcalc -n 1000000 --algo fast -c -o result.txt

# Only compute last 100 digits
fibcalc -n 10000000 --last-digits 100 -c

# Run with memory limit and timeout
fibcalc -n 100000000 --memory-limit 4G --timeout 10m

# Generate shell completion
fibcalc --completion bash > fibcalc.bash
```

## Architecture

Cargo workspace with 7 crates in a four-layer architecture (Rust 2021 edition, resolver v2):

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

| Crate                     | Lines  | Role                                                                                  |
| ------------------------- | ------ | ------------------------------------------------------------------------------------- |
| `fibcalc`               | ~1,170 | Binary entry point, clap config, app orchestration, error handling                    |
| `fibcalc-core`          | ~5,090 | Fibonacci algorithms, strategies, observers, dynamic thresholds, arena, memory budget |
| `fibcalc-bigfft`        | ~2,780 | FFT multiplication, Fermat numbers, transform cache, bump allocator, pools            |
| `fibcalc-orchestration` | ~550   | Parallel execution, calculator selection, result analysis                             |
| `fibcalc-cli`           | ~620   | CLI output, progress bars (indicatif), ETA, shell completion                          |
| `fibcalc-tui`           | ~3,080 | Interactive TUI dashboard (ratatui + crossterm, Elm architecture)                     |
| `fibcalc-calibration`   | ~1,230 | Auto-tuning, adaptive benchmarks, calibration profiles                                |

**Total**: ~14,500 lines of Rust across 77 source files.

### Key Traits

| Trait                    | File                               | Purpose                                                      |
| ------------------------ | ---------------------------------- | ------------------------------------------------------------ |
| `Calculator`           | `fibcalc-core/src/calculator.rs` | Public trait for orchestration (`calculate()`, `name()`) |
| `CoreCalculator`       | `fibcalc-core/src/calculator.rs` | Internal trait for algorithm implementations                 |
| `Multiplier`           | `fibcalc-core/src/strategy.rs`   | Narrow interface for multiply/square                         |
| `DoublingStepExecutor` | `fibcalc-core/src/strategy.rs`   | Extended strategy for optimized doubling steps               |
| `ProgressObserver`     | `fibcalc-core/src/observer.rs`   | Observer pattern with lock-free `Freeze()` snapshots       |
| `CalculatorFactory`    | `fibcalc-core/src/registry.rs`   | Factory with lazy creation and `RwLock<HashMap>` cache     |

### Crate Dependency Graph

```
fibcalc
├── fibcalc-core
│   └── fibcalc-bigfft
├── fibcalc-orchestration
│   └── fibcalc-core
├── fibcalc-cli
│   ├── fibcalc-core
│   └── fibcalc-orchestration
├── fibcalc-tui
│   ├── fibcalc-core
│   └── fibcalc-orchestration
└── fibcalc-calibration
    └── fibcalc-core
```

## Project Structure

```
FibRust/
├── Cargo.toml                  # Workspace root (7 members)
├── deny.toml                   # License & security policy
├── .cargo/config.toml          # Build flags (target-cpu=native)
├── crates/
│   ├── fibcalc/                # Binary entry point (6 files, ~1,170 lines)
│   │   ├── src/                #   main.rs, lib.rs, app.rs, config.rs, errors.rs, version.rs
│   │   └── tests/              #   e2e.rs, proptest.rs
│   ├── fibcalc-core/           # Core algorithms (28 files, ~5,090 lines)
│   │   ├── src/                #   calculator, fastdoubling, matrix, fft_based, strategy, observer, ...
│   │   ├── tests/              #   properties.rs
│   │   └── benches/            #   fibonacci.rs (criterion)
│   ├── fibcalc-bigfft/         # FFT multiplication (14 files, ~2,780 lines)
│   ├── fibcalc-orchestration/  # Parallel execution (4 files, ~550 lines)
│   ├── fibcalc-cli/            # CLI output (6 files, ~620 lines)
│   ├── fibcalc-tui/            # TUI dashboard (12 files, ~3,080 lines)
│   └── fibcalc-calibration/    # Auto-tuning (7 files, ~1,230 lines)
├── tests/
│   ├── golden.rs               # Workspace-level golden file tests
│   └── testdata/
│       └── fibonacci_golden.json  # Golden values F(0)..F(1,000,000)
├── fuzz/
│   └── fuzz_targets/           # 4 fuzz targets (fast_doubling, matrix, fft, cross_algorithm)
└── docs/                       # 10 technical documents
```

## Design Patterns

The project implements several GoF and systems patterns mapped from Go idioms to Rust:

| Pattern                      | Implementation                                                                               | Location                              |
| ---------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------- |
| **Decorator**          | `FibCalculator` wraps `CoreCalculator`, adds fast path (n <= 93) and progress reporting  | `fibcalc-core/src/calculator.rs`    |
| **Factory + Registry** | `DefaultFactory` with lazy creation and `RwLock<HashMap>` cache                          | `fibcalc-core/src/registry.rs`      |
| **Strategy + ISP**     | `Multiplier` (narrow, 2 methods) and `DoublingStepExecutor` (broad, optimized steps)     | `fibcalc-core/src/strategy.rs`      |
| **Observer**           | `ProgressSubject`/`ProgressObserver` with lock-free `Freeze()` snapshots for hot loops | `fibcalc-core/src/observer.rs`      |
| **Arena**              | `bumpalo::Bump` allocator for FFT temporaries, avoiding per-allocation overhead            | `fibcalc-bigfft/src/bump.rs`        |
| **Zero-copy**          | `std::mem::take` / `std::mem::replace` for result return without cloning                 | `fibcalc-core/src/fast_doubling.rs` |

## Algorithms

### Fast Doubling

Uses the identities F(2k) = F(k)[2F(k+1) - F(k)] and F(2k+1) = F(k)^2 + F(k+1)^2 to compute F(n) in O(log n) multiplications. Generally the fastest algorithm for most input sizes.

### Matrix Exponentiation

Raises the matrix [[1,1],[1,0]] to the nth power using binary exponentiation. O(log n) matrix multiplications. Provides independent cross-validation.

### FFT-Based

Fast Doubling with FFT-accelerated big-number multiplication using Fermat Number Transform. Faster for very large n where multiplication cost dominates.

See [docs/ALGORITHMS.md](docs/ALGORITHMS.md) for mathematical foundations, proofs, and complexity analysis.

## Testing

```bash
cargo test --workspace               # All tests (669)
cargo test --lib                      # Unit tests only
cargo test --test golden              # Golden file tests
cargo test --test e2e                 # End-to-end CLI tests
cargo test -p fibcalc-core            # Tests for a specific crate
cargo test -- --nocapture             # With stdout output
```

### Test Coverage

**96.1% line coverage** | **97.0% function coverage** (measured with `cargo-llvm-cov`)

| Crate                                                      | Tests | Line Coverage |
| ---------------------------------------------------------- | ----- | ------------- |
| `fibcalc-core`                                           | 195   | 96-100%       |
| `fibcalc-tui`                                            | 153   | 94-100%       |
| `fibcalc-bigfft`                                         | 122   | 87-100%       |
| `fibcalc-calibration`                                    | 43    | 87-100%       |
| `fibcalc`                                                | 33    | 80-100%       |
| `fibcalc-cli`                                            | 42    | 95-100%       |
| `fibcalc-orchestration`                                  | 20    | 95-100%       |
| **Workspace (golden + e2e + proptest + properties)** | 52    | --            |
| **Doc-tests**                                        | 9     | --            |

### Test Types

- **Unit tests**: Inline tests across all modules (table-driven)
- **Integration tests**: Golden file validation against known Fibonacci values F(0) to F(1,000,000)
- **End-to-end tests**: 23 CLI tests via `assert_cmd` covering all modes and error handling
- **Property-based tests**: `proptest` for algorithm agreement and recurrence verification
- **Fuzz testing**: 4 `cargo-fuzz` targets (fast doubling, matrix, fft, cross-algorithm)
- **Benchmarks**: `criterion` suite with HTML reports for all 3 algorithms at multiple input sizes

```bash
# Measure coverage
cargo install cargo-llvm-cov
cargo llvm-cov --workspace          # Text report
cargo llvm-cov --workspace --html   # HTML report in target/llvm-cov/html/
```

## Code Quality

| Tool             | Purpose                                         | Command                                 |
| ---------------- | ----------------------------------------------- | --------------------------------------- |
| `cargo clippy` | Linting (`pedantic` level, 0 warnings target) | `cargo clippy -- -W clippy::pedantic` |
| `cargo fmt`    | Formatting enforcement                          | `cargo fmt --check`                   |
| `cargo audit`  | Security vulnerability detection                | `cargo audit`                         |
| `cargo deny`   | License compatibility via `deny.toml`         | `cargo deny check`                    |
| `cargo fuzz`   | Fuzz testing for edge cases                     | `cargo fuzz run fuzz_fast_doubling`   |
| `cargo geiger` | Unsafe code audit                               | `cargo geiger`                        |

### Build Profiles

The release profile enables LTO (link-time optimization), single codegen unit, symbol stripping, `opt-level = 3`, `panic = "abort"`, and overflow checks. The bench profile inherits from release but keeps debug symbols for profiling.

### Linting Policy

- `unsafe_code = "forbid"` — no unsafe code anywhere in the workspace
- `clippy::pedantic` — warnings on all pedantic lints with [specific exceptions](Cargo.toml) for mathematical variable naming, module repetitions, and documentation

### License Policy (`deny.toml`)

Allowed licenses: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode, Zlib, BSL-1.0, CC0-1.0, MPL-2.0. Unlicensed dependencies are denied. Copyleft is warned.

## Configuration

Configuration precedence: CLI flags > Environment variables (`FIBCALC_*`) > Adaptive calibration > Static defaults.

| Parameter          | Default      | Description                       |
| ------------------ | ------------ | --------------------------------- |
| Parallel threshold | 4,096 bits   | Switch to parallel multiplication |
| FFT threshold      | 500,000 bits | Switch to FFT multiplication      |
| Strassen threshold | 3,072 bits   | Switch to Strassen multiplication |

The calibration system auto-tunes these thresholds per hardware. Profiles are stored in `.fibcalc_calibration.json` (gitignored) with CPU model, core count, and measured optimal thresholds.

```bash
# Run auto-calibration
fibcalc --auto-calibrate

# Run full calibration (more thorough)
fibcalc --calibrate
```

## Documentation

| Document                                       | Description                                                   |
| ---------------------------------------------- | ------------------------------------------------------------- |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md)           | Four-layer architecture, trait hierarchy, crate dependencies  |
| [ALGORITHMS.md](docs/ALGORITHMS.md)               | Mathematical foundations, proofs, complexity analysis         |
| [PERFORMANCE.md](docs/PERFORMANCE.md)             | Benchmarks, calibration system, profiling, optimization flags |
| [API_REFERENCE.md](docs/API_REFERENCE.md)         | Public API for all 7 crates, traits, error types, CLI flags   |
| [CONTRIBUTING.md](docs/CONTRIBUTING.md)           | Dev setup, code style, testing, PR process                    |
| [CROSS_COMPILATION.md](docs/CROSS_COMPILATION.md) | 5 target triples, per-platform build instructions             |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)     | Build, runtime, TUI, and platform-specific issues             |
| [SECURITY.md](docs/SECURITY.md)                   | Vulnerability reporting, unsafe policy, supply chain security |
| [CHANGELOG.md](docs/CHANGELOG.md)                 | Release history (Keep a Changelog format)                     |
| [PRD.md](docs/PRD.md)                             | Complete specification (78 tasks, 7 phases, ~9,500 lines)     |

## Development

```bash
# Lint & format
cargo clippy -- -W clippy::pedantic  # Strict linting
cargo fmt --check                    # Check formatting
cargo fmt                            # Auto-format

# Benchmarks
cargo bench                          # All criterion benchmarks (HTML reports)
cargo bench -- "FastDoubling"        # Specific benchmark

# Security & licensing
cargo audit                          # Security audit
cargo deny check                     # License compatibility

# Fuzz testing
cargo fuzz run fuzz_fast_doubling -- -max_total_time=30
cargo fuzz run fuzz_matrix -- -max_total_time=30
cargo fuzz run fuzz_fft -- -max_total_time=30
cargo fuzz run fuzz_cross_algorithm -- -max_total_time=30
```

### Key Dependencies

| Crate                                 | Role                                          |
| ------------------------------------- | --------------------------------------------- |
| `num-bigint` / `num-traits`       | Big number arithmetic (pure Rust, default)    |
| `rug`                               | GMP bindings (optional `gmp` feature, LGPL) |
| `rayon`                             | Work-stealing parallelism                     |
| `crossbeam` / `crossbeam-channel` | Channels and concurrent primitives            |
| `ratatui` + `crossterm`           | Interactive TUI                               |
| `clap` + `clap_complete`          | CLI parsing + shell completion                |
| `bumpalo`                           | Bump allocator for FFT temporaries            |
| `parking_lot`                       | Fast mutexes/RwLocks                          |
| `tracing` + `tracing-subscriber`  | Structured logging with env-filter            |
| `thiserror` / `anyhow`            | Error derivation / contextual errors          |
| `serde` + `serde_json`            | Serialization (calibration profiles)          |
| `indicatif` + `console`           | Progress bars and terminal utilities          |
| `sysinfo`                           | CPU/system information for calibration        |
| `criterion`                         | Benchmarks with HTML reports (dev)            |
| `proptest`                          | Property-based testing (dev)                  |
| `assert_cmd` + `predicates`       | E2E CLI testing (dev)                         |

## Cross-Compilation

| Target                        | Priority |
| ----------------------------- | -------- |
| `x86_64-unknown-linux-gnu`  | P0       |
| `x86_64-unknown-linux-musl` | P1       |
| `x86_64-pc-windows-msvc`    | P1       |
| `x86_64-apple-darwin`       | P1       |
| `aarch64-apple-darwin`      | P1       |

> **Note**: Remove `-C target-cpu=native` from `.cargo/config.toml` when cross-compiling. See [docs/CROSS_COMPILATION.md](docs/CROSS_COMPILATION.md) for detailed per-platform instructions.

## References

- **Original Go implementation**: [FibGo](https://github.com/agbruneau/Fibonacci) by agbruneau
- **Fast Doubling algorithm**: Karatsuba, A. & Ofman, Y. (1963). "Multiplication of multidigit numbers on automata." *Soviet Physics Doklady*, 7, 595-596.
- **Matrix Exponentiation**: Knuth, D. E. (1997). *The Art of Computer Programming, Volume 2: Seminumerical Algorithms* (3rd ed.). Addison-Wesley.
- **Fermat Number Transform**: Crandall, R. & Fagin, B. (1994). "Discrete weighted transforms and large-integer arithmetic." *Mathematics of Computation*, 62(205), 305-324.
- **Rust language**: [The Rust Programming Language](https://doc.rust-lang.org/book/)

## License

Apache-2.0

Note: The optional `gmp` feature links against libgmp (LGPL). When using `--features gmp`, the combined work must comply with LGPL terms.
