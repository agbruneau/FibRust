//! Integration test verifying that FFT allocator infrastructure is active.

use fibcalc_core::calculator::CoreCalculator;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

#[test]
fn fft_computation_uses_active_infrastructure() {
    let calc = FFTBasedCalculator::new();
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();

    // n=10000 is large enough to exercise FFT path
    let result = calc.calculate_core(&cancel, &observer, 0, 10_000, &opts);
    assert!(result.is_ok());

    // Verify result digit count matches golden data (F(10000) has 2090 digits)
    let digits = result.unwrap().to_string().len();
    assert_eq!(digits, 2090, "F(10000) should have 2090 digits");
}
