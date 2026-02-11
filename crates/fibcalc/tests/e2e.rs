//! End-to-end CLI integration tests.

use assert_cmd::Command;
use predicates::prelude::*;

fn fibcalc() -> Command {
    Command::cargo_bin("fibcalc").expect("binary not found")
}

#[test]
fn help_flag() {
    fibcalc()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fibonacci"));
}

#[test]
fn version_flag() {
    fibcalc()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fibcalc"));
}

#[test]
fn compute_f100_fast() {
    fibcalc()
        .args(["-n", "100", "--algo", "fast", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("354224848179261915075"));
}

#[test]
fn compute_f10_all_algos() {
    fibcalc()
        .args(["-n", "10", "--algo", "all", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("55"));
}

#[test]
fn compute_f0() {
    fibcalc()
        .args(["-n", "0", "--algo", "fast", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn compute_f1() {
    fibcalc()
        .args(["-n", "1", "--algo", "fast", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn verbose_mode() {
    fibcalc()
        .args(["-n", "100", "--algo", "fast", "-c", "-v"])
        .assert()
        .success();
}

#[test]
fn details_mode() {
    fibcalc()
        .args(["-n", "100", "--algo", "fast", "-c", "-d"])
        .assert()
        .success();
}

#[test]
fn invalid_algo() {
    fibcalc()
        .args(["-n", "100", "--algo", "invalid", "-c"])
        .assert()
        .failure();
}

#[test]
fn last_digits_mode() {
    fibcalc()
        .args([
            "-n",
            "1000",
            "--algo",
            "fast",
            "-c",
            "-q",
            "--last-digits",
            "10",
        ])
        .assert()
        .success();
}

#[test]
fn quiet_mode() {
    fibcalc()
        .args(["-n", "10", "--algo", "fast", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains("55"));
}

#[test]
fn compute_f1000() {
    fibcalc()
        .args(["-n", "1000", "--algo", "fast", "-c", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "43466557686937456435688527675040625802564",
        ));
}

#[test]
fn matrix_algo() {
    fibcalc()
        .args(["-n", "500", "--algo", "matrix", "-c", "-q"])
        .assert()
        .success();
}

#[test]
fn fft_algo() {
    fibcalc()
        .args(["-n", "500", "--algo", "fft", "-c", "-q"])
        .assert()
        .success();
}

#[test]
fn output_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("result.txt");
    fibcalc()
        .args([
            "-n",
            "100",
            "-c",
            "-q",
            "--algo",
            "fast",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn shell_completion_bash() {
    fibcalc()
        .args(["--completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fibcalc"));
}

#[test]
fn shell_completion_zsh() {
    fibcalc()
        .args(["--completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fibcalc"));
}

#[test]
fn shell_completion_fish() {
    fibcalc()
        .args(["--completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fibcalc"));
}

#[test]
fn env_var_fibcalc_n() {
    fibcalc()
        .env("FIBCALC_N", "42")
        .args(["-c", "-q", "--algo", "fast"])
        .assert()
        .success()
        .stdout(predicate::str::contains("267914296"));
}

#[test]
fn memory_limit_sufficient() {
    fibcalc()
        .args([
            "-n",
            "1000",
            "--memory-limit",
            "1G",
            "-c",
            "-q",
            "--algo",
            "fast",
        ])
        .assert()
        .success();
}

#[test]
fn memory_limit_insufficient() {
    fibcalc()
        .args([
            "-n",
            "1000000000",
            "--memory-limit",
            "1K",
            "-c",
            "--algo",
            "fast",
        ])
        .assert()
        .failure();
}

/// Timeout is only checked between parallel calculator dispatches, not
/// mid-computation. A single-algorithm run completes regardless of the
/// timeout value. This test documents the current behaviour.
#[test]
fn timeout_too_short() {
    // With --algo all (3 calculators in parallel), the timeout check
    // may trigger for later-dispatched workers, but the process still
    // returns success because partial results are accepted.
    fibcalc()
        .args([
            "-n",
            "100",
            "--timeout",
            "1ms",
            "-c",
            "-q",
            "--algo",
            "fast",
        ])
        .assert()
        .success();
}

#[test]
fn calibration_auto() {
    fibcalc()
        .args(["--auto-calibrate", "-q"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}
