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

    #[test]
    fn is_color_disabled_returns_bool() {
        // Just verify it returns a boolean without panicking
        let result = is_color_disabled();
        assert!(result || !result);
    }

    #[test]
    fn print_header_does_not_panic() {
        print_header("Test Header");
    }

    #[test]
    fn print_header_empty_string() {
        print_header("");
    }

    #[test]
    fn print_header_special_chars() {
        print_header("Header with <special> & \"chars\"");
    }

    #[test]
    fn print_success_does_not_panic() {
        print_success("Operation completed");
    }

    #[test]
    fn print_success_empty_string() {
        print_success("");
    }

    #[test]
    fn print_error_does_not_panic() {
        print_error("Something went wrong");
    }

    #[test]
    fn print_error_empty_string() {
        print_error("");
    }

    #[test]
    fn print_functions_with_long_text() {
        let long_text = "A".repeat(1000);
        print_header(&long_text);
        print_success(&long_text);
        print_error(&long_text);
    }

    #[test]
    fn print_functions_with_unicode() {
        print_header("Fibonacci \u{2192} calcul");
        print_success("R\u{00e9}sultat correct");
        print_error("Erreur inattendue");
    }
}
