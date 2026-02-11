//! Version information.

/// Get the version string.
#[must_use]
#[allow(dead_code)]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the full version string with build info.
#[must_use]
#[allow(dead_code)]
pub fn full_version() -> String {
    format!("fibcalc {} (rust {})", version(), rustc_version())
}

fn rustc_version() -> &'static str {
    // Will be populated at build time
    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_not_empty() {
        assert!(!version().is_empty());
    }
}
