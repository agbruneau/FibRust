//! Calculator selection logic.

use std::sync::Arc;

use fibcalc_core::calculator::{Calculator, FibError};
use fibcalc_core::registry::CalculatorFactory;

/// Get calculators to run based on algorithm selection.
///
/// # Errors
///
/// Returns `FibError` if the requested algorithm name is unknown.
pub fn get_calculators_to_run(
    algo: &str,
    factory: &dyn CalculatorFactory,
) -> Result<Vec<Arc<dyn Calculator>>, FibError> {
    match algo {
        "all" => {
            let names = factory.available();
            let mut calcs = Vec::new();
            for name in names {
                calcs.push(factory.get(name)?);
            }
            Ok(calcs)
        }
        name => {
            let calc = factory.get(name)?;
            Ok(vec![calc])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fibcalc_core::registry::DefaultFactory;

    #[test]
    fn select_all() {
        let factory = DefaultFactory::new();
        let calcs = get_calculators_to_run("all", &factory).unwrap();

        #[cfg(not(feature = "gmp"))]
        let expected_count = 3;

        // If the workspace feature "gmp" is enabled, fibcalc-core/gmp is enabled too.
        // However, this test crate (fibcalc-orchestration) doesn't directly expose a "gmp" feature flag
        // in its Cargo.toml that propagates to fibcalc-core.
        // Instead, we check if the underlying factory reports GMP.
        #[cfg(feature = "gmp")]
        let expected_count = 4;

        // Fallback check if feature flag isn't reliable in test context (e.g. unified workspace features)
        let actual_expected = if factory.available().contains(&"gmp") {
            4
        } else {
            3
        };

        assert_eq!(calcs.len(), actual_expected);
    }

    #[test]
    fn select_single() {
        let factory = DefaultFactory::new();
        let calcs = get_calculators_to_run("fast", &factory).unwrap();
        assert_eq!(calcs.len(), 1);
        assert_eq!(calcs[0].name(), "FastDoubling");
    }

    #[test]
    fn select_unknown() {
        let factory = DefaultFactory::new();
        let result = get_calculators_to_run("unknown", &factory);
        assert!(result.is_err());
    }
}
