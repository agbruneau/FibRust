# FibCalc-rs

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![MSRV](https://img.shields.io/badge/MSRV-1.80%2B-orange)
![Tests](https://img.shields.io/badge/tests-669%2B-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-96.1%25-brightgreen)

High-performance Fibonacci calculator in Rust. Computes arbitrarily large Fibonacci numbers using three cross-validated algorithms. Academic prototype ported from [FibGo](https://github.com/agbruneau/Fibonacci) (Go).

## Quick Start

```bash
git clone https://github.com/agbruneau/FibRust.git
cd FibRust
cargo build --release
```

## Installation

**Default (pure Rust, zero system dependencies):**

```bash
cargo install --path crates/fibcalc
```

**With GMP (faster big-integer arithmetic via libgmp):**

```bash
# Install libgmp first:
#   Linux:  sudo apt-get install libgmp-dev
#   macOS:  brew install gmp
cargo install --path crates/fibcalc --features gmp
```

See [docs/INSTALLATION.md](docs/INSTALLATION.md) for Docker, Windows, cross-compilation, and troubleshooting.

## Usage Examples

```bash
# Compute F(1000) and display the result
fibcalc -n 1000 -c

# Compare all three algorithms on F(50,000) with timing details
fibcalc -n 50000 --algo all -d

# Launch interactive TUI dashboard
fibcalc --tui

# Auto-calibrate thresholds for your hardware
fibcalc --auto-calibrate

# Compute F(1,000,000), save to file
fibcalc -n 1000000 --algo fast -c -o result.txt

# Only compute last 100 digits of F(10,000,000)
fibcalc -n 10000000 --last-digits 100 -c
```

## Architecture

Cargo workspace with 8 crates in a four-layer architecture:

```
fibcalc (binary)              -- CLI entry point, config, error handling
    |
fibcalc-orchestration         -- parallel execution, result aggregation
    |
fibcalc-core + fibcalc-bigfft -- algorithms, FFT multiplication
    |            |
fibcalc-memory                -- unified memory: pools, arenas, warming
    |
fibcalc-cli / fibcalc-tui     -- CLI output / TUI dashboard
fibcalc-calibration            -- auto-tuning, adaptive benchmarks
```

| Crate                   | Role                                                 |
| ----------------------- | ---------------------------------------------------- |
| `fibcalc`               | Binary entry point, clap config, error handling      |
| `fibcalc-core`          | Fibonacci algorithms, strategies, observers, traits  |
| `fibcalc-bigfft`        | FFT multiplication, Fermat numbers, bump allocator   |
| `fibcalc-memory`        | Unified memory: BigInt pools, bump arenas, thread-local pools, warming |
| `fibcalc-orchestration` | Parallel execution, calculator selection, analysis   |
| `fibcalc-cli`           | CLI output, progress bars, shell completion          |
| `fibcalc-tui`           | Interactive TUI dashboard (ratatui, Elm MVU)         |
| `fibcalc-calibration`   | Auto-tuning, micro-benchmarks, calibration profiles  |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full trait hierarchy, crate dependency graph, and design patterns.

## Algorithms

| Algorithm              | Complexity     | Best for                        |
| ---------------------- | -------------- | ------------------------------- |
| **Fast Doubling**      | O(log n) muls  | General use, fastest for most n |
| **Matrix Exponentiation** | O(log n) muls | Independent cross-validation   |
| **FFT-Based**          | O(n log n)     | Very large n (multiplication-dominated) |

All three run in parallel during cross-validation mode (`--algo all`).

See [docs/ALGORITHMS.md](docs/ALGORITHMS.md) for mathematical foundations and proofs.

## Performance

Benchmarked on Windows 11, release mode with LTO, native CPU:

| N         | Fast Doubling | Matrix Exp. | FFT-Based | Digits  |
| --------- | ------------- | ----------- | --------- | ------- |
| 1,000     | 21 us         | -           | -         | 209     |
| 10,000    | 124 us        | 120 us      | 68 us     | 2,090   |
| 1,000,000 | 5.8 ms        | 26.7 ms     | 11.5 ms   | 208,988 |

See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for detailed benchmarks and calibration.

## Testing

```bash
cargo test --workspace               # All tests (669+)
cargo test -p fibcalc-core            # Single crate
cargo test --test golden              # Golden file tests
cargo test --test e2e                 # End-to-end CLI tests
```

**96.1% line coverage** across 8 crates. Test types: unit, golden file, property-based (`proptest`), end-to-end (`assert_cmd`), fuzz (`cargo-fuzz`), and benchmarks (`criterion`).

## Features

- **Three algorithms** with automatic cross-validation
- **Interactive TUI** with live progress, sparklines, and charts
- **Auto-calibration** for hardware-specific threshold tuning
- **Massive scale**: F(100,000,000)+ with configurable memory limits and timeouts
- **Optional GMP support** via the `rug` crate for faster arithmetic
- **Shell completion** for bash, zsh, fish, PowerShell, and elvish
- **Zero unsafe code**: `unsafe_code = "forbid"` at workspace level

## Documentation

| Document | Description |
| -------- | ----------- |
| [INSTALLATION.md](docs/INSTALLATION.md) | Multi-platform installation, Docker, troubleshooting |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Four-layer architecture, traits, crate dependencies |
| [ALGORITHMS.md](docs/ALGORITHMS.md) | Mathematical foundations, proofs, complexity analysis |
| [PERFORMANCE.md](docs/PERFORMANCE.md) | Benchmarks, calibration, profiling |
| [API_REFERENCE.md](docs/API_REFERENCE.md) | Public API for all 8 crates |
| [CONTRIBUTING.md](docs/CONTRIBUTING.md) | Dev setup, code style, PR process |
| [CHANGELOG.md](docs/CHANGELOG.md) | Release history |

## License

Apache-2.0. See [LICENSE](LICENSE) for details.

The optional `gmp` feature links against libgmp (LGPL). When using `--features gmp`, the combined work must comply with LGPL terms.
