//! Calculator factory and registry.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::calculator::{Calculator, FibCalculator, FibError};
use crate::fastdoubling::OptimizedFastDoubling;
use crate::fft_based::FFTBasedCalculator;
use crate::matrix::MatrixExponentiation;

/// Factory trait for creating calculators.
pub trait CalculatorFactory: Send + Sync {
    /// Get or create a calculator by name.
    fn get(&self, name: &str) -> Result<Arc<dyn Calculator>, FibError>;

    /// List all available calculator names.
    fn available(&self) -> Vec<&str>;
}

/// Default factory with lazy creation and cache.
pub struct DefaultFactory {
    cache: RwLock<HashMap<String, Arc<dyn Calculator>>>,
}

impl DefaultFactory {
    /// Create a new default factory.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn create_calculator(name: &str) -> Result<Arc<dyn Calculator>, FibError> {
        match name {
            "fast" | "fastdoubling" => {
                let core = Arc::new(OptimizedFastDoubling::new());
                Ok(Arc::new(FibCalculator::new(core)))
            }
            "matrix" => {
                let core = Arc::new(MatrixExponentiation::new());
                Ok(Arc::new(FibCalculator::new(core)))
            }
            "fft" => {
                let core = Arc::new(FFTBasedCalculator::new());
                Ok(Arc::new(FibCalculator::new(core)))
            }
            _ => Err(FibError::Config(format!("unknown calculator: {name}"))),
        }
    }
}

impl Default for DefaultFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorFactory for DefaultFactory {
    fn get(&self, name: &str) -> Result<Arc<dyn Calculator>, FibError> {
        // Check cache first
        if let Some(calc) = self.cache.read().get(name) {
            return Ok(Arc::clone(calc));
        }

        // Create and cache
        let calc = Self::create_calculator(name)?;
        self.cache
            .write()
            .insert(name.to_string(), Arc::clone(&calc));
        Ok(calc)
    }

    fn available(&self) -> Vec<&str> {
        vec!["fast", "matrix", "fft"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_creates_fast_doubling() {
        let factory = DefaultFactory::new();
        let calc = factory.get("fast");
        assert!(calc.is_ok());
        assert_eq!(calc.unwrap().name(), "FastDoubling");
    }

    #[test]
    fn factory_creates_matrix() {
        let factory = DefaultFactory::new();
        let calc = factory.get("matrix");
        assert!(calc.is_ok());
        assert_eq!(calc.unwrap().name(), "MatrixExponentiation");
    }

    #[test]
    fn factory_creates_fft() {
        let factory = DefaultFactory::new();
        let calc = factory.get("fft");
        assert!(calc.is_ok());
        assert_eq!(calc.unwrap().name(), "FFTBased");
    }

    #[test]
    fn factory_caches() {
        let factory = DefaultFactory::new();
        let calc1 = factory.get("fast").unwrap();
        let calc2 = factory.get("fast").unwrap();
        assert!(Arc::ptr_eq(&calc1, &calc2));
    }

    #[test]
    fn factory_unknown_name() {
        let factory = DefaultFactory::new();
        assert!(factory.get("nonexistent").is_err());
    }

    #[test]
    fn factory_available() {
        let factory = DefaultFactory::new();
        let available = factory.available();
        assert!(available.contains(&"fast"));
        assert!(available.contains(&"matrix"));
        assert!(available.contains(&"fft"));
    }
}
