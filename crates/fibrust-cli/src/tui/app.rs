//! Application state management for the TUI.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::state::*;

/// Maximum number of progress history entries for sparkline.
const MAX_PROGRESS_HISTORY: usize = 60;

/// Main application state.
pub struct App {
    /// Current application mode.
    pub mode: AppMode,
    /// Input state.
    pub input: InputState,
    /// Calculation state.
    pub calculation: CalculationState,
    /// Progress history for sparkline display.
    pub progress_history: VecDeque<f64>,
    /// Algorithm results.
    pub results: Vec<AlgorithmResult>,
    /// Result analysis.
    pub analysis: ResultAnalysis,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// System info.
    pub system_info: SystemInfo,
    /// Calculation start time.
    calculation_start: Option<Instant>,
    /// Shared progress value for calculation thread.
    shared_progress: Arc<AtomicU64>,
    /// Shared cancel flag.
    cancel_flag: Arc<AtomicBool>,
}

/// System information for display.
pub struct SystemInfo {
    pub cpu_count: usize,
    pub parallelism_threshold: &'static str,
    pub fft_threshold: &'static str,
    pub version: &'static str,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu_count: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            parallelism_threshold: "40,000",
            fft_threshold: "50,000 bits",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl App {
    /// Create a new App instance.
    pub fn new() -> Self {
        Self {
            mode: AppMode::Idle,
            input: InputState::default(),
            calculation: CalculationState::default(),
            progress_history: VecDeque::with_capacity(MAX_PROGRESS_HISTORY),
            results: Vec::new(),
            analysis: ResultAnalysis::default(),
            should_quit: false,
            system_info: SystemInfo::default(),
            calculation_start: None,
            shared_progress: Arc::new(AtomicU64::new(0)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if currently editing a field.
    pub fn is_editing(&self) -> bool {
        self.mode == AppMode::Editing
    }

    /// Toggle help overlay.
    pub fn toggle_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Idle;
        } else {
            self.mode = AppMode::Help;
        }
    }

    /// Move to next input field.
    pub fn next_field(&mut self) {
        self.input.focus = match self.input.focus {
            EditField::N => EditField::Algorithm,
            EditField::Algorithm => EditField::N,
        };
    }

    /// Move to previous input field.
    pub fn prev_field(&mut self) {
        self.next_field(); // Only 2 fields, so same as next
    }

    /// Focus the N input field.
    pub fn focus_n(&mut self) {
        self.input.focus = EditField::N;
        self.mode = AppMode::Editing;
    }

    /// Focus the algorithm selector.
    pub fn focus_algorithm(&mut self) {
        self.input.focus = EditField::Algorithm;
        self.mode = AppMode::Editing;
    }

    /// Start editing the N field.
    pub fn start_edit_n(&mut self) {
        self.input.focus = EditField::N;
        self.input.n_str.clear();
        self.mode = AppMode::Editing;
    }

    /// Cancel current edit.
    pub fn cancel_edit(&mut self) {
        self.mode = AppMode::Idle;
    }

    /// Confirm current edit.
    pub fn confirm_edit(&mut self) {
        self.mode = AppMode::Idle;
    }

    /// Select previous algorithm.
    pub fn select_prev_algorithm(&mut self) {
        let count = Algorithm::all_variants().len();
        if self.input.algorithm_index == 0 {
            self.input.algorithm_index = count - 1;
        } else {
            self.input.algorithm_index -= 1;
        }
    }

    /// Select next algorithm.
    pub fn select_next_algorithm(&mut self) {
        let count = Algorithm::all_variants().len();
        self.input.algorithm_index = (self.input.algorithm_index + 1) % count;
    }

    /// Reset the app to initial state.
    pub fn reset(&mut self) {
        self.mode = AppMode::Idle;
        self.calculation = CalculationState::default();
        self.progress_history.clear();
        self.results.clear();
        self.analysis = ResultAnalysis::default();
    }

    /// Start a calculation.
    pub fn start_calculation(&mut self) {
        let Some(n) = self.input.n() else {
            return; // Invalid input
        };

        self.mode = AppMode::Running;
        self.calculation.status = CalculationStatus::Running;
        self.calculation.progress = 0.0;
        self.calculation.elapsed = Duration::ZERO;
        self.calculation.eta = None;
        self.progress_history.clear();
        self.results.clear();
        self.analysis = ResultAnalysis::default();
        self.calculation_start = Some(Instant::now());
        self.shared_progress.store(0, Ordering::Relaxed);
        self.cancel_flag.store(false, Ordering::Relaxed);

        let algorithm = self.input.algorithm();
        let progress = self.shared_progress.clone();
        let cancel = self.cancel_flag.clone();

        // Spawn calculation thread
        std::thread::spawn(move || {
            run_calculation(n, algorithm, progress, cancel);
        });
    }

    /// Cancel the current calculation.
    pub fn cancel_calculation(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.mode = AppMode::Idle;
        self.calculation.status = CalculationStatus::Idle;
    }

    /// Called on each tick to update state.
    pub fn tick(&mut self) {
        if self.mode != AppMode::Running {
            return;
        }

        // Update elapsed time
        if let Some(start) = self.calculation_start {
            self.calculation.elapsed = start.elapsed();
        }

        // Read progress from shared atomic
        let progress_raw = self.shared_progress.load(Ordering::Relaxed);

        // Check if calculation is complete (progress >= 100 means done, results encoded)
        if progress_raw >= 100 {
            self.complete_calculation(progress_raw);
            return;
        }

        // Update progress (0-99 range during calculation)
        let progress = (progress_raw as f64).min(99.0) / 100.0;
        self.calculation.progress = progress;

        // Add to history for sparkline
        if self.progress_history.len() >= MAX_PROGRESS_HISTORY {
            self.progress_history.pop_front();
        }
        self.progress_history.push_back(progress);

        // Estimate ETA
        if progress > 0.01 {
            let elapsed = self.calculation.elapsed.as_secs_f64();
            let remaining = elapsed / progress * (1.0 - progress);
            self.calculation.eta = Some(Duration::from_secs_f64(remaining));
        }
    }

    fn complete_calculation(&mut self, _progress_raw: u64) {
        self.mode = AppMode::Complete;
        self.calculation.status = CalculationStatus::Complete;
        self.calculation.progress = 1.0;

        // The calculation thread stores results differently based on algorithm
        // For now, we'll compute the result here for display
        if let Some(n) = self.input.n() {
            let algorithm = self.input.algorithm();
            self.populate_results(n, algorithm);
        }
    }

    fn populate_results(&mut self, n: u64, algorithm: Algorithm) {
        use fibrust_core::{
            fibonacci_adaptive, fibonacci_fast_doubling, fibonacci_fft, fibonacci_parallel,
        };

        self.results.clear();

        let result = match algorithm {
            Algorithm::Adaptive => {
                let r = fibonacci_adaptive(n);
                self.results.push(AlgorithmResult {
                    name: "Adaptive".to_string(),
                    duration: self.calculation.elapsed,
                    result: Some(r.clone()),
                    status: AlgorithmStatus::Done,
                });
                r
            }
            Algorithm::FastDoubling => {
                let r = fibonacci_fast_doubling(n);
                self.results.push(AlgorithmResult {
                    name: "Fast Doubling".to_string(),
                    duration: self.calculation.elapsed,
                    result: Some(r.clone()),
                    status: AlgorithmStatus::Done,
                });
                r
            }
            Algorithm::Parallel => {
                let r = fibonacci_parallel(n);
                self.results.push(AlgorithmResult {
                    name: "Parallel".to_string(),
                    duration: self.calculation.elapsed,
                    result: Some(r.clone()),
                    status: AlgorithmStatus::Done,
                });
                r
            }
            Algorithm::Fft => {
                let r = fibonacci_fft(n);
                self.results.push(AlgorithmResult {
                    name: "FFT".to_string(),
                    duration: self.calculation.elapsed,
                    result: Some(r.clone()),
                    status: AlgorithmStatus::Done,
                });
                r
            }
            Algorithm::All => {
                let start = Instant::now();
                let fd = fibonacci_fast_doubling(n);
                let fd_dur = start.elapsed();

                let start = Instant::now();
                let par = fibonacci_parallel(n);
                let par_dur = start.elapsed();

                let start = Instant::now();
                let fft = fibonacci_fft(n);
                let fft_dur = start.elapsed();

                self.results.push(AlgorithmResult {
                    name: "Fast Doubling".to_string(),
                    duration: fd_dur,
                    result: Some(fd.clone()),
                    status: AlgorithmStatus::Done,
                });
                self.results.push(AlgorithmResult {
                    name: "Parallel".to_string(),
                    duration: par_dur,
                    result: Some(par.clone()),
                    status: AlgorithmStatus::Done,
                });
                self.results.push(AlgorithmResult {
                    name: "FFT".to_string(),
                    duration: fft_dur,
                    result: Some(fft.clone()),
                    status: AlgorithmStatus::Done,
                });

                // Sort by duration
                self.results.sort_by(|a, b| a.duration.cmp(&b.duration));

                // Check consistency
                self.analysis.consistent = fd == par && par == fft;

                fd
            }
        };

        // Populate analysis
        self.analysis.binary_bits = result.bit_len();
        let result_str = result.to_string();
        self.analysis.digit_count = result_str.len();

        if result_str.len() > 1 {
            let exp = result_str.len() - 1;
            let mantissa = &result_str[..7.min(result_str.len())];
            self.analysis.scientific = format!(
                "{}.{}e+{}",
                &mantissa[..1],
                &mantissa[1..],
                exp
            );
        } else {
            self.analysis.scientific = result_str.clone();
        }

        if result_str.len() > 40 {
            self.analysis.preview = format!(
                "{}...{{{}}}",
                &result_str[..20],
                result_str.len()
            );
        } else {
            self.analysis.preview = result_str;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the calculation in a background thread.
fn run_calculation(
    n: u64,
    algorithm: Algorithm,
    progress: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
) {
    use fibrust_core::{
        fibonacci_adaptive, fibonacci_fast_doubling, fibonacci_fft, fibonacci_parallel,
    };

    // Simulate progress updates
    let steps = 20;
    for i in 0..steps {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
        progress.store((i * 100 / steps) as u64, Ordering::Relaxed);
    }

    // Run the actual calculation
    let _result = match algorithm {
        Algorithm::Adaptive => fibonacci_adaptive(n),
        Algorithm::FastDoubling => fibonacci_fast_doubling(n),
        Algorithm::Parallel => fibonacci_parallel(n),
        Algorithm::Fft => fibonacci_fft(n),
        Algorithm::All => {
            // Run all algorithms
            let _ = fibonacci_fast_doubling(n);
            let _ = fibonacci_parallel(n);
            fibonacci_fft(n)
        }
    };

    // Signal completion
    progress.store(100, Ordering::Relaxed);
}
