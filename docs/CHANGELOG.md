# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Code quality**: Implemented comprehensive clippy pedantic lint compliance across all crates.
  - Added `# Errors` documentation sections to all public fallible functions.
  - Added `# Panics` documentation to functions that can panic.
  - Added `#[must_use]` attributes to functions returning values that should not be ignored.
  - Added targeted `#[allow(...)]` attributes with justification comments for intentional lint suppressions (`dead_code`, `clippy::unused_self`, `clippy::similar_names`, `clippy::struct_excessive_bools`).
  - Moved `rayon` imports to module top-level in `fibcalc-orchestration` to follow import conventions.

### Removed

- Removed temporary debug files (`Debug.md`, `DebugPlan.md`).

## [0.1.0] - 2026-02-11

### Added

- Three Fibonacci algorithms: Fast Doubling, Matrix Exponentiation, and FFT-Based multiplication.
- CLI interface with clap argument parsing and comprehensive option set (`-n`, `--algo`, `-c`, `-v`, `-d`, `-o`, `-q`, `--timeout`, `--last-digits`, `--memory-limit`).
- Interactive TUI dashboard with ratatui (Elm architecture), live progress display, sparklines, and charts.
- Auto-calibration system with Quick, Full, and Adaptive modes for hardware-specific threshold tuning.
- Cross-validation between algorithms to verify result correctness.
- Dynamic threshold management for Parallel (4,096 bits), FFT (500,000 bits), and Strassen (3,072 bits) multiplication.
- Progress tracking with observer pattern and lock-free snapshots for hot loops.
- Shell completion generation for bash, zsh, fish, PowerShell, and elvish.
- Multiplication strategies: Karatsuba, Parallel Karatsuba, FFT-only, and Adaptive.
- FFT multiplication with Fermat Number Transform and transform cache.
- Memory arena allocation via bumpalo for FFT temporaries.
- BigInt memory pool for allocation reuse.
- Golden file test suite with known Fibonacci values.
- Property-based testing with proptest.
- Fuzz testing targets with cargo-fuzz.
- Criterion benchmarks for all algorithms.
- Optional GMP support via the rug crate (feature flag: `gmp`).
- Fast path optimization for n <= 93 using a precomputed u64 lookup table.
- ETA calculation for long-running computations.
- Configuration precedence: CLI flags > environment variables (`FIBCALC_*`) > adaptive calibration > static defaults.
- Cargo workspace with 7 crates: fibcalc, fibcalc-core, fibcalc-bigfft, fibcalc-orchestration, fibcalc-cli, fibcalc-tui, and fibcalc-calibration.
- Four-layer architecture: entry point, orchestration, core algorithms, and presentation.
- 96%+ test coverage across all crates (680+ tests).
- Cross-compilation support for x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc, x86_64-apple-darwin, and aarch64-apple-darwin.

[Unreleased]: https://github.com/agbruneau/FibRust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/agbruneau/FibRust/releases/tag/v0.1.0
