#[macro_use]
extern crate log;
#[macro_use]
extern crate rust_embed;

pub mod cli;
pub mod context;
pub mod dmmf;
pub mod error;
pub mod exec_loader;
pub mod opt;
pub mod request_handlers;
pub mod server;

use error::*;
use once_cell::sync::Lazy;
use request_handlers::{PrismaRequest, PrismaResponse, RequestHandler};
use std::error::Error;
use tracing::subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};


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
pub type AnyError = Box<dyn Error + Send + Sync + 'static>;


pub fn init_logger() -> Result<(), AnyError> {
	LogTracer::init()?;

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

pub fn set_panic_hook() -> Result<(), AnyError> {
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

	Ok(())
}