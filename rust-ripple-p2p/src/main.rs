extern crate futures;
extern crate core;

use std::env;
use log::*;
use env_logger;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::broadcast;
use crate::byzzfuzz::ByzzFuzz;
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
mod byzzfuzz;
mod toxiproxy;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

fn main() {
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
        // vec![0, 1, 2, 3, 4], // 0
        // vec![0, 1, 2, 3, 4], // 1
        // vec![0, 1, 2, 3, 4], // 2
        // vec![0, 1, 2, 3, 4], // 3
        // vec![2, 3, 4, 5, 6], // 4
        // vec![2, 3, 4, 5, 6], // 5
        // vec![2, 3, 4, 5, 6], // 6


        vec![0, 1, 2, 3, 4, 5], // 0
        vec![0, 1, 2, 3, 4, 5, 6], // 1
        vec![0, 1, 2, 3, 4, 5, 6], // 2
        vec![0, 1, 2, 3, 4, 5, 6], // 3
        vec![0, 1, 2, 3, 4, 5, 6], // 4
        vec![0, 1, 2, 3, 4, 5, 6], // 5
        vec![1, 2, 3, 4, 5, 6], // 6

        // // config 1.5
        // vec![0, 1, 2, 3, 7], // 0
        // vec![0, 1, 2, 3, 7], // 1
        // vec![0, 1, 2, 3, 7], // 2
        // vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // 3
        // vec![3, 4, 5, 6, 8], // 4
        // vec![3, 4, 5, 6, 8], // 5
        // vec![3, 4, 5, 6, 8], // 6
        // vec![0, 1, 2, 3, 7], // 7
        // vec![3, 4, 5, 6, 8], // 8

        // config 2
        // vec![0, 1, 2, 3, 7, 8], // 0
        // vec![0, 1, 2, 3, 7, 8], // 1
        // vec![0, 1, 2, 3, 7, 8], // 2
        // vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // 3
        // vec![3, 4, 5, 6, 7, 8], // 4
        // vec![3, 4, 5, 6, 7, 8], // 5
        // vec![3, 4, 5, 6, 7, 8], // 6
        // vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // 7
        // vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // 8
    ];

    loop {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        let mut toxiproxy = Command::new(
            r"C:\Users\levin\Downloads\toxiproxy-server-windows-amd64.exe"
        ).stderr(Stdio::null()).spawn().unwrap();
        let node_keys = start_docker_containers(n, &bug_unls);
        for k in &node_keys {
            println!("node key {}", k.validation_public_key);
        }

        let byzz_fuzz = ByzzFuzz::new(7, 10, 10, 10, node_keys.clone());
        println!("{:?}", &byzz_fuzz);
        let app = app::App::new(n as u16, only_subscribe, node_keys);

        if let Err(error) = runtime.block_on(app.start(
            byzz_fuzz,
            shutdown_tx,
            shutdown_rx,
        )) {
            error!("Error: {}", error);
        }

        toxiproxy.kill().unwrap();
        runtime.shutdown_timeout(Duration::from_millis(100));
    }
}
