# FibRust Refactoring Design

**Date:** 2026-02-24
**Approach:** Bottom-Up (memory first, then P2 activation, then perf, then hygiene)
**Scope:** 4 phases across all 7 workspace crates

## Codebase Assessment

Key findings from exploration:

| Area | Status | Action Required |
|------|--------|-----------------|
| Dynamic dispatch | No Box<dyn> in hot paths | Verify only |
| GMP isolation | Perfect feature gating | Verify only |
| Rayon parallelism | 4 parallelization points, configurable thresholds | Already done |
| Clippy pedantic | Workspace-wide with 10 exceptions | Already done |
| Memory management | 3 separate allocation systems, duplication | Consolidate |
| In-place operations | Matrix ops return new values; BigUint allocation-heavy | Optimize |
| P2 infrastructure | Arena, bump allocator, pool warming implemented but dead | Activate |
| Inline annotations | Only 4 in codebase | Conservative audit |

## Phase 1: Memory Consolidation

### New crate: fibcalc-memory

Unifies all allocation infrastructure into a single crate.

**What moves:**

| Component | From | To |
|-----------|------|-----|
| BigIntPool (mutex, size classes) | fibcalc-bigfft/src/pool.rs | fibcalc-memory/src/pool.rs |
| FFTBumpAllocator (bumpalo) | fibcalc-bigfft/src/bump.rs | fibcalc-memory/src/arena.rs |
| Pool warming (size prediction) | fibcalc-bigfft/src/pool_warming.rs | fibcalc-memory/src/warming.rs |
| ObjectPool<T> (generic) | fibcalc-core/src/pool.rs | fibcalc-memory/src/pool.rs (merged) |
| CalculationArena (bumpalo) | fibcalc-core/src/arena.rs | fibcalc-memory/src/arena.rs (merged) |
| Thread-local CalculationState pool | fibcalc-core/src/fastdoubling.rs (inline) | fibcalc-memory/src/thread_local.rs |
| Thread-local MatrixState pool | fibcalc-core/src/matrix.rs (inline) | fibcalc-memory/src/thread_local.rs |
| Allocation statistics | fibcalc-bigfft/src/pool.rs (inline) | fibcalc-memory/src/stats.rs |

**Target structure:**

```
fibcalc-memory/
  Cargo.toml          # deps: bumpalo, num-bigint, parking_lot
  src/
    lib.rs            # Re-exports
    pool.rs           # Unified BigIntPool + generic ObjectPool
    arena.rs          # Unified BumpArena (merges FFTBumpAllocator + CalculationArena)
    thread_local.rs   # Generic ThreadLocalPool<T> with acquire/release
    warming.rs        # Pool warming (from bigfft, unchanged)
    stats.rs          # Allocation statistics (atomic counters)
```

**Dependency graph change:**

```
Before:                          After:
fibcalc-core                     fibcalc-memory (new)
  └── fibcalc-bigfft               ↑              ↑
                                 fibcalc-core    fibcalc-bigfft
                                   └── fibcalc-bigfft
```

**Design decisions:**
- ThreadLocalPool<T>: single generic type with `acquire(factory)` / `release(value, reset)` replaces two identical inline implementations
- BigIntPool stays mutex-based with size classes (already well-optimized)
- Single BumpArena type with typed allocation methods (no trait abstraction)
- Concrete types only, no `dyn Allocator` (implementation details, not extension points)

## Phase 2: P2 Infrastructure Activation

### Activations

1. **FastDoubling** — Replace inline CalculationState thread-local pool with `fibcalc_memory::ThreadLocalPool<CalculationState>`. The CalculationState struct (5 BigUint registers) stays the same.

2. **Matrix** — Replace inline MatrixState thread-local pool with `fibcalc_memory::ThreadLocalPool<MatrixState>`.

3. **FFT loops** — Wire BumpArena into fft_core.rs for scratch buffer allocation. Currently FFT allocates Vec<u64> per iteration; the bump arena amortizes this.

4. **Pool warming** — In DefaultFactory::create() (registry.rs), call `warming::warm_pool(n)` to pre-allocate BigInt buffers sized for the target Fibonacci number.

5. **Dead code cleanup** — Remove all `#[allow(dead_code)]` annotations on activated infrastructure.

### Risk mitigation
- Each activation is independently testable
- Golden file tests validate correctness after each change
- Benchmarks measure performance impact
- No behavioral change: same algorithms, same results, different allocation strategy

## Phase 3: Performance Optimizations

### 3A. In-Place Matrix Operations

Add `square_symmetric_into(&mut self)` and `multiply_symmetric_into(&mut self, &Matrix)` to matrix_types.rs. Immutable versions remain for API compatibility but delegate to mutable ones internally.

