//! Application entry point and dispatch.

use anyhow::Result;

use fibcalc_cli::output::write_to_file;
use fibcalc_cli::presenter::CLIResultPresenter;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;
use fibcalc_core::registry::DefaultFactory;
use fibcalc_orchestration::calculator_selection::get_calculators_to_run;
use fibcalc_orchestration::interfaces::ResultPresenter;
use fibcalc_orchestration::orchestrator::{
    analyze_comparison_results, execute_calculations, execute_calculations_with_observer,
};

use crate::config::AppConfig;

/// Run the application.
///
/// # Errors
///
/// Returns an error if calculation, calibration, or TUI execution fails.
pub fn run(config: &AppConfig) -> Result<()> {
    // Handle shell completion
    if let Some(shell) = config.completion {
        let mut cmd = <AppConfig as clap::CommandFactory>::command();
        fibcalc_cli::completion::generate_completion(&mut cmd, shell, &mut std::io::stdout());
        return Ok(());
    }

    // Handle calibration
    if config.calibrate || config.auto_calibrate {
        return run_calibration(config);
    }

    // Handle TUI mode
    if config.tui {
        return run_tui(config);
    }

    // CLI mode
    run_cli(config)
}

/// Build `Options` from `AppConfig`, validating the memory-limit string.
fn build_options(config: &AppConfig) -> Result<Options> {
    Ok(Options {
        parallel_threshold: config.threshold,
        fft_threshold: config.fft_threshold,
        strassen_threshold: config.strassen_threshold,
        last_digits: if config.last_digits == 0 {
            None
        } else {
            Some(config.last_digits)
        },
        memory_limit: if config.memory_limit.is_empty() {
            None
        } else {
            Some(
                fibcalc_core::memory_budget::parse_memory_limit(&config.memory_limit).map_err(
                    |e| anyhow::anyhow!("invalid --memory-limit '{}': {e}", config.memory_limit),
                )?,
            )
        },
        verbose: config.verbose,
        details: config.details,
    }
    .normalize())
}

fn run_cli(config: &AppConfig) -> Result<()> {
    let opts = build_options(config)?;

    // Memory budget check
    let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
    if !estimate.fits_in(opts.memory_limit) {
        anyhow::bail!(
            "Estimated memory ({} MB) exceeds limit ({} MB)",
            estimate.total_bytes / (1024 * 1024),
            opts.memory_limit.unwrap_or(0) / (1024 * 1024)
        );
    }

    let factory = DefaultFactory::new();
    let calculators = get_calculators_to_run(&config.algo, &factory)?;
    let cancel = CancellationToken::new();

    // Set up Ctrl+C handler
    let cancel_clone = cancel.clone();
    ctrlc_handler(cancel_clone);

    let timeout = Some(config.timeout_duration());
    let results = execute_calculations(&calculators, config.n, &opts, &cancel, timeout);

    // Analyze results
    if results.len() > 1 {
        if let Err(e) = analyze_comparison_results(&results) {
            eprintln!("Warning: {e}");
        }
    }

    // Present results
    let presenter = CLIResultPresenter::new(config.verbose, config.quiet);
    for result in &results {
        if let Ok(value) = &result.outcome {
            presenter.present_result(
                &result.algorithm,
                config.n,
                value,
                result.duration,
                config.details,
            );
        } else if let Err(error) = &result.outcome {
            presenter.present_error(error);
        }
    }

    // Present comparison if multiple
    if results.len() > 1 {
        presenter.present_comparison(&results);
    }

    // Write to file if requested
    if let Some(ref path) = config.output {
        if let Some(result) = results.iter().find(|r| r.outcome.is_ok()) {
            write_to_file(path, result.outcome.as_ref().unwrap())?;
        }
    }

    Ok(())
}

fn run_calibration(config: &AppConfig) -> Result<()> {
    use fibcalc_calibration::calibration::{CalibrationEngine, CalibrationMode};

    let mode = if config.calibrate {
        CalibrationMode::Full
    } else {
        CalibrationMode::Auto
    };

    let engine = CalibrationEngine::new(mode);
    let profile = engine.calibrate();

    if !config.quiet {
        println!("Calibration complete:");
        println!("  Parallel threshold: {} bits", profile.parallel_threshold);
        println!("  FFT threshold: {} bits", profile.fft_threshold);
        println!("  Strassen threshold: {} bits", profile.strassen_threshold);
    }

    fibcalc_calibration::io::save_profile(&profile)?;
    Ok(())
}

