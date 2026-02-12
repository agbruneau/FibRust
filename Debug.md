## REVIEW INSTRUCTIONS

Perform a comprehensive review covering ALL of the following areas. For each area,
provide: (a) current state assessment, (b) specific issues found with code
references, (c) concrete fix with code examples, (d) explanation of the Rust idiom
or principle behind the fix.

### 1. ARCHITECTURE AND MODULE STRUCTURE

- Is the code organized into appropriate modules (lib.rs vs main.rs separation)?
- Best practice: computation logic in lib.rs (testable, reusable), CLI/main logic
  in main.rs. Check if this separation exists.
- Are public APIs well-defined with appropriate visibility (pub, pub(crate))?
- Does the project structure follow Cargo conventions (src/, tests/, benches/,
  examples/)?
- For an enterprise architect: suggest how to structure this as a library crate
  that others could depend on.

### 2. ALGORITHM ANALYSIS

Evaluate each Fibonacci algorithm implemented. Common approaches to check for:

| Algorithm             | Time     | Space          | Notes                                                  |
| --------------------- | -------- | -------------- | ------------------------------------------------------ |
| Naive recursive       | O(2^n)   | O(n) stack     | Exponential — only acceptable as a benchmark baseline |
| Memoized recursive    | O(n)     | O(n)           | HashMap vs Vec tradeoff                                |
| Iterative             | O(n)     | O(1)           | Standard, recommended for most cases                   |
| Matrix exponentiation | O(log n) | O(1)           | Uses [[1,1],[1,0]]^n identity                          |
| Fast doubling         | O(log n) | O(log n) stack | F(2n)=F(n)[2F(n+1)–F(n)], best for single large n     |
| Iterator trait        | O(n)     | O(1)           | Most idiomatic Rust for sequence generation            |

For each algorithm found:

- Verify correctness of base cases (F(0)=0, F(1)=1 convention vs F(0)=1, F(1)=1)
- Check for off-by-one errors
- Verify the algorithm matches the claimed complexity
- Suggest missing algorithms that would improve the project

### 3. INTEGER OVERFLOW HANDLING (CRITICAL)

This is the #1 issue in Fibonacci implementations. Check thoroughly:

- Does the code handle overflow? u32 overflows at F(47), u64 at F(93), u128 at
  F(186).
- In release builds, Rust wraps on overflow SILENTLY. Does Cargo.toml set
  `overflow-checks = true` in `[profile.release]`?
- Does the code use `checked_add()` returning `Option<T>`?
- Does the code use `overflowing_add()` or `saturating_add()`?
- Is `num-bigint::BigUint` used for arbitrary precision?
- Are overflow limits documented in function doc comments?
- Suggest this pattern if missing:

```rust
  pub fn fibonacci_checked(n: u32) -> Option<u128> {
      let (mut a, mut b): (u128, u128) = (0, 1);
      for _ in 0..n {
          let c = a.checked_add(b)?;
          a = b;
          b = c;
      }
      Some(a)
  }
```

### 4. RUST IDIOMS AND PATTERNS

Check for proper use of these Rust idioms:

- **Pattern matching**: `match n { 0 | 1 => ..., _ => ... }` vs if/else chains
- **Iterator implementation**: Does the project implement the `Iterator` trait for
  Fibonacci sequences? If not, suggest adding:

```rust
  pub struct Fibonacci { a: u64, b: u64 }
  impl Iterator for Fibonacci {
      type Item = u64;
      fn next(&mut self) -> Option<Self::Item> {
          let val = self.a;
          let next = self.a.checked_add(self.b)?;
          self.a = self.b;
          self.b = next;
          Some(val)
      }
  }
```

- **`std::iter::successors`**: Elegant one-liner approach:

```rust
  successors(Some((0u64, 1u64)), |&(a, b)| a.checked_add(b).map(|c| (b, c)))
```

- **`std::mem::replace`** / `std::mem::swap` for efficient value swapping
  (especially important with BigUint to avoid clones)
- **`#[must_use]`** attribute on pure computation functions
- **`const fn`** for compile-time computation of small Fibonacci numbers
- **Generics with num-traits**: Functions generic over `T: Zero + One + Add`
- **`impl Into<T>` or `From` traits** for flexible input types
- **Clippy patterns**: Watch for manual swap patterns, unnecessary mut,
  unnecessary clone, range patterns

### 5. ERROR HANDLING

- Are errors handled with `Result<T, E>` or `Option<T>` where appropriate?
- Does the code use `unwrap()` or `expect()` in library code? (Should not)
- Is there a custom error type? Suggest `thiserror` derive macro:

```rust
  #[derive(Debug, thiserror::Error)]
  pub enum FibError {
      #[error("overflow computing F({0}): result exceeds {1} capacity")]
      Overflow(u32, &'static str),
      #[error("invalid input: n must be non-negative")]
      InvalidInput,
  }
```

- For the main binary: Is `anyhow` used for application-level error handling?
- Are panics documented with `# Panics` sections in doc comments?