Rationale: Matrix exponentiation calls these in O(log n) squarings. In-place mutation reuses buffer capacity via clone_from() and *= / += / -= operators.

### 3B. FastDoubling Allocation Reduction

Where num-bigint supports it, use mul_assign patterns. For operations that must produce new values, write results directly into pool registers via std::mem::swap rather than through intermediates.

Constraint: num-bigint's MulAssign only works for `BigUint *= &BigUint`. Can't do `a = b * c` in-place without a temporary. Improvement is incremental.

### 3C. Conservative Inline Audit

Add `#[inline]` only to:
- Pool acquire/release functions (called per-recursion)
- Matrix element accessors (if separate functions)
- Strategy dispatch execute_step() in adaptive multiplier

Method: Benchmark before/after with cargo bench. Only keep annotations that show measurable improvement.

### 3D. Observer Dispatch Verification

Verify (don't change) that:
- Progress checks in hot loops use FrozenObserver (atomic load, no vtable call)
- dyn dispatch only happens during freeze/thaw (cold path)
- Add `#[cfg(feature = "metrics")]` gate only if observers add measurable overhead

Likely outcome: No changes needed. Current design is sound.

## Phase 4: Code Hygiene & Verification

### 4A. Dead Code Cleanup

- Remove remaining `#[allow(dead_code)]` annotations
- For Strassen placeholder: remove or document
- Remove empty ObjectPool<T> from core (replaced by fibcalc-memory)
- Remove empty arena.rs and pool.rs stubs from core

### 4B. Shared Formatting Extraction

Move format_duration, format_number, format_result to fibcalc-orchestration formatting module (sits between core and CLI/TUI). Only if TUI actually duplicates the same logic.

### 4C. GMP Isolation Verification

Run `cargo build --no-default-features` and `cargo build --features gmp`. Verify no GMP types leak into non-gmp code paths.

### 4D. Full Test Suite Validation

1. `cargo test --workspace` — all 669+ tests pass
2. `cargo test --workspace --features gmp` — GMP tests pass (if available)
3. `cargo clippy --workspace -- -D warnings` — zero warnings
4. `cargo bench --package fibcalc-core` — benchmark comparison vs baseline
5. Golden file tests pass unchanged

## Phase 5: Documentation Update

### 5A. README.md

Update the main README to reflect:
- New fibcalc-memory crate in workspace description and architecture diagram
- Updated crate count (7 → 8)
- Updated dependency graph showing fibcalc-memory
- Any new performance characteristics from in-place operations and P2 activation

### 5B. docs/ARCHITECTURE.md

- Add fibcalc-memory crate description and responsibilities
- Update the architecture layer diagram to include the memory layer
- Document the unified allocation strategy (pools, arenas, thread-local, warming)
- Update dependency graph

### 5C. docs/PERFORMANCE.md

- Document allocation strategy changes and their impact
- Update benchmark results with before/after comparison
- Document the activated P2 infrastructure (arena, bump allocator, pool warming)
- Note inline annotation decisions and their measured impact

### 5D. docs/CHANGELOG.md

Add entry for this refactoring:
- New fibcalc-memory crate
- P2 infrastructure activation
- In-place matrix operations
- Dead code cleanup
- Performance benchmark comparison

### 5E. CLAUDE.md (Project Instructions)

- Update workspace description (8 crates instead of 7)
- Add fibcalc-memory to the architecture section
- Update the crate list with fibcalc-memory description
- Update line count and test count if changed

### 5F. Inline Documentation

- Ensure fibcalc-memory has module-level rustdoc on lib.rs, pool.rs, arena.rs, thread_local.rs
- Update rustdoc on moved/modified modules in core and bigfft to reference fibcalc-memory
- No unnecessary doc additions to unchanged code

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution approach | Bottom-Up | Memory is foundation; avoids rework |
| Unified allocator location | New fibcalc-memory crate | Breaks circular concern between core and bigfft |
| Inlining strategy | Conservative | Only proven bottlenecks; benchmark-validated |
| Dynamic thresholds | Keep runtime calibration | More portable; cost paid once at startup |
| Observer dispatch | Verify, don't change | Already well-designed |

## Success Criteria

- All existing tests pass (669+)
- Clippy pedantic clean
- Benchmark shows no regression (and ideally improvement on large n)
- Zero unsafe code
- Dead code markers removed from activated infrastructure
- Single source of truth for allocation (fibcalc-memory)
- README, ARCHITECTURE.md, PERFORMANCE.md, CHANGELOG.md, and CLAUDE.md updated
- fibcalc-memory has complete rustdoc coverage
