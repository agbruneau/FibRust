//! UI helpers for CLI display.

use console::style;

/// Check if color output is disabled via `NO_COLOR` env var.
#[must_use]
pub fn is_color_disabled() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Print a styled header.
pub fn print_header(text: &str) {
    if is_color_disabled() {
        println!("=== {text} ===");
    } else {
        println!("{}", style(format!("=== {text} ===")).bold().cyan());
    }
}

/// Print a success message.
pub fn print_success(text: &str) {
    if is_color_disabled() {
        println!("[OK] {text}");
    } else {
        println!("{} {text}", style("[OK]").green().bold());
    }
}

/// Print an error message.
pub fn print_error(text: &str) {
    if is_color_disabled() {
        eprintln!("[ERROR] {text}");
    } else {
        eprintln!("{} {text}", style("[ERROR]").red().bold());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_check_does_not_panic() {
        let _ = is_color_disabled();
    }
}
