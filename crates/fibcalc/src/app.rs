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

fn run_cli(config: &AppConfig) -> Result<()> {
    let opts = Options {
        parallel_threshold: config.threshold,
        fft_threshold: config.fft_threshold,
        strassen_threshold: config.strassen_threshold,
        last_digits: config.last_digits,
        memory_limit: fibcalc_core::memory_budget::parse_memory_limit(&config.memory_limit)
            .unwrap_or(0),
        verbose: config.verbose,
        details: config.details,
    }
    .normalize();

    // Memory budget check
    let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
    if !estimate.fits_in(opts.memory_limit) {
        anyhow::bail!(
            "Estimated memory ({} MB) exceeds limit ({} MB)",
            estimate.total_bytes / (1024 * 1024),
            opts.memory_limit / (1024 * 1024)
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
        if let Some(value) = &result.value {
            presenter.present_result(
                &result.algorithm,
                config.n,
                value,
                result.duration,
                config.details,
            );
        } else if let Some(error) = &result.error {
            presenter.present_error(error);
        }
    }

    // Present comparison if multiple
    if results.len() > 1 {
        presenter.present_comparison(&results);
    }

    // Write to file if requested
    if let Some(ref path) = config.output {
        if let Some(result) = results.iter().find(|r| r.value.is_some()) {
            write_to_file(path, result.value.as_ref().unwrap())?;
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
    let opts = Options {
        parallel_threshold: config.threshold,
        fft_threshold: config.fft_threshold,
        strassen_threshold: config.strassen_threshold,
        last_digits: config.last_digits,
        memory_limit: fibcalc_core::memory_budget::parse_memory_limit(&config.memory_limit)
            .unwrap_or(0),
        verbose: config.verbose,
        details: config.details,
    }
    .normalize();

    // Memory budget check
    let estimate = fibcalc_core::memory_budget::MemoryEstimate::estimate(config.n);
    if !estimate.fits_in(opts.memory_limit) {
        anyhow::bail!(
            "Estimated memory ({} MB) exceeds limit ({} MB)",
            estimate.total_bytes / (1024 * 1024),
            opts.memory_limit / (1024 * 1024)
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
                .send(fibcalc_tui::TuiMessage::SystemMetrics(
                    collector.snapshot(),
                ))
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
            if result.value.is_some() {
                let _ = tx.send(fibcalc_tui::TuiMessage::Complete {
                    algorithm: result.algorithm.clone(),
                    duration: result.duration,
                });
                let _ = tx.send(fibcalc_tui::TuiMessage::Log(format!(
                    "F({n}) computed by {} in {:.3?}",
                    result.algorithm, result.duration
                )));
            } else if let Some(error) = &result.error {
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

fn ctrlc_handler(cancel: CancellationToken) {
    ctrlc::set_handler(move || {
        cancel.cancel();
    })
    .expect("Error setting Ctrl+C handler");
}
