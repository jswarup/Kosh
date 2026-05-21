//-- main.rs ----------------------------------------------------------------------------------------------------------------------
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

//---------------------------------------------------------------------------------------------------------------------------------
/// Kosh:

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args
{
    /// Enable verbose logging (debug messages)
    #[arg(short, long)]
    verbose: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

fn setup_logging(verbose: bool) -> Result<()>
{
    let filter = if verbose
    {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy()
    }
    else
    {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy()
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    Ok(())
}

//---------------------------------------------------------------------------------------------------------------------------------

fn main() -> Result<()>
{
    let args = Args::parse();                                                   // Parse command line arguments
    setup_logging(args.verbose).context("Setting up logging framework failed")?;      // Initialize logging based on verbosity flag

    debug!("Starting Kosh CLI and demonstrating Buffer usage");
    kosh::buff::TestBuff();

    info!("Initialized a new Buffer of integers with len: {}", 4);

    println!("Buffer element at index {}: {}", 0.to_string().red(), 4.to_string().cyan());
    debug!("Kosh CLI execution finished successfully");
    return Ok(())
}

//---------------------------------------------------------------------------------------------------------------------------------
