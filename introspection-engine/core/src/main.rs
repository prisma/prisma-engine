pub mod cli;
mod connector_loader;
mod error;
mod rpc;

#[cfg(test)]
mod tests;
use jsonrpc_core::*;
use jsonrpc_stdio_server::ServerBuilder;
use rpc::{Rpc, RpcImpl};

fn main() {
    let matches = cli::clap_app().get_matches();
    init_logger();

    if matches.is_present("version") {
        println!(env!("GIT_HASH"));
    } else {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(RpcImpl::new().to_delegate());

        user_facing_errors::set_panic_hook();

        let mut io_handler = IoHandler::new();
        io_handler.extend_with(RpcImpl::new().to_delegate());

        let server = ServerBuilder::new(io_handler);
        server.build();
    }
}

fn init_logger() {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init()
}
