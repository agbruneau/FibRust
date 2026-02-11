//! GMP-based calculator using the `rug` crate.
//!
//! Only available when the `gmp` feature is enabled.

#[cfg(feature = "gmp")]
use rug::Integer;

// P2 feature: GMP-backed calculator using rug::Integer for hardware-accelerated
// big-integer arithmetic. The default pure-Rust num-bigint backend provides full
// correctness and portability; GMP integration would add ~2-3x speedup for
// very large n (>1M) at the cost of an LGPL system dependency (libgmp).
