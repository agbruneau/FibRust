# Algorithms

Comprehensive documentation of the Fibonacci algorithms implemented in FibCalc-rs, including their mathematical foundations, implementation details, complexity analysis, and selection criteria.

## Table of Contents

1. [Mathematical Foundation](#mathematical-foundation)
2. [Fast Doubling](#fast-doubling)
3. [Matrix Exponentiation](#matrix-exponentiation)
4. [FFT-Based Calculator](#fft-based-calculator)
5. [Multiplication Strategies](#multiplication-strategies)
6. [Complexity Analysis](#complexity-analysis)
7. [Dynamic Thresholds](#dynamic-thresholds)
8. [Algorithm Selection Flowchart](#algorithm-selection-flowchart)
9. [Cross-Validation Mechanism](#cross-validation-mechanism)
10. [Fast Path: Precomputed Lookup Table](#fast-path-precomputed-lookup-table)

---

## Mathematical Foundation

The Fibonacci sequence is defined by the recurrence relation:

```
F(0) = 0
F(1) = 1
F(n) = F(n-1) + F(n-2)   for n >= 2
```

The naive recursive computation has exponential time complexity O(phi^n), where phi = (1 + sqrt(5)) / 2 is the golden ratio. The iterative approach runs in O(n) time, but for very large n (millions or billions), even O(n) is too slow because the size of F(n) itself grows linearly in n -- F(n) has approximately n * log10(phi) ~ 0.209n decimal digits.

FibCalc-rs implements three O(log n) algorithms that exploit algebraic identities to halve the problem size at each step. The key insight is that Fibonacci numbers satisfy matrix and doubling identities that allow computing F(n) in O(log n) *steps*, where each step involves big-number arithmetic on increasingly large operands.

### Key Identity: The Q-Matrix

The Fibonacci recurrence can be expressed as a matrix equation:

```
| F(n+1) |   | 1  1 |^n   | F(1) |   | 1  1 |^n   | 1 |
|        | = |      |   *  |      | = |      |   *  |   |
| F(n)   |   | 1  0 |     | F(0) |   | 1  0 |     | 0 |
```

Equivalently, defining Q = [[1,1],[1,0]]:

```
Q^n = | F(n+1)  F(n)   |
      | F(n)    F(n-1) |
```

This yields F(n) = Q^n[0][1] = Q^n[1][0].

### Key Identity: Fast Doubling

From the Q-matrix identity, the following doubling formulas are derived:

```
F(2k)   = F(k) * [2*F(k+1) - F(k)]
F(2k+1) = F(k+1)^2 + F(k)^2
```

These allow computing F(n) from F(n/2) and F(n/2+1), halving the problem at each step.

---

## Fast Doubling

**Source**: `crates/fibcalc-core/src/fastdoubling.rs` -- `OptimizedFastDoubling`

### Algorithm Description

Fast Doubling is the primary and fastest algorithm in FibCalc-rs. It computes F(n) in O(log n) doubling steps by scanning the binary representation of n from the most significant bit (MSB) to the least significant bit (LSB).

### Derivation

Starting from the matrix identity Q^n = [[F(n+1), F(n)], [F(n), F(n-1)]], we can derive:

```
Q^(2k) = (Q^k)^2
```

Expanding the matrix square and extracting the (0,1) and (1,1) entries:

```
F(2k)   = F(k) * [2*F(k+1) - F(k)]        (1 multiplication)
F(2k+1) = F(k)^2 + F(k+1)^2               (2 squarings)
```

### Bit-Scanning Loop

The implementation processes the bits of n from MSB to LSB. Starting with (F(0), F(1)) = (0, 1):

1. **Doubling step**: Apply the doubling identities to transform (F(k), F(k+1)) into (F(2k), F(2k+1))
2. **Conditional addition**: If the current bit of n is 1, advance one position: (F(2k), F(2k+1)) becomes (F(2k+1), F(2k) + F(2k+1)) = (F(2k+1), F(2k+2))

After processing all bits, the state holds (F(n), F(n+1)).

### Implementation Details

```
for each bit i from (MSB-1) down to 0:
    (fk, fk1) = doubling_step(fk, fk1)     // F(2k), F(2k+1)
    if bit i of n is set:
        (fk, fk1) = (fk1, fk + fk1)        // advance by 1
return fk
```

Key implementation features:

- **Thread-local state pooling** (`CalculationState`): Reuses pre-allocated `BigUint` temporaries across calls to avoid repeated heap allocation. A thread-local pool with a maximum size of 4 states is maintained via `RefCell<Vec<CalculationState>>`.
- **Zero-copy result extraction**: Uses `std::mem::take` to extract the final result without cloning.
- **Pointer rotation**: Uses `std::mem::replace` for the conditional addition step, avoiding unnecessary copies.
- **Progress reporting**: Reports progress via `FrozenObserver` snapshots to avoid lock contention in hot loops. Progress is reported only when the change exceeds 1%.
- **Cancellation support**: Checks the `CancellationToken` at each bit iteration, allowing graceful interruption of long-running computations.

### State Structure

The `CalculationState` struct holds all temporaries needed for one computation:

| Field | Purpose |
|-------|---------|
| `fk`  | Current F(k) |
| `fk1` | Current F(k+1) |
| `t1`  | Temporary for doubling |
| `t2`  | Temporary for doubling |
| `t3`  | Temporary for doubling |

States are acquired from and returned to the thread-local pool before and after each computation.

---

## Matrix Exponentiation

**Source**: `crates/fibcalc-core/src/matrix.rs` -- `MatrixExponentiation`
**Types**: `crates/fibcalc-core/src/matrix_types.rs` -- `Matrix`, `MatrixState`
**Operations**: `crates/fibcalc-core/src/matrix_ops.rs` -- `matrix_multiply`, `matrix_square`

### Algorithm Description

Matrix Exponentiation computes F(n) by raising the Q-matrix to the n-th power using binary exponentiation (repeated squaring). The result F(n) is extracted from Q^n[0][1].

### Binary Exponentiation

```
result = I (identity matrix)
base = Q = [[1,1],[1,0]]

for each bit i from (MSB-1) down to 0:
    result = result^2                       // square
    if bit i of n is set:
        result = result * base              // multiply by Q
return result.b                             // F(n) = Q^n[0][1]
```

### Symmetric Matrix Optimization

A critical optimization exploits the fact that all powers of the Fibonacci Q-matrix are **symmetric** (i.e., element [0][1] equals element [1][0]). This holds because Q itself is symmetric and the product of two symmetric matrices of this specific form remains symmetric.

For a symmetric 2x2 matrix [[a,b],[b,d]]:

**Squaring** (3 multiplications instead of 8):
```
a' = a*a + b*b
b' = b*(a + d)
d' = b*b + d*d
```

**Multiplication** of two symmetric matrices (5 multiplications instead of 8):
```
a' = a1*a2 + b1*b2
b' = a1*b2 + b1*d2
d' = b1*b2 + d1*d2
```

These optimizations are implemented in `Matrix::square_symmetric()` and `Matrix::multiply_symmetric()`, called via `matrix_square()` and `matrix_multiply()` respectively.

### State Structure

The `MatrixState` holds:

| Field | Purpose |
|-------|---------|
| `result` | Accumulated result matrix (starts as identity) |
| `base`   | The Q-matrix [[1,1],[1,0]] |
| `temp`   | Scratch matrix for operations |

Like Fast Doubling, matrix states use thread-local pooling with a maximum of 4 states per thread.

### Comparison with Fast Doubling

Matrix Exponentiation and Fast Doubling have the same O(log n) step count, but Matrix Exponentiation has a higher constant factor because:

- Each step involves 3 or 5 big-number multiplications (symmetric square/multiply) vs. 1 multiplication + 2 squarings for doubling
- It maintains a 2x2 matrix (4 big integers) vs. 2 big integers for doubling
- The result extraction is identical: a single field read

Matrix Exponentiation serves primarily as a **cross-validation** algorithm to verify Fast Doubling results.

---

## FFT-Based Calculator

**Source**: `crates/fibcalc-core/src/fft_based.rs` -- `FFTBasedCalculator`
**FFT Library**: `crates/fibcalc-bigfft/` -- NTT multiplication engine

### Algorithm Description

The FFT-Based calculator uses the same Fast Doubling framework (doubling identities + bit scanning) but replaces the multiplication strategy with an `AdaptiveStrategy` that switches to FFT-based multiplication when operand sizes exceed the FFT threshold.

This is critical for very large n (millions or higher), where operands grow to hundreds of thousands or millions of bits and standard Karatsuba multiplication becomes the bottleneck.

### NTT Multiplication Pipeline

The FFT multiplication uses a Number Theoretic Transform (NTT) over a Fermat ring, implementing the Schonhage-Strassen algorithm:

1. **Decomposition**: Split big integers into polynomials with small coefficients. Each integer is decomposed into "pieces" of `piece_bits` bits, forming a polynomial where each coefficient represents one piece.

2. **Forward NTT**: Transform polynomial coefficients into the frequency domain using NTT over the ring Z/(2^s + 1), where 2^s + 1 is a Fermat number.

3. **Pointwise multiplication**: Multiply corresponding coefficients in the frequency domain. This converts the O(n^2) convolution into O(n) independent multiplications.

4. **Inverse NTT**: Transform the product back to the time domain.

5. **Reassembly**: Reconstruct the result big integer from the polynomial coefficients, propagating carries.

### Fermat Number Arithmetic

**Source**: `crates/fibcalc-bigfft/src/fermat.rs` -- `FermatNum`

Fermat numbers F_k = 2^(2^k) + 1 serve as the modulus for NTT because 2 is a primitive root of unity in Z/F_k, enabling efficient butterfly operations via bit shifts rather than general modular multiplication.

The `FermatNum` type stores values as little-endian u64 limbs and implements:

| Operation | Description |
|-----------|-------------|
| `add` | Limb-level addition with carry, mod (2^shift + 1) |
| `sub` | Limb-level subtraction with borrow, mod (2^shift + 1) |
| `fermat_mul` | Multiplication mod (2^shift + 1) via BigUint fallback |
| `shift_left` | Multiplication by 2^s mod (2^shift + 1) |
| `shift_right` | Division by 2^k mod (2^shift + 1), via inverse shift |
| `normalize` | Reduce mod (2^shift + 1) |

### FFT Parameter Selection

The `select_fft_params` function in `crates/fibcalc-bigfft/src/fermat.rs` chooses optimal parameters based on operand size:

| Operand Bits | Piece Size | Description |
|-------------|------------|-------------|
| < 10,000    | 64 bits    | Small operands |
| < 100,000   | 256 bits   | Medium operands |
| < 1,000,000 | 1,024 bits | Large operands |
| >= 1,000,000| 4,096 bits | Very large operands |

The number of NTT points is the smallest power of 2 >= (n_a + n_b), where n_a and n_b are the number of pieces for each operand. The Fermat shift is chosen to provide sufficient precision to avoid wrap-around errors.

### Squaring Optimization

FFT squaring (`fft_square`) performs only **one** forward NTT instead of two, since both operands are the same. The pointwise step becomes pointwise squaring, saving 50% of the transform cost.

### Routing and Fallback

The public API in `crates/fibcalc-bigfft/src/fft.rs` routes to FFT only when operands exceed 10,000 bits (the `FFT_BIT_THRESHOLD` constant in the bigfft crate). Below this threshold, standard `num-bigint` multiplication is used, since FFT has significant overhead from polynomial splitting, transforms, and reassembly.

---

## Multiplication Strategies

**Source**: `crates/fibcalc-core/src/strategy.rs`

FibCalc-rs uses the Strategy pattern to decouple the doubling loop from the multiplication method. All strategies implement the `Multiplier` trait (narrow interface) and the `DoublingStepExecutor` trait (extended interface for optimized doubling steps).

### Trait Hierarchy

```
Multiplier (ISP: narrow interface)
  |-- multiply(a, b) -> BigUint
  |-- square(a) -> BigUint       (default: multiply(a, a))
  |-- name() -> &str
  |
  +-- DoublingStepExecutor (extends Multiplier)
        |-- execute_doubling_step(fk, fk1) -> (F(2k), F(2k+1))
```

### Karatsuba Strategy

**Struct**: `KaratsubaStrategy`

The simplest strategy. Delegates to `num-bigint`'s built-in multiplication, which uses Karatsuba's algorithm for operands above a certain size (approximately 32 limbs) and schoolbook multiplication below.

The doubling step uses:
- 1 multiplication: `fk * (2*fk1 - fk)` for F(2k)
- 2 squarings: `fk^2 + fk1^2` for F(2k+1)

### Parallel Karatsuba Strategy

**Struct**: `ParallelKaratsubaStrategy`

Extends Karatsuba by parallelizing the three independent multiplications in the doubling step when operand bit length exceeds the `parallel_threshold`:

```rust
if max_bits >= parallel_threshold {
    // Parallel: 3 concurrent multiplications via rayon::join
    let ((fk_sq, fk1_sq), f2k) = rayon::join(
        || rayon::join(|| fk * fk, || fk1 * fk1),
        || fk * &t,
    );
    f2k1 = fk_sq + fk1_sq;
}
```

The nested `rayon::join` creates three tasks: fk^2, fk1^2, and fk*t, which execute concurrently on the Rayon work-stealing thread pool.

Below the threshold, the strategy falls back to sequential execution to avoid the overhead of task scheduling for small operands.

### FFT-Only Strategy

**Struct**: `FFTOnlyStrategy`

Routes all multiplications through `fibcalc_bigfft::mul` and `fibcalc_bigfft::sqr`. The bigfft functions internally apply FFT only when operands exceed 10,000 bits, falling back to standard multiplication for smaller inputs.

### Adaptive Strategy

**Struct**: `AdaptiveStrategy`

The most sophisticated strategy, used by the `FFTBasedCalculator`. It dynamically selects the multiplication method based on operand size:

```
if max_bits >= fft_threshold:
    use FFT multiplication (fibcalc_bigfft::mul / sqr)
else:
    use standard Karatsuba (num-bigint)
```

The `fft_threshold` and `strassen_threshold` parameters can be tuned by the dynamic threshold manager or calibration system.

### Strategy Usage by Calculator

| Calculator | Strategy | Parallelism |
|-----------|----------|-------------|
| `OptimizedFastDoubling` | `ParallelKaratsubaStrategy` | rayon::join above threshold |
| `MatrixExponentiation` | Built-in symmetric multiply | None |
| `FFTBasedCalculator` | `AdaptiveStrategy` | FFT for large operands |

---

## Complexity Analysis

### Time Complexity

All three algorithms perform O(log n) steps. The total time is dominated by the cost of big-number operations on operands of increasing size.

F(n) has approximately n * log2(phi) ~ 0.694n bits. At step i (counting from the end), operands have approximately 0.694 * n / 2^i bits.

| Algorithm | Steps | Operations per Step | Total Time |
|-----------|-------|-------------------|------------|
| Fast Doubling | O(log n) | 1 multiply + 2 squarings (3 big-int ops) | O(M(n) * log n) |
| Matrix Exp | O(log n) | 3-5 multiplies (symmetric optimization) | O(M(n) * log n) |
| FFT-Based | O(log n) | Same as Fast Doubling, with FFT multiply | O(M(n) * log n) |

Where M(n) is the cost of multiplying two numbers of bit-length proportional to n:

| Multiplication Method | M(n) | Used When |
|----------------------|------|-----------|
| Schoolbook | O(n^2) | Very small operands |
| Karatsuba (num-bigint) | O(n^1.585) | Default, up to ~500K bits |
| Schonhage-Strassen (FFT) | O(n * log n * log log n) | Above FFT threshold |
| Parallel Karatsuba | O(n^1.585 / p) for p cores | Above parallel threshold |

### Space Complexity

| Algorithm | Space |
|-----------|-------|
| Fast Doubling | O(n) -- two big integers of ~n bits each, plus temporaries |
| Matrix Exp | O(n) -- four big integers in the 2x2 matrix |
| FFT-Based | O(n * log n) -- polynomial coefficients and transform workspace |

The FFT algorithm requires more memory due to the polynomial decomposition and transform arrays.

### Practical Constants

Fast Doubling has the smallest constant factor among the three algorithms because:
1. It maintains only 2 big integers (vs. 4 for Matrix Exponentiation)
2. Each step does 1 multiply + 2 squarings (vs. 3-5 multiplies for Matrix)
3. Squaring is faster than general multiplication (symmetric operands)

---

## Dynamic Thresholds

**Source**: `crates/fibcalc-core/src/dynamic_threshold.rs` -- `DynamicThresholdManager`
**Types**: `crates/fibcalc-core/src/threshold_types.rs`
**Constants**: `crates/fibcalc-core/src/constants.rs`

### Default Thresholds

| Threshold | Default Value | Purpose |
|-----------|--------------|---------|
| `ParallelThreshold` | 4,096 bits | Switch from sequential to parallel Karatsuba |
| `FFTThreshold` | 500,000 bits | Switch from Karatsuba to FFT multiplication |
| `StrassenThreshold` | 3,072 bits | Switch to Strassen-like matrix multiplication |
| `ParallelFFTThreshold` | 5,000,000 bits | Switch to parallel FFT execution |

### Adaptive Adjustment

The `DynamicThresholdManager` adjusts thresholds at runtime based on observed performance metrics. It uses a ring buffer to collect `IterationMetric` samples that record:

- Operand bit length
- FFT speedup factor (positive = FFT was faster)
- Parallel speedup factor
- Iteration duration (nanoseconds)
- Which multiplication method was used

### Adjustment Algorithm

The manager computes rolling averages from the ring buffer and adjusts thresholds using hysteresis:

1. **Compute statistics**: Average FFT benefit, parallel benefit, and Strassen benefit from the ring buffer
2. **Apply dead zone**: If the average benefit is within the dead zone (default: +/- 0.02), no adjustment is made
3. **Apply hysteresis**: Only adjust if the benefit exceeds the hysteresis factor (default: 0.05)
4. **Adjust threshold**: Decrease by up to `max_adjustment` (default: 10%) if the alternative method is faster, or increase if it is slower
5. **Apply floor**: Thresholds cannot go below minimum values (FFT: 1,024 bits, Parallel: 512 bits, Strassen: 512 bits)

### Configuration

The `DynamicThresholdConfig` controls the adjustment behavior:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `ring_buffer_size` | 32 | Number of metric samples to retain |
| `hysteresis_factor` | 0.05 | Minimum benefit to trigger adjustment |
| `max_adjustment` | 0.10 | Maximum fraction to adjust per cycle |
| `dead_zone` | 0.02 | Benefit range that causes no adjustment |

Thresholds can also be set directly from loaded calibration profiles via `set_thresholds()`.

---

## Algorithm Selection Flowchart

The algorithm selection logic is implemented in the orchestration layer (`crates/fibcalc-orchestration/src/calculator_selection.rs`) and the calculator decorator (`crates/fibcalc-core/src/calculator.rs`).

### Decision Tree

```
Input: n (Fibonacci index)
         |
         v
    n <= 93?  ------YES------>  Lookup FIB_TABLE[n]  (O(1), instant)
         |
        NO
         |
         v
    Algorithm specified?
     /       |       \
    v        v        v
  "fast"   "matrix"  "fft"    "all"
    |        |        |         |
    v        v        v         v
  Fast     Matrix    FFT     Run all 3
 Doubling   Exp    Based    in parallel
    |        |        |         |
    v        v        v         v
  Parallel  Symmetric Adaptive  Cross-
  Karatsuba Matrix   Strategy   validate
  Strategy   Ops                results
```

### Within Each Algorithm: Strategy Selection

For Fast Doubling and FFT-Based calculators, the multiplication strategy adapts at each iteration of the doubling loop:

```
Each doubling step:
    |
    v
  operand bits >= FFT threshold? ----YES----> FFT multiplication
    |
   NO
    |
    v
  operand bits >= parallel threshold? ----YES----> Parallel Karatsuba
    |                                               (rayon::join with 3 tasks)
   NO
    |
    v
  Sequential Karatsuba (num-bigint)
```

### When to Use Which

| Scenario | Recommended Algorithm | Reason |
|----------|----------------------|--------|
| n <= 93 | (automatic fast path) | Instant lookup, O(1) |
| n < 10,000 | Fast Doubling | Low overhead, fast Karatsuba |
| n < 1,000,000 | Fast Doubling + Parallel Karatsuba | Parallelism benefits on multi-core |
| n >= 1,000,000 | FFT-Based (Adaptive Strategy) | FFT multiplication dominates |
| Verification mode | All algorithms | Cross-validate results for correctness |

---

## Cross-Validation Mechanism

**Source**: `crates/fibcalc-orchestration/src/orchestrator.rs`

Cross-validation provides confidence in correctness by running multiple independent algorithms and comparing their results.

### Execution

When the algorithm is set to `"all"`, the orchestrator:

1. Retrieves all registered calculators from the `CalculatorFactory` (Fast Doubling, Matrix Exponentiation, FFT-Based)
2. Executes them **in parallel** using `rayon::ParallelIterator`
3. Each calculator receives its own `calc_index` for progress reporting
4. Optional timeout prevents infinite computation

### Result Analysis

The `analyze_comparison_results()` function:

1. Filters results to only those with a `value` and no `error`
2. If no valid results remain, returns `FibError::Calculation`
3. Compares all valid results against the first valid result
4. If any result differs, returns `FibError::Mismatch`
5. Otherwise, returns `Ok(())`

Results with errors (timeout, cancellation, failure) are excluded from comparison. This means cross-validation succeeds as long as all *successful* computations agree.

### Use Cases

- **Development**: Verify new algorithm implementations against known-good ones
- **Production**: Run with `--algo all` to double-check results for critical computations
- **Testing**: Golden file tests compare all three algorithms against known Fibonacci values

---

## Fast Path: Precomputed Lookup Table

**Source**: `crates/fibcalc-core/src/constants.rs` -- `FIB_TABLE`
**Decorator**: `crates/fibcalc-core/src/calculator.rs` -- `FibCalculator`

### Design

For n <= 93, F(n) fits in a 64-bit unsigned integer (`u64`). F(93) = 12,200,160,415,121,876,738 is the largest Fibonacci number that fits in u64. F(94) = 19,740,274,219,868,223,167 overflows u64.

FibCalc-rs precomputes all 94 values (F(0) through F(93)) in a compile-time constant array:

```rust
pub const FIB_TABLE: [u64; 94] = {
    let mut table = [0u64; 94];
    table[0] = 0;
    table[1] = 1;
    let mut i = 2;
    while i < 94 {
        table[i] = table[i - 1] + table[i - 2];
        i += 1;
    }
    table
};
```

This array is computed at compile time using const evaluation -- there is zero runtime cost.

### Decorator Pattern

The `FibCalculator` struct wraps any `CoreCalculator` and intercepts the `calculate()` call:

```rust
fn calculate(&self, cancel, observer, calc_index, n, opts) -> Result<BigUint, FibError> {
    if n <= MAX_FIB_U64 {   // MAX_FIB_U64 = 93
        observer.on_progress(&ProgressUpdate::done(calc_index, self.inner.name()));
        return Ok(BigUint::from(FIB_TABLE[n as usize]));
    }
    // ... delegate to inner CoreCalculator
}
```

This means that all three algorithms (Fast Doubling, Matrix Exponentiation, FFT-Based) benefit from the fast path transparently. The decorator also checks for cancellation before delegating to the core algorithm for large n.

### Performance Impact

For n <= 93, the computation is a single array index + BigUint conversion -- effectively O(1) with no heap allocation beyond the BigUint itself. This eliminates the overhead of the doubling loop, strategy selection, and progress reporting for the most common small inputs.
