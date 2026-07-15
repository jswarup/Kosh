//-- main.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	std::process::{ Command, Stdio };
use	anyhow::{ Context, Result };
use	clap::Parser;
use	tracing::{ debug, level_filters::LevelFilter };
use	tracing_subscriber::EnvFilter;

//---------------------------------------------------------------------------------------------------------------------------------

/// Kosh:
#[derive( Parser, Debug)]
#[command( author, version, about, long_about = None)]
struct Args
{
    /// Enable verbose logging ( debug messages)
    #[arg( short, long)]
    verbose: bool,
    /// Run unit tests (optionally specify a filter)
    #[arg( long, num_args = 0..=1, default_missing_value = "all" )]
    test: Option< String>,
    /// Enable output prints from tests (nocapture)
    #[arg( short = 'g', long)]
    nocapture: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	setup_logging( verbose: bool) -> Result< ()>
{
    let  	filter = if verbose {
        EnvFilter::builder()
            .with_default_directive( LevelFilter::DEBUG.into())
            .from_env_lossy()
    } else {
        EnvFilter::builder()
            .with_default_directive( LevelFilter::INFO.into())
            .from_env_lossy()
    };
    tracing_subscriber::fmt()
        .with_env_filter( filter)
        .with_target( false)
        .try_init()
        .map_err( |e| anyhow::anyhow!( "Failed to initialize logging: {}", e))?;
    Ok( ())
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	run_tests( filter: &str, nocapture: bool) -> Result< ()>
{
    let  	mut cmd = Command::new( "cargo");
    cmd.arg( "test");
    if filter != "all" {
        cmd.arg( filter);
    }
    if nocapture {
        cmd.arg( "--");
        cmd.arg( "--nocapture");
    }
    cmd.stdout( Stdio::inherit());
    cmd.stderr( Stdio::inherit());
    let  	status = cmd.status().context( "Failed to run cargo test")?;
    if !status.success() {
        anyhow::bail!( "Tests failed with exit code: {:?}", status.code());
    }
    Ok( ())
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	main() -> Result< ()>
{
    let  	args = Args::parse();                                      // Parse command line arguments
    if let  	Some( ref filter) = args.test {
        return run_tests( filter, args.nocapture);
    }
    setup_logging( args.verbose).context( "Setting up logging framework failed")?; // Initialize logging based on verbosity flag
    debug!( "Kosh CLI execution finished successfully");
    Ok( ())
}

//---------------------------------------------------------------------------------------------------------------------------------