fn run_tui(config: &AppConfig) -> Result<()> {
    let opts = build_options(config)?;

    // Memory budget check
    let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
    if !estimate.fits_in(opts.memory_limit) {
        anyhow::bail!(
            "Estimated memory ({} MB) exceeds limit ({} MB)",
            estimate.total_bytes / (1024 * 1024),
            opts.memory_limit.unwrap_or(0) / (1024 * 1024)
        );
    }

    let factory = DefaultFactory::new();
    let calculators = get_calculators_to_run(&config.algo, &factory)?;
    let cancel = CancellationToken::new();

    // Set up Ctrl+C handler
    let cancel_clone = cancel.clone();
    ctrlc_handler(cancel_clone);

    // Create crossbeam channel for TUI messages
    let (tx, rx) = crossbeam_channel::unbounded::<fibcalc_tui::TuiMessage>();

    // Create TUI app
    let mut app = fibcalc_tui::TuiApp::new(rx);
    app.set_n(config.n);

    // Spawn metrics collection thread
    let metrics_tx = tx.clone();
    let metrics_cancel = cancel.clone();
    std::thread::spawn(move || {
        let mut collector = fibcalc_tui::MetricsCollector::new();
        loop {
            if metrics_cancel.is_cancelled() {
                break;
            }
            collector.refresh();
            if metrics_tx
                .send(fibcalc_tui::TuiMessage::SystemMetrics(collector.snapshot()))
                .is_err()
            {
                break; // channel closed, TUI exited
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // Spawn background thread for calculations
    let n = config.n;
    let timeout = Some(config.timeout_duration());
    std::thread::spawn(move || {
        let _ = tx.send(fibcalc_tui::TuiMessage::Started);

        // Use the bridge observer so progress updates reach the TUI
        let observer = fibcalc_tui::TuiBridgeObserver::new(tx.clone());
        let results =
            execute_calculations_with_observer(&calculators, n, &opts, &cancel, timeout, &observer);

        // Analyze comparison results
        if results.len() > 1 {
            if let Err(e) = analyze_comparison_results(&results) {
                let _ = tx.send(fibcalc_tui::TuiMessage::Log(format!("Warning: {e}")));
            }
        }

        // Send results to TUI
        for result in &results {
            if result.outcome.is_ok() {
                let _ = tx.send(fibcalc_tui::TuiMessage::Complete {
                    algorithm: result.algorithm.clone(),
                    duration: result.duration,
                });
                let _ = tx.send(fibcalc_tui::TuiMessage::Log(format!(
                    "F({n}) computed by {} in {:.3?}",
                    result.algorithm, result.duration
                )));
            } else if let Err(error) = &result.outcome {
                let _ = tx.send(fibcalc_tui::TuiMessage::Error(format!(
                    "{}: {error}",
                    result.algorithm
                )));
            }
        }

        // Freeze the elapsed timer
        let _ = tx.send(fibcalc_tui::TuiMessage::Finished);
        let _ = tx.send(fibcalc_tui::TuiMessage::Log(
            "All calculations complete. Press 'q' to quit.".to_string(),
        ));
    });

    // Run TUI event loop on the main thread
    app.run().map_err(|e| anyhow::anyhow!("TUI error: {e}"))?;

    Ok(())
}

/// # Panics
///
/// Panics if the Ctrl+C signal handler cannot be registered with the OS.
fn ctrlc_handler(cancel: CancellationToken) {
    ctrlc::set_handler(move || {
        cancel.cancel();
    })
    .expect("Error setting Ctrl+C handler");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use fibcalc_core::progress::CancellationToken;
    use fibcalc_core::registry::DefaultFactory;
    use fibcalc_orchestration::calculator_selection::get_calculators_to_run;
    use fibcalc_orchestration::orchestrator::{analyze_comparison_results, execute_calculations};

    /// Helper to build a minimal AppConfig for testing.
    fn test_config() -> AppConfig {
        AppConfig {
            n: 100,
            algo: "fast".to_string(),
            calculate: false,
            verbose: false,
            details: false,
            output: None,
            quiet: false,
            calibrate: false,
            auto_calibrate: false,
            timeout: "5m".to_string(),
            threshold: 0,
            fft_threshold: 0,
            strassen_threshold: 0,
            tui: false,
            completion: None,
            last_digits: 0,
            memory_limit: String::new(),
        }
    }

    /// Build Options from config (delegates to the shared build_options helper).
    fn opts_from_config(config: &AppConfig) -> Options {
        build_options(config).expect("test config should always produce valid options")
    }

    /// Execute the core logic of run_cli without the ctrlc handler.
    /// This mirrors the run_cli function except for ctrlc_handler setup.
    fn execute_cli_logic(config: &AppConfig) -> Result<()> {
        let opts = opts_from_config(config);

        // Memory budget check
        let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
        if !estimate.fits_in(opts.memory_limit) {
            anyhow::bail!(
                "Estimated memory ({} MB) exceeds limit ({} MB)",
                estimate.total_bytes / (1024 * 1024),
                opts.memory_limit.unwrap_or(0) / (1024 * 1024)
            );
        }

        let factory = DefaultFactory::new();
        let calculators = get_calculators_to_run(&config.algo, &factory)?;
        let cancel = CancellationToken::new();
        let timeout = Some(config.timeout_duration());
        let results = execute_calculations(&calculators, config.n, &opts, &cancel, timeout);

        if results.len() > 1 {
            if let Err(e) = analyze_comparison_results(&results) {
                eprintln!("Warning: {e}");
            }
        }

        let presenter = CLIResultPresenter::new(config.verbose, config.quiet);
        for result in &results {
            if let Ok(value) = &result.outcome {
                presenter.present_result(
                    &result.algorithm,
                    config.n,
                    value,
                    result.duration,
                    config.details,
                );
            } else if let Err(error) = &result.outcome {
                presenter.present_error(error);
            }
        }

        if results.len() > 1 {
            presenter.present_comparison(&results);
        }

        if let Some(ref path) = config.output {
            if let Some(result) = results.iter().find(|r| r.outcome.is_ok()) {
                write_to_file(path, result.outcome.as_ref().unwrap())?;
            }
        }

        Ok(())
    }

    #[test]
    fn run_cli_single_algorithm_fast() {
        let config = test_config();
        let result = execute_cli_logic(&config);
        assert!(
            result.is_ok(),
            "run_cli with algo=fast should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_cli_all_algorithms() {
        let mut config = test_config();
        config.algo = "all".to_string();
        config.n = 50;
        let result = execute_cli_logic(&config);
        assert!(
            result.is_ok(),
            "run_cli with algo=all should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_cli_matrix_algorithm() {
        let mut config = test_config();
        config.algo = "matrix".to_string();
        config.n = 50;
        let result = execute_cli_logic(&config);
        assert!(
            result.is_ok(),
            "run_cli with algo=matrix should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_cli_fft_algorithm() {
        let mut config = test_config();
        config.algo = "fft".to_string();
        config.n = 50;
        let result = execute_cli_logic(&config);
        assert!(
            result.is_ok(),
            "run_cli with algo=fft should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_cli_verbose_mode() {
        let mut config = test_config();
        config.verbose = true;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_quiet_mode() {
        let mut config = test_config();
        config.quiet = true;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_details_mode() {
        let mut config = test_config();
        config.details = true;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_with_output_file() {
        let dir = TempDir::new().unwrap();
        let output_path = dir.path().join("fib_output.txt");
        let mut config = test_config();
        config.output = Some(output_path.to_string_lossy().to_string());
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
        assert!(output_path.exists(), "Output file should be created");
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(!content.is_empty(), "Output file should contain the result");
    }

    #[test]
    fn run_cli_with_last_digits() {
        let mut config = test_config();
        config.last_digits = 10;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_with_custom_thresholds() {
        let mut config = test_config();
        config.threshold = 8192;
        config.fft_threshold = 500_000;
        config.strassen_threshold = 3072;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_memory_limit_exceeded() {
        let mut config = test_config();
        config.n = 100_000_000;
        config.memory_limit = "1B".to_string();
        let result = execute_cli_logic(&config);
        assert!(result.is_err(), "Should fail when memory limit is exceeded");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeds limit"),
            "Error should mention memory limit: {err_msg}"
        );
    }

    #[test]
    fn run_cli_memory_limit_sufficient() {
        let mut config = test_config();
        config.n = 100;
        config.memory_limit = "8G".to_string();
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_unknown_algorithm_fails() {
        let mut config = test_config();
        config.algo = "nonexistent".to_string();
        let result = execute_cli_logic(&config);
        assert!(result.is_err(), "Unknown algorithm should produce an error");
    }

    #[test]
    fn run_cli_n_zero() {
        let mut config = test_config();
        config.n = 0;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok(), "n=0 should be handled: {:?}", result.err());
    }

    #[test]
    fn run_cli_n_one() {
        let mut config = test_config();
        config.n = 1;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_large_n_fast_path() {
        let mut config = test_config();
        config.n = 93;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_just_above_fast_path() {
        let mut config = test_config();
        config.n = 94;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_cli_comparison_with_all_algos() {
        let mut config = test_config();
        config.algo = "all".to_string();
        config.n = 1000;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_dispatches_to_calibration() {
        let mut config = test_config();
        config.auto_calibrate = true;
        config.quiet = true;
        let result = run(&config);
        assert!(
            result.is_ok(),
            "auto_calibrate should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_calibration_full_mode() {
        let mut config = test_config();
        config.calibrate = true;
        config.quiet = true;
        let result = run_calibration(&config);
        assert!(
            result.is_ok(),
            "Full calibration should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_calibration_auto_mode() {
        let mut config = test_config();
        config.auto_calibrate = true;
        config.quiet = true;
        let result = run_calibration(&config);
        assert!(
            result.is_ok(),
            "Auto calibration should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn run_calibration_verbose() {
        let mut config = test_config();
        config.auto_calibrate = true;
        config.quiet = false;
        let result = run_calibration(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn timeout_duration_default() {
        let config = test_config();
        let duration = config.timeout_duration();
        assert_eq!(duration, std::time::Duration::from_secs(300));
    }

    #[test]
    fn run_cli_with_timeout() {
        let mut config = test_config();
        config.timeout = "30s".to_string();
        config.n = 50;
        let result = execute_cli_logic(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn opts_from_config_normalizes_zeros() {
        let config = test_config();
        let opts = opts_from_config(&config);
        assert!(
            opts.parallel_threshold > 0,
            "Should apply default parallel threshold"
        );
        assert!(opts.fft_threshold > 0, "Should apply default FFT threshold");
        assert!(
            opts.strassen_threshold > 0,
            "Should apply default Strassen threshold"
        );
    }

    #[test]
    fn opts_from_config_preserves_custom_values() {
        let mut config = test_config();
        config.threshold = 8192;
        config.fft_threshold = 600_000;
        config.strassen_threshold = 4096;
        config.last_digits = 20;
        config.verbose = true;
        config.details = true;
        let opts = opts_from_config(&config);
        assert_eq!(opts.parallel_threshold, 8192);
        assert_eq!(opts.fft_threshold, 600_000);
        assert_eq!(opts.strassen_threshold, 4096);
        assert_eq!(opts.last_digits, Some(20));
        assert!(opts.verbose);
        assert!(opts.details);
    }

    #[test]
    fn memory_budget_check_zero_unlimited() {
        let config = test_config();
        let opts = opts_from_config(&config);
        let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
        // Default memory_limit="" parses to None, which means unlimited
        assert_eq!(opts.memory_limit, None);
        assert!(estimate.fits_in(opts.memory_limit));
    }

    #[test]
    fn run_cli_output_file_contains_correct_value() {
        let dir = TempDir::new().unwrap();
        let output_path = dir.path().join("fib10.txt");
        let mut config = test_config();
        config.n = 10; // F(10) = 55
        config.output = Some(output_path.to_string_lossy().to_string());
        execute_cli_logic(&config).unwrap();
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "55");
    }

    #[test]
    fn run_cli_all_algos_output_file() {
        let dir = TempDir::new().unwrap();
        let output_path = dir.path().join("fib_all.txt");
        let mut config = test_config();
        config.algo = "all".to_string();
        config.n = 10;
        config.output = Some(output_path.to_string_lossy().to_string());
        execute_cli_logic(&config).unwrap();
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "55");
    }

    #[test]
    fn run_calibration_mode_selection() {
        // calibrate=true should pick Full mode
        let mut config = test_config();
        config.calibrate = true;
        config.quiet = true;
        let result = run_calibration(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn run_dispatches_calibrate_flag() {
        let mut config = test_config();
        config.calibrate = true;
        config.quiet = true;
        let result = run(&config);
        assert!(result.is_ok());
    }
}
