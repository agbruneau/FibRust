# Contributing to FibCalc-rs

Thank you for your interest in contributing to FibCalc-rs! This document provides guidelines and instructions for contributing to the project.

FibCalc-rs is a high-performance Fibonacci calculator written in Rust, implementing three algorithms (Fast Doubling, Matrix Exponentiation, FFT-Based) with CLI and interactive TUI modes. Whether you are fixing a bug, improving performance, adding a new algorithm, or improving documentation, your contribution is welcome.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing Requirements](#testing-requirements)
- [Adding a New Algorithm](#adding-a-new-algorithm)
- [Adding a Multiplication Strategy](#adding-a-multiplication-strategy)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Code Review Expectations](#code-review-expectations)
- [Safety and `unsafe` Policy](#safety-and-unsafe-policy)
- [Licensing](#licensing)

## Prerequisites

Before you begin, make sure you have the following installed:

- **Rust 1.80+** (MSRV). Install via [rustup](https://rustup.rs/).
- **Cargo** (comes with rustup).
- **Git** for version control.
- (Optional) **libgmp** if you plan to work on the `gmp` feature.
- (Optional) **cargo-tarpaulin** or **cargo-llvm-cov** for code coverage measurement.
- (Optional) **cargo-deny** for license/security policy checks.
- (Optional) **cargo-audit** for dependency security audits.

Verify your setup:

```bash
rustc --version   # Should be 1.80.0 or later
cargo --version
git --version
```

## Development Setup

1. **Fork and clone** the repository:

   ```bash
   git clone https://github.com/agbruneau/FibRust.git
   cd FibRust
   ```

2. **Build** the project:

   ```bash
   cargo build --release
   ```

3. **Run the tests** to make sure everything works:

   ```bash
   cargo test
   ```

4. **Run linting** to check for issues:

   ```bash
   cargo clippy -- -W clippy::pedantic
   cargo fmt --check
   ```

5. **Run the binary** to explore the tool:

   ```bash
   cargo run --release -p fibcalc -- -n 1000 --algo fast -c
   ```

### Workspace Structure

FibCalc-rs is a Cargo workspace with 7 crates. Understanding the architecture helps you find the right place for your change:

| Crate | Type | Role |
|-------|------|------|
| `fibcalc` | bin | Binary entry point, app config, error handling |
| `fibcalc-core` | lib | Fibonacci algorithms, strategies, observers, dynamic thresholds |
| `fibcalc-bigfft` | lib | FFT multiplication, Fermat numbers, transform cache, bump allocator |
| `fibcalc-orchestration` | lib | Parallel execution, calculator selection, result analysis |
| `fibcalc-cli` | lib | CLI output, progress bars, ETA, shell completion |
| `fibcalc-tui` | lib | Interactive TUI dashboard (ratatui + crossterm) |
| `fibcalc-calibration` | lib | Auto-tuning, adaptive benchmarks, calibration profiles |

The layered architecture flows as:

```
Entry point (src/main.rs)
    |
Orchestration (fibcalc-orchestration)
    |
Core (fibcalc-core, fibcalc-bigfft)
    |
Presentation (fibcalc-cli, fibcalc-tui)
```

## Code Style

### Formatting

All code must pass `cargo fmt --check`. Run `cargo fmt` before committing to auto-format your code.

### Linting

All code must pass strict Clippy linting with zero warnings:

```bash
cargo clippy -- -W clippy::pedantic
```

The workspace `Cargo.toml` configures allowed Clippy exceptions centrally. If you believe a new exception is warranted, discuss it in the PR.

### Import Ordering

Group imports in the following order, separated by blank lines:

1. Standard library (`std::*`)
2. External crates (`num_bigint`, `rayon`, etc.)
3. Workspace crates (`fibcalc_core`, `fibcalc_bigfft`, etc.)

Example:

```rust
use std::collections::HashMap;
use std::sync::Arc;

use num_bigint::BigUint;
use parking_lot::RwLock;

use crate::calculator::{Calculator, FibCalculator};
```

### Error Handling

- Use `thiserror` for library error types.
- Use `anyhow` only in the binary crate (`fibcalc`).
- Use the `FibError` enum for calculation errors, with variants: `Calculation`, `Config`, `Cancelled`, `Timeout`, `Mismatch`.

### Cyclomatic Complexity

Keep functions under a cyclomatic complexity of **15**. If a function grows too complex, refactor it into smaller, well-named helper functions.

### Concurrency

- Use `rayon` for CPU-bound parallelism.
- Use `crossbeam::channel` for inter-thread communication.
- Do **not** use `tokio` for computation. The workload is CPU-bound and synchronous.

## Testing Requirements

We maintain high test quality. The current test suite has **680+ tests** with **96.1% line coverage**.

### Running Tests

```bash
cargo test                          # All tests
cargo test --lib                    # Unit tests only
cargo test --test golden            # Golden file tests
cargo test --test e2e               # End-to-end tests
cargo test -p fibcalc-core          # Tests for a specific crate
```

### Coverage Target

The project target is **>75% line coverage**, and the current codebase maintains **96%+**. New code should come with tests that maintain or improve this coverage level.

Measure coverage with:

```bash
# Using cargo-llvm-cov (recommended)
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --html   # HTML report in target/llvm-cov/html/

# Using cargo-tarpaulin
cargo tarpaulin --out html
```

### Types of Tests

1. **Unit tests**: Table-driven `#[test]` functions co-located with the code under `#[cfg(test)] mod tests`.

2. **Golden file tests** (`tests/golden.rs`): Verify all three algorithms produce correct results against known Fibonacci values stored in `tests/testdata/fibonacci_golden.json`. If you add a new algorithm, add it to the golden test suite.

3. **Property-based tests**: Use `proptest` to verify invariants hold across random inputs. Especially useful for mathematical properties like F(a+b) = F(a)*F(b+1) + F(a-1)*F(b).

4. **Fuzz tests** (`fuzz/fuzz_targets/`): Fuzz testing for robustness against unexpected inputs.

   ```bash
   cargo fuzz run fuzz_fast_doubling -- -max_total_time=30
   ```

5. **End-to-end tests** (`tests/e2e.rs`): Test the compiled binary with `assert_cmd` to verify CLI behavior.

6. **Benchmarks**: Use `criterion` for performance benchmarks.

   ```bash
   cargo bench                      # All benchmarks
   cargo bench -- "FastDoubling"    # Specific benchmark
   ```

### Adding Tests

When contributing new functionality:

- Add unit tests for all public functions.
- Add golden file entries for new algorithms (update `tests/testdata/fibonacci_golden.json`).
- Add property-based tests for mathematical invariants.
- Include edge case tests for boundary values (n=0, n=1, n=93, n=94).
- Test cancellation behavior for long-running computations.

## Adding a New Algorithm

To add a new Fibonacci algorithm, follow these steps:

### 1. Implement `CoreCalculator`

Create a new file in `crates/fibcalc-core/src/` (e.g., `myalgo.rs`) and implement the `CoreCalculator` trait:

```rust
use num_bigint::BigUint;

use crate::calculator::{CoreCalculator, FibError};
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::CancellationToken;

pub struct MyAlgorithm;

impl MyAlgorithm {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl CoreCalculator for MyAlgorithm {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError> {
        // Your algorithm implementation here.
        // Check cancel.is_cancelled() periodically in loops.
        // Report progress via observer.on_progress().
        todo!()
    }

    fn name(&self) -> &str {
        "MyAlgorithm"
    }
}
```

Key requirements:
- Check `cancel.is_cancelled()` periodically in your main loop and return `Err(FibError::Cancelled)` if true.
- Report progress updates via the `observer` parameter.
- Handle edge cases: n=0 returns 0, n=1 and n=2 return 1.

### 2. Register in the Factory

Add your algorithm to `crates/fibcalc-core/src/registry.rs` in the `DefaultFactory::create_calculator` method:

```rust
fn create_calculator(name: &str) -> Result<Arc<dyn Calculator>, FibError> {
    match name {
        "fast" | "fastdoubling" => { /* ... */ }
        "matrix" => { /* ... */ }
        "fft" => { /* ... */ }
        "myalgo" => {
            let core = Arc::new(MyAlgorithm::new());
            Ok(Arc::new(FibCalculator::new(core)))
        }
        _ => Err(FibError::Config(format!("unknown calculator: {name}"))),
    }
}
```

Also update the `available()` method to include your algorithm name.

### 3. Export the Module

Add `pub mod myalgo;` to `crates/fibcalc-core/src/lib.rs`.

### 4. Add Tests

- Add unit tests in your algorithm file.
- Add your algorithm to the golden tests in `tests/golden.rs`.
- Verify cross-algorithm agreement (your algorithm must produce the same results as existing algorithms for all test values).
- Add a criterion benchmark in `benches/`.

## Adding a Multiplication Strategy

The multiplication system uses the Strategy pattern with the `Multiplier` trait (narrow interface) and the optional `DoublingStepExecutor` trait (extended interface).

### 1. Implement `Multiplier`

Create your strategy in `crates/fibcalc-core/src/strategy.rs` or a new file:

```rust
use num_bigint::BigUint;

use crate::strategy::Multiplier;

pub struct MyMultiplier;

impl Multiplier for MyMultiplier {
    fn multiply(&self, a: &BigUint, b: &BigUint) -> BigUint {
        // Your multiplication implementation
        todo!()
    }

    fn square(&self, a: &BigUint) -> BigUint {
        // Optional: optimized squaring. Falls back to multiply(a, a) by default.
        todo!()
    }

    fn name(&self) -> &str {
        "MyMultiplier"
    }
}
```

### 2. Optionally Implement `DoublingStepExecutor`

If your strategy can optimize the complete Fast Doubling step (computing F(2k) and F(2k+1) together), implement `DoublingStepExecutor`:

```rust
use crate::strategy::DoublingStepExecutor;

impl DoublingStepExecutor for MyMultiplier {
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        // F(2k) = F(k) * (2*F(k+1) - F(k))
        // F(2k+1) = F(k)^2 + F(k+1)^2
        todo!()
    }
}
```

### 3. Add Tests

Add tests verifying that your strategy produces the same results as `KaratsubaStrategy` for a range of inputs. See the `all_strategies_agree_on_doubling` test for a pattern to follow.

## Pull Request Process

### Branch Naming

Use descriptive branch names with a prefix:

- `feat/short-description` -- new features
- `fix/short-description` -- bug fixes
- `docs/short-description` -- documentation
- `refactor/short-description` -- code refactoring
- `perf/short-description` -- performance improvements
- `test/short-description` -- test additions/fixes
- `chore/short-description` -- tooling, CI, dependencies

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/). Every commit message must follow this format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.

Examples:

```
feat(core): add Lucas sequence algorithm
fix(bigfft): correct overflow in Fermat transform for N > 2^24
perf(strategy): parallelize squaring in AdaptiveStrategy
docs: update CONTRIBUTING.md with new test guidelines
test(golden): add golden entries for n=50000
refactor(orchestration): extract result aggregation into separate module
chore: update num-bigint to 0.5
```

### Before Submitting

Run the full quality check suite:

```bash
cargo fmt --check                    # Formatting
cargo clippy -- -W clippy::pedantic  # Linting
cargo test                           # All tests
cargo deny check                     # License compliance
cargo audit                          # Security audit
```

### PR Requirements

1. All CI checks must pass (formatting, linting, tests, security).
2. New code must include tests. Coverage should not decrease.
3. The PR description should explain *what* changed and *why*.
4. Keep PRs focused -- one logical change per PR.
5. Update documentation if your change affects public APIs, configuration, or usage.

## Issue Reporting

When reporting a bug or requesting a feature, please include:

### Bug Reports

- **Rust version** (`rustc --version`).
- **Operating system** and version.
- **Steps to reproduce** the issue, including the exact command you ran.
- **Expected behavior** vs. **actual behavior**.
- **Error messages** or panic output, if any.
- The **Fibonacci index** (n) that triggered the issue, if applicable.

### Feature Requests

- A clear description of the proposed feature.
- The **use case** or motivation for the feature.
- Any relevant references (papers, other implementations).

## Code Review Expectations

During code review, we look for:

1. **Correctness**: The code does what it claims. Mathematical algorithms must be verified against golden values.
2. **Tests**: New functionality has comprehensive tests. Edge cases are covered.
3. **Performance**: No unnecessary allocations or copies. Use `std::mem::take` / `std::mem::replace` for zero-copy patterns where appropriate.
4. **Style**: Code follows formatting and linting rules. Imports are ordered correctly.
5. **Complexity**: Functions stay under cyclomatic complexity 15. Complex logic is broken into well-named helpers.
6. **Safety**: No new `unsafe` without justification (see below). No security vulnerabilities.
7. **Documentation**: Public items have doc comments. Non-obvious logic has inline comments.

Be receptive to feedback and willing to iterate. Code review is a collaborative process.

## Safety and `unsafe` Policy

The project has a strict policy on `unsafe` code:

- **Maximum 5 `unsafe` blocks** across the entire codebase.
- Every `unsafe` block **must** have a `// SAFETY:` comment explaining why the code is sound.
- New `unsafe` usage requires explicit justification in the PR description.
- Run `cargo geiger` to audit unsafe usage before submitting.

Prefer safe abstractions. If you think `unsafe` is necessary for performance, benchmark the safe alternative first and include the comparison in your PR.

## Licensing

FibCalc-rs is licensed under **Apache-2.0**. By contributing, you agree that your contributions will be licensed under the same terms.

### GMP Feature and LGPL

The optional `gmp` feature links against libgmp via the `rug` crate, which is licensed under **LGPL**. When the `gmp` feature is enabled, the combined work must comply with LGPL terms. Keep this in mind if your contribution touches the `gmp` feature path.

### Dependency Licenses

All dependencies must have licenses compatible with the project policy defined in `deny.toml`. Allowed licenses include: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, BSL-1.0, CC0-1.0, MPL-2.0, and Unicode licenses. Copyleft licenses trigger a warning and require discussion.

Before adding a new dependency, run:

```bash
cargo deny check
```

---

Thank you for taking the time to contribute to FibCalc-rs. If you have questions, feel free to open an issue or start a discussion.
