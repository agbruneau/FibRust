//! Error handling and exit codes.

use fibcalc_core::calculator::FibError;
use fibcalc_core::constants::exit_codes;

/// Handle a calculation error and return the appropriate exit code.
#[allow(dead_code)]
pub fn handle_error(err: &FibError) -> i32 {
    match err {
        FibError::Calculation(_) | FibError::Overflow(_, _) | FibError::InvalidInput(_) => {
            exit_codes::ERROR_GENERIC
        }
        FibError::Config(_) => exit_codes::ERROR_CONFIG,
        FibError::Cancelled => exit_codes::ERROR_CANCELED,
        FibError::Timeout(_) => exit_codes::ERROR_TIMEOUT,
        FibError::Mismatch => exit_codes::ERROR_MISMATCH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes() {
        assert_eq!(handle_error(&FibError::Cancelled), 130);
        assert_eq!(handle_error(&FibError::Timeout("5m".into())), 2);
        assert_eq!(handle_error(&FibError::Mismatch), 3);
        assert_eq!(handle_error(&FibError::Config("bad".into())), 4);
    }
}
