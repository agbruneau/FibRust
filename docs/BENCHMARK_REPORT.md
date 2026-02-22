# Benchmark Report

**Date:** 2026-02-22
**Benchmark framework:** Criterion 0.5.1
**Profile:** `bench` (optimized + debuginfo)

## System Information

| Property | Value |
|----------|-------|
| OS | Windows 11 Pro 10.0.26220 |
| CPU | Intel Core Ultra 9 275HX |
| Cores | 24 physical / 24 logical |
| RAM | 64 GB |
| Rust | rustc 1.92.0 (ded5c06cf 2025-12-08) |
| Machine | ASUS ROG Strix SCAR 18 G835LW |

## Benchmark Results

All three Fibonacci algorithms were benchmarked at five input sizes: 100, 1,000, 10,000, 100,000, and 1,000,000. Each benchmark collected 100 samples using Criterion's statistical analysis.

### Summary Table

| n | Fast Doubling | Matrix Exponentiation | FFT-Based |
|--:|:-------------:|:---------------------:|:---------:|
| 100 | **784 ns** | 1.67 us | 991 ns |
| 1,000 | **1.21 us** | 2.66 us | 1.48 us |
| 10,000 | **7.64 us** | 11.02 us | 7.72 us |
| 100,000 | **216 us** | 373 us | 270 us |
| 1,000,000 | **3.46 ms** | 11.84 ms | 8.75 ms |

**Bold** = fastest algorithm for each input size.

### Detailed Results

#### Fast Doubling

| n | Lower bound | Estimate | Upper bound |
|--:|------------:|---------:|------------:|
| 100 | 778.03 ns | 784.13 ns | 790.68 ns |
| 1,000 | 1.2053 us | 1.2109 us | 1.2172 us |
| 10,000 | 7.5878 us | 7.6416 us | 7.7087 us |
| 100,000 | 214.49 us | 216.45 us | 218.91 us |
| 1,000,000 | 3.4376 ms | 3.4622 ms | 3.4933 ms |

#### Matrix Exponentiation

| n | Lower bound | Estimate | Upper bound |
|--:|------------:|---------:|------------:|
| 100 | 1.6636 us | 1.6672 us | 1.6711 us |
| 1,000 | 2.6094 us | 2.6561 us | 2.7208 us |
| 10,000 | 10.995 us | 11.017 us | 11.043 us |
| 100,000 | 371.88 us | 372.77 us | 373.89 us |
| 1,000,000 | 11.810 ms | 11.835 ms | 11.866 ms |

#### FFT-Based

| n | Lower bound | Estimate | Upper bound |
|--:|------------:|---------:|------------:|
| 100 | 989.76 ns | 991.35 ns | 993.29 ns |
| 1,000 | 1.4742 us | 1.4774 us | 1.4819 us |
| 10,000 | 7.7024 us | 7.7159 us | 7.7300 us |
| 100,000 | 269.55 us | 269.99 us | 270.45 us |
| 1,000,000 | 8.7274 ms | 8.7531 ms | 8.7851 ms |

### Scaling Analysis

**Fast Doubling** is the fastest algorithm across all tested input sizes. At n=1,000,000 it is 3.4x faster than Matrix Exponentiation and 2.5x faster than FFT-Based.

Relative performance (ratio vs Fast Doubling):

| n | Matrix Exponentiation | FFT-Based |
|--:|----------------------:|----------:|
| 100 | 2.13x | 1.26x |
| 1,000 | 2.19x | 1.22x |
| 10,000 | 1.44x | 1.01x |
| 100,000 | 1.72x | 1.25x |
| 1,000,000 | 3.42x | 2.53x |

FFT-Based and Fast Doubling perform similarly at n=10,000 (within 1%), but Fast Doubling pulls ahead significantly at larger inputs.

## How to Reproduce

```bash
cargo bench --bench fibonacci -p fibcalc-core
```

HTML reports are generated in `target/criterion/` (requires gnuplot for full reports; falls back to plotters backend otherwise).
