extern crate futures;

use std::env;
use log::*;
use env_logger;

mod app;
mod protos;
mod message_handler;
mod client;
mod collector;
mod scheduler;
mod peer_connection;
mod test_harness;
mod node_state;
mod ga;
mod trace_comparisons;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let args: Vec<String> = env::args().collect();
    let n: u16 = (&args[1]).parse().unwrap();
    let only_subscribe = if &args.len() > &2 {
        match (&args[2]).parse::<u16>() {
            Ok(_) => true,
            Err(_) => false
        }
    } else { false };

    env_logger::Builder::new().parse_default_env().init();

    let app = app::App::new(n, only_subscribe);

    if let Err(error) = runtime.block_on(app.start()) {
        error!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}
