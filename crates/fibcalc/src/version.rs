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

    #[test]
    fn full_version_contains_version_and_rust() {
        let fv = full_version();
        // Should contain the package version
        assert!(
            fv.contains(version()),
            "full_version should contain the package version"
        );
        // Should contain "fibcalc"
        assert!(
            fv.starts_with("fibcalc "),
            "full_version should start with 'fibcalc '"
        );
        // Should contain "rust" somewhere
        assert!(fv.contains("rust"), "full_version should contain 'rust'");
    }

    #[test]
    fn full_version_contains_rustc_version() {
        let fv = full_version();
        let rv = rustc_version();
        assert!(fv.contains(rv), "full_version should embed rustc_version()");
    }

    #[test]
    fn rustc_version_returns_something() {
        let rv = rustc_version();
        // It currently returns "unknown" as a placeholder
        assert!(!rv.is_empty());
    }

    #[test]
    fn version_is_semver_like() {
        let v = version();
        // Should look like a semver: at least "x.y.z"
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2, "version should have at least major.minor");
    }
}
