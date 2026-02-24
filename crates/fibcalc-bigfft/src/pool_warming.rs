//! Pool warming — re-exported from fibcalc-memory.

#[allow(unused_imports)] // Will be used when warming is wired into registry
pub use fibcalc_memory::warming::{
    estimate_result_bits, predict_sizes, warm_pool, warm_pool_default, SizePrediction,
    WarmingConfig,
};
