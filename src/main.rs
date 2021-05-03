mod auth;
mod config;
mod db;
mod reject;
mod routes;
mod runtime;
mod util;

#[cfg(test)]
mod tests;

use anyhow::Error;
use config::Config;
use std::net::IpAddr;
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Reply};

#[derive(StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// Configuration file to use
    #[structopt(
        short,
        long,
        name = "FILE",
        env = "FILITE_CONFIG",
        default_value = "filite.json"
    )]
    config: String,

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
        config::write(&args.config)?;
        println!("Default config written");
        return Ok(());
    }

    let config = config::read(&args.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let db = db::connect(&config.database)?;

    let mut runtime = runtime::build(&config.runtime)?;
    runtime.block_on(serve(
        routes::handler(config, db).with(warp::trace::request()),
        config,
    ));

    Ok(())
}

async fn serve(
    filter: impl Filter<Extract = (impl Reply,)> + Send + Sync + Clone + 'static,
    config: &Config,
) {
    match &config.tls {
        Some(tls_config) => {
            warp::serve(filter)
                .tls()
                .cert_path(&tls_config.cert)
                .key_path(&tls_config.key)
                .run((config.host.parse::<IpAddr>().unwrap(), config.port))
                .await
        }
        None => {
            warp::serve(filter)
                .run((config.host.parse::<IpAddr>().unwrap(), config.port))
                .await
        }
    }
}
