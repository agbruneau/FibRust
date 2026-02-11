# FibCalc-rs

High-performance Fibonacci calculator written in Rust. Computes arbitrarily large Fibonacci numbers using three algorithms with automatic cross-validation. Ported from [FibGo](https://github.com/agbruneau/Fibonacci).

## Features

- **Three algorithms**: Fast Doubling, Matrix Exponentiation, FFT-Based multiplication
- **Cross-validation**: Run all algorithms and verify results match
- **Interactive TUI**: Terminal dashboard with live progress, sparklines, and charts (powered by [ratatui](https://ratatui.rs))
- **Auto-calibration**: Adaptive threshold tuning based on your hardware
- **Massive scale**: Compute F(100,000,000)+ with configurable memory limits and timeouts
- **Optional GMP support**: Link against libgmp via the `rug` crate for even faster arithmetic

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

Benchmarked on a single run (Windows 11, release mode with LTO):

| N | Fast Doubling | Matrix Exp. | FFT-Based | Digits |
|---|---------------|-------------|-----------|--------|
| 1,000 | 21 us | - | - | 209 |
| 10,000 | 124 us | 120 us | 68 us | 2,090 |
| 1,000,000 | 5.8 ms | 26.7 ms | 11.5 ms | 208,988 |

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

## Usage

```
fibcalc [OPTIONS]

Options:
  -n, --n <N>              Fibonacci number to compute [default: 100000000]
      --algo <ALGO>        Algorithm: fast, matrix, fft, or all [default: all]
  -c, --calculate          Calculate and display the result
  -v, --verbose            Verbose output
  -d, --details            Show detailed information
  -o, --output <OUTPUT>    Output file path
  -q, --quiet              Quiet mode (only output the number)
      --tui                Launch interactive TUI
      --calibrate          Run full calibration
      --auto-calibrate     Run automatic calibration
      --timeout <TIMEOUT>  Timeout duration (e.g., "5m", "1h") [default: 5m]
      --last-digits <K>    Compute only last K digits
      --memory-limit <LIM> Memory limit (e.g., "8G", "512M")
      --completion <SHELL> Generate shell completion (bash, zsh, fish, powershell, elvish)
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
```

## Architecture

Cargo workspace with 7 crates in a four-layer architecture:

```
fibcalc (binary)              -- entry point, config, error handling
    |
fibcalc-orchestration         -- parallel execution, result aggregation
    |
fibcalc-core, fibcalc-bigfft  -- algorithms, FFT multiplication
    |
fibcalc-cli, fibcalc-tui      -- CLI output / TUI dashboard
fibcalc-calibration            -- auto-tuning, adaptive benchmarks
```

| Crate | Role |
|-------|------|
| `fibcalc` | Binary entry point, app config, error handling |
| `fibcalc-core` | Fibonacci algorithms, strategies, observers, dynamic thresholds |
| `fibcalc-bigfft` | FFT multiplication, Fermat numbers, transform cache, bump allocator |
| `fibcalc-orchestration` | Parallel execution, calculator selection, result analysis |
| `fibcalc-cli` | CLI output, progress bars, ETA, shell completion |
| `fibcalc-tui` | Interactive TUI dashboard (ratatui + crossterm) |
| `fibcalc-calibration` | Auto-tuning, adaptive benchmarks, calibration profiles |

## Algorithms

### Fast Doubling

Uses the identities F(2k) = F(k)[2F(k+1) - F(k)] and F(2k+1) = F(k)^2 + F(k+1)^2 to compute F(n) in O(log n) multiplications. Generally the fastest algorithm.

### Matrix Exponentiation

Raises the matrix [[1,1],[1,0]] to the nth power using binary exponentiation. O(log n) matrix multiplications.

### FFT-Based

Fast Doubling with FFT-accelerated big-number multiplication using Fermat Number Transform. Faster for very large n where multiplication cost dominates.

## Testing

```bash
cargo test                          # All tests (680+ tests)
cargo test --lib                    # Unit tests only
cargo test --test golden            # Golden file tests
cargo test --test e2e               # End-to-end tests
cargo test -p fibcalc-core          # Tests for a specific crate
```

### Test Coverage

**96.1% line coverage** | **97.0% function coverage** (measured with `cargo-llvm-cov`)

| Crate | Tests | Line Coverage |
|-------|-------|---------------|
| `fibcalc-core` | 182 | 96-100% |
| `fibcalc-tui` | 171 | 94-100% |
| `fibcalc-bigfft` | 121 | 87-100% |
| `fibcalc-calibration` | 43 | 87-100% |
| `fibcalc` | 74 | 80-100% |
| `fibcalc-orchestration` | 21 | 95-100% |
| `fibcalc-cli` | 48 | 95-100% |
| **Workspace golden** | 17 | -- |

Coverage includes unit tests, property-based tests (proptest), golden file tests, and end-to-end CLI tests.

```bash
# Measure coverage (requires cargo-llvm-cov)
cargo install cargo-llvm-cov
cargo llvm-cov --workspace          # Text report
cargo llvm-cov --workspace --html   # HTML report in target/llvm-cov/html/
```

## Development

```bash
cargo clippy -- -W clippy::pedantic  # Strict linting
cargo fmt --check                    # Check formatting
cargo bench                          # Criterion benchmarks
cargo bench -- "FastDoubling"        # Specific benchmark
cargo audit                          # Security audit
cargo deny check                     # License compatibility
```

## Configuration

Configuration precedence: CLI flags > Environment variables (`FIBCALC_*`) > Adaptive calibration > Static defaults.

Default thresholds:
- Parallel multiplication: 4,096 bits
- FFT multiplication: 500,000 bits
- Strassen multiplication: 3,072 bits

## License

Apache-2.0

Note: The optional `gmp` feature links against libgmp (LGPL). When using `--features gmp`, the combined work must comply with LGPL terms.
