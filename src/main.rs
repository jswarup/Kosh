#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use kosh::Buffer;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

/// Kosh: A starter Rust CLI template configured for WSL development
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args
{
    /// Enable verbose logging (debug messages)
    #[arg(short, long)]
    verbose: bool,
}

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

fn main() -> Result<()>
{
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging based on verbosity flag
    setup_logging(args.verbose).context("Setting up logging framework failed")?;

    debug!("Starting Kosh CLI and demonstrating Buffer usage");

    // Initialize a new Buffer of integers using the custom naming convention
    let mut buffer = Buffer::New(vec![100, 200, 300]);
    info!("Initialized a new Buffer of integers with len: {}", buffer.Len());

    // Print initial elements
    for i in 0..buffer.Len()
    {
        if let Some(val) = buffer.Get(i)
        {
            println!("Buffer element at index {}: {}", i, val.to_string().cyan());
        }
    }

    // Set an element and push a new one
    buffer.Set(1, 250).context("Failed to set buffer element")?;
    buffer.Push(400);
    info!("Modified buffer, new len: {}", buffer.Len());

    println!("{}", "Updated Buffer:".green().bold());
    for i in 0..buffer.Len()
    {
        if let Some(val) = buffer.Get(i)
        {
            println!("  [{}] = {}", i, val.to_string().yellow());
        }
    }

    debug!("Kosh CLI execution finished successfully");
    Ok(())
}
