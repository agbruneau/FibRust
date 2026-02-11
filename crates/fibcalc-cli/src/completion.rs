//! Shell completion generation.

use std::io;

use clap::Command;
use clap_complete::{generate, Shell};

/// Generate shell completion script.
pub fn generate_completion(cmd: &mut Command, shell: Shell, out: &mut dyn io::Write) {
    generate(shell, cmd, "fibcalc", out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_bash_completion() {
        let mut cmd = Command::new("fibcalc");
        let mut buf = Vec::new();
        generate_completion(&mut cmd, Shell::Bash, &mut buf);
        assert!(!buf.is_empty());
    }
}
