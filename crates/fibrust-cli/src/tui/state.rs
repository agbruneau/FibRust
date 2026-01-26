//! TUI state types and enums.

use ibig::UBig;
use std::time::Duration;

/// Algorithm selection for calculation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Algorithm {
    #[default]
    Adaptive,
    FastDoubling,
    Parallel,
    Fft,
    All,
}

impl Algorithm {
    pub fn all_variants() -> &'static [Algorithm] {
        &[
            Algorithm::Adaptive,
            Algorithm::FastDoubling,
            Algorithm::Parallel,
            Algorithm::Fft,
            Algorithm::All,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Adaptive => "Adaptive",
            Algorithm::FastDoubling => "Fast Doubling",
            Algorithm::Parallel => "Parallel",
            Algorithm::Fft => "FFT",
            Algorithm::All => "All (Compare)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Algorithm::Adaptive => "Auto-selects best algorithm",
            Algorithm::FastDoubling => "O(log n) sequential",
            Algorithm::Parallel => "Parallelized Fast Doubling",
            Algorithm::Fft => "FFT-based multiplication",
            Algorithm::All => "Run all and compare",
        }
    }
}

/// Current status of calculation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CalculationStatus {
    #[default]
    Idle,
    Running,
    Complete,
    Error,
}

/// Status of an individual algorithm run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlgorithmStatus {
    Pending,
    Running,
    Done,
    Error,
}

/// Result of a single algorithm execution.
#[derive(Clone, Debug)]
pub struct AlgorithmResult {
    pub name: String,
    pub duration: Duration,
    pub result: Option<UBig>,
    pub status: AlgorithmStatus,
}

impl AlgorithmResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            duration: Duration::ZERO,
            result: None,
            status: AlgorithmStatus::Pending,
        }
    }
}

/// Input state for the TUI.
#[derive(Clone, Debug)]
pub struct InputState {
    /// The n value as string for editing.
    pub n_str: String,
    /// Selected algorithm index.
    pub algorithm_index: usize,
    /// Current edit field focus.
    pub focus: EditField,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            n_str: "1000000".to_string(),
            algorithm_index: 0,
            focus: EditField::N,
        }
    }
}

impl InputState {
    /// Get n as u64, returns None if invalid.
    pub fn n(&self) -> Option<u64> {
        self.n_str.parse().ok()
    }

    /// Get selected algorithm.
    pub fn algorithm(&self) -> Algorithm {
        Algorithm::all_variants()
            .get(self.algorithm_index)
            .copied()
            .unwrap_or_default()
    }
}

/// Which field is being edited.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum EditField {
    #[default]
    N,
    Algorithm,
}

/// Application mode.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AppMode {
    #[default]
    Idle,
    Editing,
    Running,
    Complete,
    Help,
}

/// Calculation state during execution.
#[derive(Clone, Debug, Default)]
pub struct CalculationState {
    pub status: CalculationStatus,
    pub progress: f64,
    pub elapsed: Duration,
    pub eta: Option<Duration>,
}

/// Analysis of the result.
#[derive(Clone, Debug, Default)]
pub struct ResultAnalysis {
    pub binary_bits: usize,
    pub digit_count: usize,
    pub scientific: String,
    pub preview: String,
    pub consistent: bool,
}
