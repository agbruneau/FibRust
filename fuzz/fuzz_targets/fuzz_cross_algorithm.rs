#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

use fibcalc_core::calculator::{Calculator, FibCalculator};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // Use first 4 bytes as n, capped at 10000 for speed (3 algorithms)
    let n = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
    let n = n % 10_000;

    let fast: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
    let matrix: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new())));
    let fft: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(FFTBasedCalculator::new())));

    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default().normalize();

    let fast_result = fast.calculate(&cancel, &observer, 0, n, &opts);
    let matrix_result = matrix.calculate(&cancel, &observer, 0, n, &opts);
    let fft_result = fft.calculate(&cancel, &observer, 0, n, &opts);

    match (fast_result, matrix_result, fft_result) {
        (Ok(f), Ok(m), Ok(fft_r)) => {
            assert_eq!(f, m, "FastDoubling != Matrix at n={n}");
            assert_eq!(f, fft_r, "FastDoubling != FFT at n={n}");
        }
        _ => {} // All should succeed, but don't panic on errors
    }
});
