//! cli: Command-line interface entrypoint for the simulator workspace.
//!
//! This binary is currently a scaffold with logging and argument parsing.
//! Future commits will add commands to load data and run deterministic simulations.

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

/// College Football Simulator CLI
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Deterministic, testable college football simulator (scaffold)"
)]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn init_tracing(verbose: u8) {
    // Map -v levels to a filter level string
    let level: Level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    // Allow RUST_LOG to override, default to chosen level
    let default_directive = format!("{}{}", level, "");
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directive));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn main() -> Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbose);

    info!("CLI initialized");
    println!("College Football Sim CLI scaffold ready.");

    Ok(())
}
