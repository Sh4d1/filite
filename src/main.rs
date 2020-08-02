#[cfg(not(any(feature = "fi", feature = "li", feature = "te")))]
compile_error!("You need to select at least one data type");
#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
compile_error!("You need to select at least one database backend");

mod config;
mod db;
mod runtime;
mod utils;

use anyhow::Error;
use config::Config;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// Configuration file to use
    ///
    /// If unspecified, will look for a filite.json
    /// file in the current working directory.
    #[structopt(short, long, name = "FILE")]
    config: Option<PathBuf>,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    /// Initialises the configuration file with default values
    Init,
}

fn main() -> Result<(), Error> {
    let args: Opt = Opt::from_args();
    if let Some(Command::Init) = &args.command {
        return init_config(args.config.as_ref());
    }

    let config = config::read(args.config.unwrap_or_else(|| PathBuf::from("filite.json")))?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let mut runtime = runtime::build(&config)?;
    runtime.block_on(run(config))?;

    Ok(())
}

async fn run(config: &'static Config) -> Result<(), Error> {
    let _pool = db::pool::build(&config).await?;
    Ok(())
}

fn init_config(path: Option<&PathBuf>) -> Result<(), Error> {
    config::write(path.unwrap_or(&PathBuf::from("filite.json")))?;
    println!("Default config written");
    Ok(())
}
