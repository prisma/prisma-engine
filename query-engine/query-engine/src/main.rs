#[macro_use]
extern crate tracing;

use cli::CliCommand;
use error::PrismaError;
use once_cell::sync::Lazy;
use opt::PrismaOpt;
use request_handlers::{PrismaRequest, PrismaResponse, RequestHandler};
use std::{error::Error, process};
use structopt::StructOpt;
use tracing::subscriber;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod cli;
mod context;
mod dmmf;
mod error;
mod exec_loader;
mod opt;
mod request_handlers;
mod server;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LogFormat {
    Text,
    Json,
}

static LOG_FORMAT: Lazy<LogFormat> =
    Lazy::new(|| match std::env::var("RUST_LOG_FORMAT").as_ref().map(|s| s.as_str()) {
        Ok("devel") => LogFormat::Text,
        _ => LogFormat::Json,
    });

pub type PrismaResult<T> = Result<T, PrismaError>;
type AnyError = Box<dyn Error + Send + Sync + 'static>;

#[async_std::main]
async fn main() -> Result<(), AnyError> {
    init_logger()?;
    return main().await.map_err(|err| {
        info!("Encountered error during initialization:");
        err.render_as_json().expect("error rendering");
        process::exit(1)
    });

    async fn main() -> Result<(), PrismaError> {
        let opts = PrismaOpt::from_args();
        match CliCommand::from_opt(&opts)? {
            Some(cmd) => cmd.execute().await?,
            None => {
                set_panic_hook();
                server::listen(opts).await?;
            }
        }
        Ok(())
    }
}

fn init_logger() -> Result<(), AnyError> {
    match *LOG_FORMAT {
        LogFormat::Text => {
            let subscriber = FmtSubscriber::builder()
                .with_env_filter(EnvFilter::from_default_env())
                .finish();

            subscriber::set_global_default(subscriber)?;
        }
        LogFormat::Json => {
            let subscriber = FmtSubscriber::builder()
                .json()
                .with_env_filter(EnvFilter::from_default_env())
                .finish();

            subscriber::set_global_default(subscriber)?;
        }
    }

    Ok(())
}

fn set_panic_hook() {
    match *LOG_FORMAT {
        LogFormat::Text => (),
        LogFormat::Json => {
            std::panic::set_hook(Box::new(|info| {
                let payload = info
                    .payload()
                    .downcast_ref::<String>()
                    .map(Clone::clone)
                    .unwrap_or_else(|| info.payload().downcast_ref::<&str>().unwrap().to_string());

                match info.location() {
                    Some(location) => {
                        tracing::event!(
                            tracing::Level::ERROR,
                            message = "PANIC",
                            reason = payload.as_str(),
                            file = location.file(),
                            line = location.line(),
                            column = location.column(),
                        );
                    }
                    None => {
                        tracing::event!(tracing::Level::ERROR, message = "PANIC", reason = payload.as_str());
                    }
                }

                std::process::exit(255);
            }));
        }
    }
}
