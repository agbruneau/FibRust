//! Calculator selection logic.

use std::sync::Arc;

use fibcalc_core::calculator::{Calculator, FibError};
use fibcalc_core::registry::CalculatorFactory;

/// Get calculators to run based on algorithm selection.
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
        assert_eq!(calcs.len(), 3);
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
