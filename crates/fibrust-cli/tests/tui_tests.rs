//! TUI tests for FibRust.

use std::time::Duration;

// Note: These tests use the public interface of the TUI module.
// Full integration tests would require a mock terminal backend.

/// Test that InputState correctly parses n values.
#[test]
fn test_input_state_parse_n() {
    use fibrust_cli::tui::state::InputState;

    let mut input = InputState::default();
    assert!(input.n().is_some());
    assert_eq!(input.n().unwrap(), 1_000_000);

    input.n_str = "12345".to_string();
    assert_eq!(input.n().unwrap(), 12345);

    input.n_str = "invalid".to_string();
    assert!(input.n().is_none());

    input.n_str = "".to_string();
    assert!(input.n().is_none());
}

/// Test algorithm selection cycling.
#[test]
fn test_algorithm_selection() {
    use fibrust_cli::tui::state::{Algorithm, InputState};

    let input = InputState::default();
    assert_eq!(input.algorithm(), Algorithm::Adaptive);
    assert_eq!(input.algorithm_index, 0);

    let mut input = InputState::default();
    input.algorithm_index = 1;
    assert_eq!(input.algorithm(), Algorithm::FastDoubling);

    input.algorithm_index = 4;
    assert_eq!(input.algorithm(), Algorithm::All);
}

/// Test Algorithm names and descriptions.
#[test]
fn test_algorithm_metadata() {
    use fibrust_cli::tui::state::Algorithm;

    let variants = Algorithm::all_variants();
    assert_eq!(variants.len(), 5);

    assert_eq!(Algorithm::Adaptive.name(), "Adaptive");
    assert!(!Algorithm::Adaptive.description().is_empty());

    assert_eq!(Algorithm::FastDoubling.name(), "Fast Doubling");
    assert_eq!(Algorithm::Parallel.name(), "Parallel");
    assert_eq!(Algorithm::Fft.name(), "FFT");
    assert_eq!(Algorithm::All.name(), "All (Compare)");
}

/// Test CalculationState default values.
#[test]
fn test_calculation_state_default() {
    use fibrust_cli::tui::state::{CalculationState, CalculationStatus};

    let state = CalculationState::default();
    assert_eq!(state.status, CalculationStatus::Idle);
    assert_eq!(state.progress, 0.0);
    assert_eq!(state.elapsed, Duration::ZERO);
    assert!(state.eta.is_none());
}

/// Test ResultAnalysis default values.
#[test]
fn test_result_analysis_default() {
    use fibrust_cli::tui::state::ResultAnalysis;

    let analysis = ResultAnalysis::default();
    assert_eq!(analysis.binary_bits, 0);
    assert_eq!(analysis.digit_count, 0);
    assert!(analysis.scientific.is_empty());
    assert!(analysis.preview.is_empty());
    assert!(!analysis.consistent);
}

/// Test App state transitions.
#[test]
fn test_app_mode_transitions() {
    use fibrust_cli::tui::app::App;
    use fibrust_cli::tui::state::AppMode;

    let mut app = App::new();
    assert_eq!(app.mode, AppMode::Idle);
    assert!(!app.is_editing());

    // Toggle help
    app.toggle_help();
    assert_eq!(app.mode, AppMode::Help);

    app.toggle_help();
    assert_eq!(app.mode, AppMode::Idle);
}

/// Test field navigation.
#[test]
fn test_field_navigation() {
    use fibrust_cli::tui::app::App;
    use fibrust_cli::tui::state::EditField;

    let mut app = App::new();
    assert_eq!(app.input.focus, EditField::N);

    app.next_field();
    assert_eq!(app.input.focus, EditField::Algorithm);

    app.next_field();
    assert_eq!(app.input.focus, EditField::N);

    app.prev_field();
    assert_eq!(app.input.focus, EditField::Algorithm);
}

/// Test algorithm selection navigation.
#[test]
fn test_algorithm_navigation() {
    use fibrust_cli::tui::app::App;
    use fibrust_cli::tui::state::Algorithm;

    let mut app = App::new();
    assert_eq!(app.input.algorithm_index, 0);

    app.select_next_algorithm();
    assert_eq!(app.input.algorithm_index, 1);

    app.select_next_algorithm();
    assert_eq!(app.input.algorithm_index, 2);

    // Wrap around at the end
    app.input.algorithm_index = 4;
    app.select_next_algorithm();
    assert_eq!(app.input.algorithm_index, 0);

    // Wrap around at the beginning
    app.select_prev_algorithm();
    assert_eq!(app.input.algorithm_index, 4);
}

/// Test app reset.
#[test]
fn test_app_reset() {
    use fibrust_cli::tui::app::App;
    use fibrust_cli::tui::state::AppMode;

    let mut app = App::new();
    app.mode = AppMode::Complete;
    app.input.n_str = "999".to_string();
    app.input.algorithm_index = 3;

    app.reset();
    assert_eq!(app.mode, AppMode::Idle);
    assert!(app.results.is_empty());
    assert!(app.progress_history.is_empty());
}

/// Test edit mode functions.
#[test]
fn test_edit_mode() {
    use fibrust_cli::tui::app::App;
    use fibrust_cli::tui::state::{AppMode, EditField};

    let mut app = App::new();

    // Focus N
    app.focus_n();
    assert_eq!(app.mode, AppMode::Editing);
    assert_eq!(app.input.focus, EditField::N);

    app.cancel_edit();
    assert_eq!(app.mode, AppMode::Idle);

    // Focus Algorithm
    app.focus_algorithm();
    assert_eq!(app.mode, AppMode::Editing);
    assert_eq!(app.input.focus, EditField::Algorithm);

    app.confirm_edit();
    assert_eq!(app.mode, AppMode::Idle);
}

/// Test SystemInfo defaults.
#[test]
fn test_system_info() {
    use fibrust_cli::tui::app::SystemInfo;

    let info = SystemInfo::default();
    assert!(info.cpu_count >= 1);
    assert!(!info.parallelism_threshold.is_empty());
    assert!(!info.fft_threshold.is_empty());
    assert!(!info.version.is_empty());
}
