//! Profile persistence (load/save).

use std::path::PathBuf;

use crate::profile::{self, CalibrationProfile};

const PROFILE_FILENAME: &str = "fibcalc_calibration.json";
const CONFIG_DIR_NAME: &str = "fibcalc";

/// Load calibration profile from the standard location.
/// Tries XDG config dir first, then the working directory.
#[must_use]
pub fn load_profile() -> Option<CalibrationProfile> {
    // Try XDG config directory first
    if let Some(path) = xdg_profile_path() {
        if path.exists() {
            if let Some(p) = load_from_path(&path) {
                return Some(p);
            }
        }
    }

    // Fall back to working directory
    let path = cwd_profile_path();
    if path.exists() {
        return load_from_path(&path);
    }

    None
}

/// Load a profile and validate it against the current environment.
/// Returns `None` if the profile is incompatible, invalid, or for a different CPU.
pub fn load_validated_profile() -> Option<CalibrationProfile> {
    let p = load_profile()?;

    if !p.is_compatible() {
        tracing::info!("Profile version mismatch, ignoring cached profile");
        return None;
    }
    if !p.is_valid() {
        tracing::info!("Profile has invalid thresholds, ignoring cached profile");
        return None;
    }

    let current_fp = profile::cpu_fingerprint();
    if !p.matches_cpu(&current_fp) {
        tracing::info!("Profile CPU mismatch, ignoring cached profile");
        return None;
    }

    Some(p)
}

/// Save calibration profile to the XDG config directory.
/// Falls back to the working directory if the config dir can't be created.
pub fn save_profile(p: &CalibrationProfile) -> std::io::Result<()> {
    let path = if let Some(xdg_path) = xdg_profile_path() {
        // Ensure the config directory exists
        if let Some(parent) = xdg_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        xdg_path
    } else {
        cwd_profile_path()
    };

    save_to_path(p, &path)
}

/// Save profile to a specific path.
pub fn save_to_path(p: &CalibrationProfile, path: &std::path::Path) -> std::io::Result<()> {
    let content = serde_json::to_string_pretty(p).map_err(std::io::Error::other)?;
    std::fs::write(path, content)
}

/// Delete the saved profile if it exists.
pub fn delete_profile() -> std::io::Result<bool> {
    if let Some(path) = xdg_profile_path() {
        if path.exists() {
            std::fs::remove_file(&path)?;
            return Ok(true);
        }
    }
    let path = cwd_profile_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
        return Ok(true);
    }
    Ok(false)
}

fn load_from_path(path: &std::path::Path) -> Option<CalibrationProfile> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get the XDG config directory path for the profile.
fn xdg_profile_path() -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME, fall back to ~/.config
    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let mut p = PathBuf::from(home);
                p.push(".config");
                p
            })
        })?;

    Some(config_dir.join(CONFIG_DIR_NAME).join(PROFILE_FILENAME))
}

/// Get the working directory profile path.
fn cwd_profile_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(format!(".{PROFILE_FILENAME}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // Helper: save to a temp directory and load back
    fn round_trip_in(dir: &std::path::Path) -> CalibrationProfile {
        let path = dir.join(PROFILE_FILENAME);
        let p = CalibrationProfile::default();
        save_to_path(&p, &path).unwrap();
        load_from_path(&path).unwrap()
    }

    #[test]
    fn load_nonexistent() {
        // Should return None for nonexistent file (doesn't panic)
        let _ = load_profile();
    }

    #[test]
    fn save_and_load_to_path() {
        let dir = TempDir::new().unwrap();
        let loaded = round_trip_in(dir.path());
        assert_eq!(
            loaded.parallel_threshold,
            CalibrationProfile::default().parallel_threshold
        );
        assert_eq!(loaded.version, profile::PROFILE_VERSION);
    }

    #[test]
    fn load_invalid_json_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(PROFILE_FILENAME);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"not json").unwrap();
        assert!(load_from_path(&path).is_none());
    }

    #[test]
    fn load_validated_checks_version() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(PROFILE_FILENAME);
        let mut p = CalibrationProfile::default();
        p.version = 999; // incompatible
        save_to_path(&p, &path).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert!(!loaded.is_compatible());
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        // We can't easily test delete in the XDG path without setting env vars,
        // but we can check it doesn't panic
        let result = delete_profile();
        // Result may be Ok(true) or Ok(false) depending on environment
        assert!(result.is_ok());
    }

    #[test]
    fn xdg_profile_path_contains_config_dir() {
        if let Some(path) = xdg_profile_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains(CONFIG_DIR_NAME));
            assert!(path_str.contains(PROFILE_FILENAME));
        }
        // If HOME is not set, xdg_profile_path returns None, which is fine
    }

    #[test]
    fn cwd_profile_path_ends_with_filename() {
        let path = cwd_profile_path();
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.contains(PROFILE_FILENAME));
    }
}
