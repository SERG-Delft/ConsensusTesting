extern crate futures;

use std::{env};
use log::*;
use env_logger;
use lazy_static::lazy_static;
use crate::powershell::start_docker_containers;

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
mod powershell;
mod deserialization;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

lazy_static! {
    pub static ref NUM_NODES: usize = get_num_nodes();
}

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

    let correct_unls: Vec<Vec<u16>> = get_full_unls(n);

    let bug_unls: Vec<Vec<u16>> = vec![
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6],
        vec![2, 3, 4, 5, 6]
    ];

    match start_docker_containers(n, correct_unls) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let app = app::App::new(n, only_subscribe);

    if let Err(error) = runtime.block_on(app.start()) {
        error!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}

pub fn get_num_nodes() -> usize {
    let args: Vec<String> = env::args().collect();
    (&args[1]).parse::<usize>().unwrap()
}

pub fn get_full_unls(num_nodes: u16) -> Vec<Vec<u16>> {
    vec![(0..num_nodes).collect(); num_nodes as usize]
}
