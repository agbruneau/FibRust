# FibRust 🦀

A high-performance, parallel Fibonacci calculator written in Rust. FibRust implements state-of-the-art algorithms to compute massive Fibonacci numbers (millions of digits) in milliseconds, leveraging multi-core parallelism and Number Theoretic Transforms.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## 🚀 Key Features

- **Extreme Performance**: Computes $F(10,000,000)$ in **~75ms** (parallel) and $F(100,000,000)$ in **~1.2s**.
- **Robust & Crash-Free**: Built on `ibig` for efficient, arbitrary-precision integers that never overflow.
- **Adaptive Algorithm**: Automatically selects the best algorithm based on input size.
- **Advanced Algorithms**:
  - **Fast Doubling**: Optimized iterative O(log n) implementation.
  - **Parallel Fast Doubling**: Parallelized O(log n) for large integers using Rayon.
  - **FFT-Based Multiplication**: Custom Number Theoretic Transform for huge numbers (millions of bits).
- **Intelligent Execution**:
  - **Auto-Calibration**: Dynamically adjusts parallelization thresholds based on CPU speed and core count.
  - **Zero-Allocation Iterators**: Lazy range generation for memory efficiency.
  - **Optimized Math**: Uses bitwise operations instead of expensive divisions where possible.
  - **Explicit Error Handling**: Validates input size against system limits to prevent crashes on massive inputs.
- **Interactive TUI**: HTOP-inspired terminal interface with real-time progress tracking and algorithm comparison.

## 📦 Project Structure

FibRust is organized as a **Cargo workspace** with 3 crates:

```
FibRust/
├── crates/
│   ├── fibrust-core/       # Core algorithms (lightweight, no heavy deps)
│   ├── fibrust-server/     # HTTP API server (Axum)
│   └── fibrust-cli/        # CLI binary
```

| Crate            | Description            | Dependencies                          |
| ---------------- | ---------------------- | ------------------------------------- |
| `fibrust-core`   | Algorithms & iterators | `ibig`, `rustfft`, `rayon`            |
| `fibrust-server` | HTTP API               | `axum`, `tokio`                       |
| `fibrust-cli`    | CLI + TUI interface    | `clap`, `indicatif`, `ratatui`, `crossterm` |

## 📦 Installation

```bash
git clone https://github.com/agbruneau/FibRust.git
cd FibRust
cargo build --workspace --release
```

> [!TIP]
> The release profile uses **LTO (Link-Time Optimization)** and `panic = "abort"` for maximum performance.

## 🛠 Usage

### CLI (`fibrust`)

```bash
# Calculate F(n)
cargo run -p fibrust-cli --release -- 1000000

# Adaptive algorithm (default) - auto-selects Fast Doubling or FFT
cargo run -p fibrust-cli --release -- 10000000

# Compare all algorithms
cargo run -p fibrust-cli --release -- 10000000 -a all

# Calculate a range of Fibonacci numbers
cargo run -p fibrust-cli --release -- range 100 200

# Show detailed analysis
cargo run -p fibrust-cli --release -- 1000000 --detail

# Launch interactive TUI mode
cargo run -p fibrust-cli --release -- --tui
```

### Interactive TUI Mode

Launch the HTOP-inspired terminal interface for interactive Fibonacci calculations:

```bash
cargo run -p fibrust-cli --release -- --tui
```

The TUI provides:
- **Real-time progress tracking** with progress bar and ETA
- **Algorithm comparison** with speedup metrics
- **Result analysis** with digit count and scientific notation
- **Keyboard navigation** for quick input and algorithm selection

#### TUI Keyboard Shortcuts

| Key            | Action                        |
| -------------- | ----------------------------- |
| `q` / `Esc`    | Quit application              |
| `r` / `Enter`  | Run calculation               |
| `?` / `F1`     | Toggle help overlay           |
| `Tab`          | Switch between input fields   |
| `n`            | Focus n input field           |
| `a`            | Focus algorithm selector      |
| `Up` / `Down`  | Navigate algorithm options    |
| `1-5`          | Direct algorithm selection    |
| `Backspace`    | Delete last digit             |

### HTTP Server (`fibrust-server`)

```bash
# Start the API server
cargo run -p fibrust-server --release -- --port 3000

# API endpoint: GET /fib/{n}?algo=adaptive|fd|par|fft
# Example: curl http://localhost:3000/fib/1000
```