### 6. TESTING

Evaluate existing tests and suggest improvements:

**Unit tests** — check for:

- Base case tests: F(0), F(1), F(2)
- Known value tests: F(10)=55, F(20)=6765, F(50)=12586269025
- Edge cases: F(0), maximum safe input for each integer type
- Overflow behavior tests (should_panic or error result)
- Cross-algorithm consistency (all implementations agree)

**Property-based tests** — suggest if missing:

```rust
use proptest::prelude::*;
proptest! {
    #[test]
    fn additive_property(n in 2u32..40) {
        prop_assert_eq!(fibonacci(n), fibonacci(n-1) + fibonacci(n-2));
    }
    #[test]
    fn monotonically_increasing(n in 1u32..90) {
        prop_assert!(fibonacci(n) >= fibonacci(n-1));
    }
    #[test]
    fn all_implementations_agree(n in 0u32..40) {
        let iter_result = fibonacci_iterative(n);
        let recursive_result = fibonacci_recursive(n);
        prop_assert_eq!(iter_result, recursive_result);
    }
}
```

Add `proptest = "1"` to `[dev-dependencies]` if missing.

**Doc tests** — every public function should have `///` examples that compile:

```rust
/// Computes the nth Fibonacci number.
///
/// # Examples
/// ```
/// use fibrust::fibonacci;
/// assert_eq!(fibonacci(0), 0);
/// assert_eq!(fibonacci(10), 55);
/// ```
///
/// # Panics
/// Panics if the result overflows `u128` (n > 186).
```

### 7. BENCHMARKING

Check if benchmarks exist. If not, suggest adding with Criterion:

Cargo.toml additions:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "fibonacci_bench"
harness = false
```

Benchmark file (`benches/fibonacci_bench.rs`):

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fibonacci");
    for n in [10, 20, 30, 40, 50, 90].iter() {
        group.bench_with_input(BenchmarkId::new("Iterative", n), n, |b, &n| {
            b.iter(|| fibonacci_iterative(black_box(n)))
        });
        group.bench_with_input(BenchmarkId::new("Recursive", n), n, |b, &n| {
            b.iter(|| fibonacci_recursive(black_box(n)))
        });
        // Add more algorithms as implemented
    }
    group.finish();
}
criterion_group!(benches, bench_algorithms);
criterion_main!(benches);
```

### 8. CARGO.TOML CONFIGURATION

Review for:

- **Edition**: Should be `edition = "2021"` (or `"2024"` if using latest features)
- **rust-version**: Should specify MSRV (e.g., `rust-version = "1.70"`)
- **Dependencies**: Are they up to date? Are unnecessary dependencies included?
- **Lints section** — suggest adding:

```toml
  [lints.rust]
  unsafe_code = "forbid"

  [lints.clippy]
  all = { level = "deny" }
  pedantic = { level = "warn" }
  nursery = { level = "warn" }
```

- **Profile settings**:

```toml
  [profile.release]
  overflow-checks = true
  lto = true
  codegen-units = 1
```

- **Metadata**: description, license, repository URL fields populated?

### 9. DOCUMENTATION AND README

Review README for:

- Clear project description and motivation
- Build/run instructions (`cargo build`, `cargo run`, `cargo test`)
- Algorithm descriptions with complexity analysis
- Example output
- Benchmark results (if available)
- Comparison with the Go implementation (FibGo)
- License information

### 10. GO-TO-RUST ANTI-PATTERNS

Since the author comes from Go, watch for these common patterns that don't
translate well to Rust:

- **Returning error tuples** `(value, error)` instead of `Result<T, E>`
- **Excessive mutability** — Go's approach to state vs Rust's ownership model
- **Missing Iterator usage** — Go uses explicit loops; Rust should use iterators
- **String handling**: Go strings are UTF-8 but simpler; watch for unnecessary
  `.to_string()` / `.clone()` on `&str`
- **Missing type annotations** where Rust's type inference is more powerful
- **Ignoring pattern matching** — using if/else chains where match is idiomatic
- **Not using `Option`** — using sentinel values (0, -1) instead of `Option<T>`
- **Package/module confusion** — Go's package system vs Rust's module system

## OUTPUT FORMAT

Structure your review as follows in "DebugPlan.md"

1. **Executive Summary** (3-5 sentences): Overall quality assessment, strongest
   aspects, most critical issues.
2. **Critical Issues** (must-fix): Bugs, overflow vulnerabilities, correctness
   errors. Provide exact code fixes.
3. **Major Improvements** (should-fix): Architecture, missing tests, error
   handling gaps. Provide code examples.
4. **Minor Improvements** (nice-to-have): Style, additional algorithms, docs.
5. **Positive Observations**: What's done well — acknowledge good patterns.
6. **Recommended Action Plan**: Prioritized list of changes, ordered by impact.
7. **Learning Resources**: 2-3 specific Rust resources relevant to the patterns
   found in this code.

Be specific. Reference line numbers. Provide before/after code. Explain the
Rust principle behind every suggestion.
