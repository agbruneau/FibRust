#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

use fibcalc_core::calculator::{Calculator, FibCalculator};
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // Use first 4 bytes as n, capped at 50000 for speed
    let n = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
    let n = n % 50_000;

    let calc: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new())));
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default().normalize();

    // Should not panic
    let _ = calc.calculate(&cancel, &observer, 0, n, &opts);
});