### Library Usage (`fibrust-core`)

Add to your `Cargo.toml`:

```toml
[dependencies]
fibrust-core = { path = "crates/fibrust-core" }
```

```rust
use fibrust_core::{fibonacci_adaptive, fibonacci_fast_doubling, FibRange};

// Adaptive (recommended) - auto-selects best algorithm
let f = fibonacci_adaptive(1_000_000);

// Fast Doubling for smaller values
let f = fibonacci_fast_doubling(10_000);

// Lazy range iteration
for f_n in FibRange::new(1000, 2000) {
    println!("{}", f_n);
}
```

### CLI Options

| Option                  | Description                                                     |
| ----------------------- | --------------------------------------------------------------- |
| `<n>` or `--n <n>`      | Calculate F(n)                                                  |
| `-a, --algorithm <alg>` | `adaptive` (default), `fast-doubling`, `parallel`, `fft`, `all` |
| `-d, --detail`          | Show detailed result analysis                                   |
| `-s, --seq`             | Force sequential execution                                      |
| `--tui`                 | Launch interactive TUI mode                                     |
| `range <start> <end>`   | Generate F(start)..F(end)                                       |

## 📊 Benchmarks

_Results on 24-core Ryzen 9 (Windows)._

| Index (n) | Bits  | Fast Doubling | Parallel | FFT        |
| --------- | ----- | ------------- | -------- | ---------- |
| 100K      | ~69K  | 0.9 ms        | 2.1 ms   | 1.5 ms     |
| 1M        | ~694K | 11 ms         | 26 ms    | 15 ms      |
| 10M       | ~6.9M | 240 ms        | 86 ms    | **64 ms**  |
| 100M      | ~69M  | 7.13 s        | 4.77 s   | **1.15 s** |

### Scalability

The **Parallel Fast Doubling** and **FFT** algorithms leverage the `rayon` thread pool to scale across available CPU cores.

- **Parallel Fast Doubling**: Scales up to ~8-16 threads for $N > 10^5$, limited by synchronization overhead.
- **FFT**: Highly scalable for massive inputs ($N > 10^6$), effectively utilizing all available cores for large integer multiplications.

> [!NOTE]
> The **adaptive** algorithm automatically selects the best method. Thresholds are chosen based on empirical benchmarks:
>
> - **$n < 40,000$: Fast Doubling**
>   - Sequential execution avoids thread pool overhead which dominates for small inputs.
> - **$40,000 \le n < 200,000$: Parallel Fast Doubling**
>   - The cost of multiplication becomes significant enough that splitting the work across cores provides a net speedup.
> - **$n \ge 200,000$: FFT**
>   - The $O(n \log n)$ asymptotic complexity of FFT-based multiplication outperforms the $O(n^{1.585})$ Karatsuba algorithm used in `ibig` for massive integers.

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace --release

# Run core library tests
cargo test -p fibrust-core --release

# Run CLI and TUI tests
cargo test -p fibrust-cli --release
```

## 🧠 Algorithms

### Complexity Analysis

| Algorithm | Time Complexity | Space Complexity | Description |
|-----------|----------------|------------------|-------------|
| **Fast Doubling** | $O(\log n \cdot M(n))$ | $O(n)$ | Standard iterative approach. $M(n)$ is the complexity of multiplication, roughly $O(n^{1.585})$ with Karatsuba. |
| **Parallel Fast Doubling** | $O(\log n \cdot M(n) / p)$ | $O(n)$ | Parallelizes the 3 multiplications in the recursive step across $p$ cores. |
| **FFT-Based** | $O(n \log n)$ | $O(n)$ | Uses Schönhage-Strassen algorithm (via `rustfft`) for multiplication. Asymptotically optimal for huge $n$. |

### Details

1. **Fast Doubling**: Uses $F(2k) = F(k)(2F(k+1) - F(k))$ and $F(2k+1) = F(k)^2 + F(k+1)^2$.
2. **Parallel Fast Doubling**: Same algorithm as above, but with parallelized multiplications using Rayon.
3. **FFT-Based**: Multiplication in frequency domain. Optimized with bitwise operations instead of expensive divisions.

## 📄 License

MIT © 2024
