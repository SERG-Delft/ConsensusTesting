extern crate futures;
extern crate core;

use std::env;
use log::*;
use env_logger;
use crate::container_manager::start_docker_containers;

mod app;
mod protos;
mod message_handler;
mod client;
mod crypto;
mod collector;
mod scheduler;
mod peer_connection;
mod deserialization;
mod container_manager;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let args: Vec<String> = env::args().collect();
    let n: usize = (&args[1]).parse().unwrap();
    let only_subscribe = if &args.len() > &2 {
        match (&args[2]).parse::<u16>() {
            Ok(_) => true,
            Err(_) => false
        }
    } else { false };


    env_logger::Builder::new().parse_default_env().init();

    let bug_unls: Vec<Vec<usize>> = vec![
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6]
    ];

    let node_keys = start_docker_containers(n, bug_unls);

    let app = app::App::new(n as u16, only_subscribe, node_keys);

    if let Err(error) = runtime.block_on(app.start()) {
        error!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}
