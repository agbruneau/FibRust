# Troubleshooting Guide

This document covers common issues when building, running, and testing FibCalc-rs, along with their solutions.

## Table of Contents

- [Build Issues](#build-issues)
- [Runtime Issues](#runtime-issues)
- [TUI Issues](#tui-issues)
- [Testing Issues](#testing-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Error Messages](#error-messages)
- [Getting Help](#getting-help)

---

## Build Issues

### Rust version too old (MSRV not met)

**Problem:** Compilation fails with syntax errors or unknown feature messages.

**Cause:** FibCalc-rs requires Rust 1.80 or later (Rust 2021 edition).

**Solution:**

```bash
# Check your current Rust version
rustc --version

# Update to the latest stable toolchain
rustup update stable
```

If you manage multiple toolchains, ensure the project uses at least 1.80:

```bash
rustup override set stable
```

### Feature flag conflicts

**Problem:** Build fails when combining certain feature flags.

**Cause:** The `gmp` feature enables `rug` (GMP bindings) which has different system requirements than the default pure-Rust build.

**Solution:** Build with only one feature set at a time:

```bash
# Default pure-Rust build (no system dependencies)
cargo build --release

# GMP build (requires libgmp installed)
cargo build --release --features gmp
```

Do not combine `--features gmp` with `--features simd` unless both sets of system dependencies are satisfied.

### GMP / libgmp installation

**Problem:** Build fails with linker errors mentioning `gmp`, `libgmp`, or `rug` when using `--features gmp`.

**Cause:** The `rug` crate requires the GMP library (libgmp) to be installed on your system.

**Solution by platform:**

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install libgmp-dev
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install gmp-devel
```

**Linux (Arch):**
```bash
sudo pacman -S gmp
```

**macOS:**
```bash
brew install gmp
```

If the linker still cannot find GMP after installation on macOS, set the library path:
```bash
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:$CPATH"
```

**Windows:**
```bash
# Using vcpkg
vcpkg install gmp:x64-windows
# Set VCPKG_ROOT environment variable to your vcpkg installation
```

Alternatively on Windows, use the default pure-Rust build (no `--features gmp`) to avoid GMP entirely.

### Linker errors

**Problem:** Build fails with undefined symbol errors or linker failures.

**Solution:**

1. Ensure you have a working C toolchain installed:
   - **Linux:** `sudo apt-get install build-essential` (or equivalent)
   - **macOS:** `xcode-select --install`
   - **Windows:** Install Visual Studio Build Tools with the "Desktop development with C++" workload

2. Clean the build cache and rebuild:
   ```bash
   cargo clean
   cargo build --release
   ```

3. If linking fails for a specific dependency, check that system libraries are discoverable:
   ```bash
   # Linux: verify library paths
   ldconfig -p | grep gmp

   # macOS: verify Homebrew paths
   brew --prefix gmp
   ```

---

## Runtime Issues

### Out of memory for very large N

**Problem:** The program crashes or the OS kills the process when computing Fibonacci numbers with very large N (e.g., N > 100,000,000).

**Cause:** Large Fibonacci numbers require significant memory. F(100,000,000) has approximately 20.9 million digits.

**Solution:**

1. Use the `--memory-limit` flag to set a budget and get a clear error instead of a crash:
   ```bash
   fibcalc -n 100000000 --memory-limit 8G
   ```

2. Use `--last-digits` to compute only the trailing digits, which uses far less memory:
   ```bash
   fibcalc -n 100000000 --last-digits 1000 -c
   ```

3. Run only a single algorithm instead of all three to reduce peak memory:
   ```bash
   fibcalc -n 100000000 --algo fast
   ```

4. Monitor memory usage in TUI mode, which shows live memory metrics:
   ```bash
   fibcalc -n 100000000 --tui
   ```

### Slow performance

**Problem:** Computation takes much longer than expected for a given N.

**Solution:**

1. Run calibration to tune thresholds for your hardware:
   ```bash
   fibcalc --auto-calibrate
   ```

2. For a more thorough calibration:
   ```bash
   fibcalc --calibrate
   ```

3. Use release mode (debug builds are significantly slower):
   ```bash
   cargo run --release -p fibcalc -- -n 1000000 --algo fast -c
   ```

4. Check that thresholds are appropriate. The defaults are:
   - Parallel threshold: 4,096 bits
   - FFT threshold: 500,000 bits
   - Strassen threshold: 3,072 bits

   Override with CLI flags if needed:
   ```bash
   fibcalc -n 1000000 --threshold 8192 --fft-threshold 250000
   ```

5. For very large N, the `fast` (Fast Doubling) algorithm is generally the fastest. Try:
   ```bash
   fibcalc -n 50000000 --algo fast
   ```

### Calibration failures

**Problem:** `--calibrate` or `--auto-calibrate` produces unexpected results or errors.

**Solution:**

1. Close other CPU-intensive programs during calibration to reduce noise.

2. Delete any stale calibration profile and re-run:
   ```bash
   # Remove existing calibration file
   rm .fibcalc_calibration.json

   # Re-calibrate
   fibcalc --auto-calibrate
   ```

3. Calibration results are saved to `.fibcalc_calibration.json` in the current directory. If the file has bad permissions or is corrupted, delete it and try again.

### Cancellation not working

**Problem:** Pressing Ctrl+C does not stop the computation.

**Cause:** The cancellation mechanism uses a cooperative `CancellationToken`. The computation checks for cancellation at specific checkpoints during the algorithm. Very tight inner loops (e.g., FFT butterfly operations) may take a moment before reaching a cancellation checkpoint.

**Solution:**

1. Wait a few seconds after pressing Ctrl+C. The computation will stop at the next checkpoint.

2. If the process is truly stuck, force-kill it:
   - **Linux/macOS:** `kill -9 <pid>`
   - **Windows:** Use Task Manager or `taskkill /F /PID <pid>`

3. In TUI mode, press `q` or `Esc` to quit, or `Ctrl+C` to cancel the computation.

### Timeout not triggering

**Problem:** The computation does not stop after the configured timeout.

**Cause:** Similar to cancellation, timeouts are checked cooperatively. The default timeout is 5 minutes.

**Solution:**

Set a shorter timeout if needed:
```bash
fibcalc -n 100000000 --timeout 1m
```

Supported duration formats: `30s`, `5m`, `1h`, `500ms`.

---

## TUI Issues

### Terminal not supported

**Problem:** The TUI fails to start or shows garbled output.

**Cause:** The TUI requires a terminal that supports crossterm (ANSI escape sequences, alternate screen, raw mode).

**Solution:**

1. Use a modern terminal emulator:
   - **Windows:** Windows Terminal, PowerShell 7+, or ConEmu (avoid legacy `cmd.exe`)
   - **macOS:** Terminal.app, iTerm2, or Alacritty
   - **Linux:** Any modern terminal emulator (gnome-terminal, kitty, alacritty, etc.)

2. Ensure your terminal supports at least 80x24 characters.

3. If running over SSH, make sure your SSH client forwards terminal capabilities:
   ```bash
   ssh -t user@host "cd /path/to/project && fibcalc --tui"
   ```

### Screen rendering artifacts

**Problem:** Visual glitches, overlapping text, or corrupted display in TUI mode.

**Solution:**

1. Resize your terminal window. The TUI will redraw on resize events.

2. If artifacts persist, quit and restart the TUI:
   ```bash
   fibcalc --tui
   ```

3. Ensure your terminal font is monospaced and supports Unicode box-drawing characters.

4. Try a different terminal emulator if problems persist.

### Keyboard shortcuts not responding

**Problem:** Pressing keys in the TUI has no effect.

**Cause:** The TUI may not be in focus, or the terminal may be intercepting key events.

**Solution:**

The TUI keyboard shortcuts are:

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Ctrl+C` | Cancel computation |
| `p` | Pause |
| `r` | Resume |
| `d` | Toggle details panel |
| `l` | Toggle logs panel |
| Up / Down | Scroll logs |
| Page Up / Page Down | Scroll logs by page |
| Home / End | Jump to top/bottom of logs |

If keys are not working:

1. Ensure the TUI window has focus.
2. Check that your terminal is not capturing keys before they reach the application (e.g., tmux prefix key conflicts).
3. In tmux, you may need to press keys twice or adjust the tmux prefix.

### Raw mode cleanup on crash

**Problem:** After a crash or forced kill of the TUI, the terminal is left in a broken state (no visible input, garbled text, no line breaks).

**Cause:** The TUI enables raw mode and alternate screen on startup. If the process is killed before cleanup (`teardown_terminal`), the terminal remains in raw mode.

**Solution:**

Run the `reset` command to restore your terminal:

```bash
# Linux/macOS
reset

# Or, if that does not work:
stty sane
```

On Windows Terminal, closing and reopening the tab is usually sufficient. Alternatively:
```powershell
# PowerShell
[Console]::ResetColor()
```

---

## Testing Issues

### Golden test failures

**Problem:** `cargo test --test golden` fails with mismatched values.

**Cause:** Golden files contain reference Fibonacci values. Failures indicate either a code regression or stale golden files.

**Solution:**

1. First, check if the algorithm is producing correct results by running a known computation:
   ```bash
   cargo run --release -p fibcalc -- -n 100 --algo all -d
   ```

2. If you intentionally changed the output format or values, regenerate the golden files:
   ```bash
   # Check the golden file location
   ls tests/testdata/fibonacci_golden.json

   # Re-run tests with update flag (if using insta)
   cargo insta review
   ```

3. Verify that the golden file has not been accidentally modified:
   ```bash
   git diff tests/testdata/
   ```

### Property test flakes

**Problem:** Property-based tests (proptest) fail intermittently with different seeds.

**Solution:**

1. Re-run the failing test to see if it reproduces:
   ```bash
   cargo test <test_name> -- --nocapture
   ```

2. Check the proptest regression file for recorded failure cases:
   ```bash
   ls proptest-regressions/
   ```

3. If a proptest generates a regression file, keep it committed to prevent the same failure from recurring.

4. Increase the test timeout if tests fail due to slow generation:
   ```bash
   PROPTEST_MAX_SHRINK_TIME=60 cargo test
   ```

### Benchmark noise reduction

**Problem:** Criterion benchmarks show high variance or inconsistent results.

**Solution:**

1. Close all other applications to reduce system noise.

2. Disable CPU frequency scaling (Linux):
   ```bash
   sudo cpupower frequency-set -g performance
   ```

3. Run benchmarks with more samples:
   ```bash
   cargo bench -- --sample-size 100
   ```

4. Run a specific benchmark to isolate results:
   ```bash
   cargo bench -- "FastDoubling"
   ```

5. Use `cargo bench -- --save-baseline <name>` to compare across runs:
   ```bash
   cargo bench -- --save-baseline before
   # Make changes
   cargo bench -- --baseline before
   ```

### Coverage report generation

**Problem:** `cargo tarpaulin` fails or produces incomplete reports.

**Solution:**

1. Install the coverage tool:
   ```bash
   cargo install cargo-tarpaulin
   ```

   Or use `cargo-llvm-cov` (recommended):
   ```bash
   cargo install cargo-llvm-cov
   ```

2. Generate coverage:
   ```bash
   # With cargo-llvm-cov
   cargo llvm-cov --workspace --html

   # With tarpaulin
   cargo tarpaulin --out html
   ```

3. If tarpaulin fails on certain tests, exclude them:
   ```bash
   cargo tarpaulin --out html --exclude-files "*/tests/*"
   ```

4. On Windows, `cargo-llvm-cov` is generally more reliable than `cargo tarpaulin`.

---

## Platform-Specific Issues

### Windows

**MSVC Build Tools required**

**Problem:** Build fails with "linker `link.exe` not found" or similar.

**Solution:** Install Visual Studio Build Tools:
1. Download from the Visual Studio Downloads page.
2. Select the "Desktop development with C++" workload.
3. Restart your terminal after installation.

**Path handling**

**Problem:** File output paths fail or calibration file is not found.

**Solution:** Use forward slashes or escaped backslashes in paths:
```bash
fibcalc -n 1000 -c -o "C:/output/result.txt"
```

**Long path support**

**Problem:** Build fails with "path too long" errors in deeply nested dependency trees.

**Solution:** Enable long paths in Windows:
1. Run `regedit` and navigate to `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem`.
2. Set `LongPathsEnabled` to `1`.
3. Restart.

Or move your project closer to the drive root (e.g., `C:\projects\FibRust`).

### macOS

**Code signing (Apple Silicon)**

**Problem:** The binary is quarantined or blocked by Gatekeeper when distributed.

**Solution:** For local development this is not an issue. For distribution:
```bash
# Ad-hoc sign the binary
codesign -s - target/release/fibcalc
```

**Homebrew dependency paths**

**Problem:** Build cannot find libraries installed via Homebrew (especially on Apple Silicon).

**Solution:** Homebrew on Apple Silicon installs to `/opt/homebrew`. Set paths:
```bash
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:$CPATH"
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
```

Add these to your shell profile (`~/.zshrc`) for persistence.

### Linux

**Missing system libraries**

**Problem:** Build fails with missing library errors.

**Solution (Debian/Ubuntu):**
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config
# For GMP support:
sudo apt-get install libgmp-dev
```

**Solution (Fedora/RHEL):**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install pkgconfig
# For GMP support:
sudo dnf install gmp-devel
```

**Solution (Alpine/musl):**
```bash
apk add build-base
# For GMP support:
apk add gmp-dev
```

**Static linking with musl**

**Problem:** Cross-compiling for `x86_64-unknown-linux-musl` fails.

**Solution:**
```bash
# Install the musl target
rustup target add x86_64-unknown-linux-musl

# Install musl tools
sudo apt-get install musl-tools

# Build
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Error Messages

FibCalc-rs uses a structured error type (`FibError`) with the following variants and exit codes:

| Error | Exit Code | Meaning |
|-------|-----------|---------|
| `Calculation` | 1 | Generic calculation error |
| `Timeout` | 2 | Computation exceeded time limit |
| `Mismatch` | 3 | Cross-validation failure (algorithm results differ) |
| `Config` | 4 | Configuration error |
| `Cancelled` | 130 | User cancelled (Ctrl+C) |

### "calculation error: ..."

**Problem:** A generic calculation error occurred.

**Possible causes and solutions:**
- Thread pool creation failure: Reduce the concurrency level or check system thread limits.
- Internal algorithm error: File an issue with the full error message and N value.

### "configuration error: ..."

**Problem:** Invalid configuration was provided.

**Possible causes and solutions:**
- Unknown algorithm name: Use `fast`, `matrix`, `fft`, or `all`.
- Invalid threshold value: Ensure threshold values are positive integers.
- Invalid memory limit format: Use formats like `512M`, `4G`, `8G`.
- Invalid timeout format: Use formats like `30s`, `5m`, `1h`.

### "calculation timed out after ..."

**Problem:** The computation exceeded the timeout duration (default: 5 minutes).

**Solution:**

1. Increase the timeout:
   ```bash
   fibcalc -n 100000000 --timeout 30m
   ```

2. Use a faster algorithm for your N value:
   ```bash
   fibcalc -n 100000000 --algo fast --timeout 10m
   ```

3. Compute only the last K digits to reduce computation time:
   ```bash
   fibcalc -n 100000000 --last-digits 100 --timeout 5m -c
   ```

### "result mismatch between algorithms"

**Problem:** When running `--algo all`, the results from different algorithms do not match.

**Cause:** This indicates a bug in one of the algorithm implementations, or memory corruption.

**Solution:**

1. Run each algorithm individually to identify which one produces incorrect results:
   ```bash
   fibcalc -n <value> --algo fast -c -o fast_result.txt
   fibcalc -n <value> --algo matrix -c -o matrix_result.txt
   fibcalc -n <value> --algo fft -c -o fft_result.txt
   ```

2. Compare outputs:
   ```bash
   diff fast_result.txt matrix_result.txt
   ```

3. Verify against known values using the golden test data in `tests/testdata/fibonacci_golden.json`.

4. File an issue with the N value, algorithm outputs, and your platform details.

### "calculation cancelled"

**Problem:** Computation was stopped by user (Ctrl+C) or programmatic cancellation.

**Solution:** This is expected behavior when you press Ctrl+C. If cancellation happens unexpectedly, check for:
- Other processes sending SIGINT to the fibcalc process.
- A very short timeout that triggers before computation completes.

### "Estimated memory (X MB) exceeds limit (Y MB)"

**Problem:** The pre-flight memory estimate exceeds the configured `--memory-limit`.

**Solution:**

1. Increase the memory limit:
   ```bash
   fibcalc -n 100000000 --memory-limit 16G
   ```

2. Remove the memory limit entirely (unlimited):
   ```bash
   fibcalc -n 100000000
   ```

3. Reduce N or use `--last-digits` to lower memory requirements.

---

## Getting Help

### Enable debug logging

FibCalc-rs uses the `tracing` framework for structured logging. Set the `RUST_LOG` environment variable to increase verbosity:

```bash
# Show warnings and errors (default)
RUST_LOG=warn fibcalc -n 1000 --algo fast -c

# Show info-level messages
RUST_LOG=info fibcalc -n 1000 --algo fast -c

# Show debug-level messages
RUST_LOG=debug fibcalc -n 1000 --algo fast -c

# Show trace-level messages (very verbose)
RUST_LOG=trace fibcalc -n 1000 --algo fast -c

# Filter by crate
RUST_LOG=fibcalc_core=debug fibcalc -n 1000 --algo fast -c
```

On Windows PowerShell:
```powershell
$env:RUST_LOG="debug"; fibcalc -n 1000 --algo fast -c
```

### Verbose mode

Use the `-v` (verbose) flag for additional runtime information:

```bash
fibcalc -n 1000 --algo all -v -d
```

The `-d` (details) flag adds timing and algorithm comparison details.

### Diagnosing build issues

```bash
# Check Rust toolchain information
rustup show
rustc --version
cargo --version

# Verify all dependencies resolve
cargo check

# Strict lint check
cargo clippy -- -W clippy::pedantic

# Security audit
cargo audit

# License compatibility
cargo deny check
```

### Filing an issue

When reporting a bug, include:

1. **Rust version:** Output of `rustc --version`.
2. **Operating system:** OS name, version, and architecture.
3. **Full command:** The exact command you ran.
4. **Full error output:** Copy-paste the complete error message.
5. **N value and algorithm:** Which N and algorithm triggered the issue.
6. **Feature flags:** Whether you used `--features gmp` or other flags.
7. **Debug log:** Output with `RUST_LOG=debug` if applicable.

File issues at the project's GitHub issue tracker.
