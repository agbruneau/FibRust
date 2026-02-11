//! FibCalc-rs — High-performance Fibonacci calculator.

mod app;
mod config;
mod errors;
mod version;

use anyhow::Result;

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
