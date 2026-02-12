//! FibCalc-rs — High-performance Fibonacci calculator.

use anyhow::Result;
use fibcalc_lib::{app, config};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    // Parse CLI args and run
    let config = config::AppConfig::parse();
    app::run(&config)
}
