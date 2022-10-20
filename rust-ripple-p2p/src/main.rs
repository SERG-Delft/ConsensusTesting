extern crate core;
extern crate futures;

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use bollard::container::{ListContainersOptions, LogOutput, LogsOptions};
use bollard::Docker;
use clap::{crate_version, Parser};
use futures::StreamExt;
use log::*;
use tokio::sync::broadcast;

use crate::byzzfuzz::ByzzFuzz;
use crate::container_manager::start_docker_containers;

mod app;
mod byzzfuzz;
mod client;
mod collector;
mod container_manager;
mod deserialization;
mod message_handler;
mod peer_connection;
mod protos;
mod scheduler;
mod specs;
mod toxiproxy;
mod utils;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

#[derive(Parser, Debug)]
struct Args {
    /// Number of validator nodes
    #[clap(short, value_parser)]
    n: usize,
    /// Bound on the number of rounds with process faults
    #[clap(short, value_parser)]
    c: usize,
    /// Bound on the number of rounds with network faults
    #[clap(short, value_parser)]
    d: usize,
    /// Bound on the number of rounds with faults
    #[clap(short, value_parser)]
    r: usize,
    /// Use any-scope corruptions
    #[clap(long, value_parser)]
    any_scope: bool,
    /// Test baseline algorithm
    #[clap(long, value_parser)]
    baseline: bool,
    /// Path to the toxiproxy executable
    #[clap(long, value_parser)]
    toxiproxy_path: String,
}

fn main() {
    let args: Args = Args::parse();
    // let args: Vec<String> = env::args().collect();
    let n: usize = args.n;
    let toxiproxypath = &args.toxiproxy_path;

    env_logger::Builder::new().parse_default_env().init();

    let configs = [
        // (7, 0, 0, 6, false, true),
        (7, 0, 0, 6, false, false),
        (7, 1, 0, 6, false, false),
        (7, 1, 0, 6, true, false),
        (7, 2, 0, 6, false, false),
        (7, 2, 0, 6, true, false),
        (7, 0, 1, 6, false, false),
        (7, 0, 1, 6, true, false),
        (7, 1, 1, 6, false, false),
        (7, 1, 1, 6, true, false),
        (7, 0, 2, 6, false, false),
        (7, 0, 2, 6, true, false),
        (7, 1, 2, 6, false, false),
        (7, 1, 2, 6, true, false),
    ];
    for (_, c, d, r, any, base) in configs {
        let args = Args {
            n,
            c,
            d,
            r,
            any_scope: any,
            baseline: base,
            toxiproxy_path: args.toxiproxy_path.clone(),
        };
        let bug_unls: Vec<Vec<usize>> = vec![
            // symmetric
            vec![0, 1, 2, 3, 4], // 0
            vec![0, 1, 2, 3, 4], // 1
            vec![0, 1, 2, 3, 4], // 2
            vec![0, 1, 2, 3, 4], // 3
            vec![2, 3, 4, 5, 6], // 4
            vec![2, 3, 4, 5, 6], // 5
            vec![2, 3, 4, 5, 6], // 6

                                 // vec![0, 1, 2, 3, 4, 5], // 0
                                 // vec![0, 1, 2, 3, 4, 5], // 1
                                 // vec![0, 1, 2, 3, 4, 5], // 2
                                 // vec![0, 1, 2, 3, 4, 5], // 3
                                 // vec![1, 2, 3, 4, 5, 6], // 4
                                 // vec![1, 2, 3, 4, 5, 6], // 5
                                 // vec![1, 2, 3, 4, 5, 6], // 6

                                 // vec![0, 1, 2, 3, 4, 5, 6], // 0
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 1
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 2
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 3
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 4
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 5
                                 // vec![0, 1, 2, 3, 4, 5, 6], // 6

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

        let execution_string = format!(
            "buggy-{}-{}-{}-{}-{}-{}",
            args.n,
            args.c,
            args.d,
            args.r,
            if args.baseline {
                "baseline"
            } else if args.any_scope {
                "any-scope"
            } else {
                "small-scope"
            },
            crate_version!()
        );

        let mut file = fs::File::create("results.txt").unwrap();

        {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let (flags_tx, mut flags_rx) = broadcast::channel(32);
            let flag_collector = runtime.spawn(async move {
                let mut flags = Vec::new();
                while let Ok(message) = flags_rx.recv().await {
                    flags.push(message);
                }
                flags
            });

            let (shutdown_tx, shutdown_rx) = broadcast::channel(16);
            let mut results = shutdown_rx.resubscribe();

            let mut toxiproxy = Command::new(toxiproxypath)
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let node_keys = start_docker_containers(n, &bug_unls);
            for k in &node_keys {
                println!("node key {}", k.validation_public_key);
            }

            let byzz_fuzz = ByzzFuzz::new(
                args.n,
                args.c,
                args.d,
                args.r,
                args.any_scope,
                args.baseline,
                node_keys.clone(),
            );
            println!("{:?}", &byzz_fuzz.process_faults);
            file.write_fmt(format_args!(
                "process faults {:?}\n",
                &byzz_fuzz.process_faults
            ))
            .expect("could not log byzzfuzz");
            file.write_fmt(format_args!(
                "network faults {:?}\n",
                &byzz_fuzz.network_faults
            ))
            .expect("could not log byzzfuzz");
            let app = app::App::new(n as u16, node_keys);

            if let Err(error) = runtime.block_on(app.start(byzz_fuzz, shutdown_tx, flags_tx)) {
                error!("Error: {}", error);
            }

            // let check = Command::new("node").arg(r"C:\Users\levin\git\xrp\index.js").output().unwrap();
            let (map, agreed, reason) = runtime.block_on(async { results.recv().await.unwrap() });
            file.write_all(format!("{:?}\n{:?}\nreason: {}\n", map, agreed, reason).as_bytes())
                .expect("could not write");

            let flags = runtime.block_on(flag_collector).unwrap();
            for flag in flags {
                file.write_all(format!("[flag] {}\n", flag).as_bytes())
                    .unwrap();
            }

            file.write_all("done!\n".as_bytes()).unwrap();

            runtime.block_on(save_results(&execution_string, time));

            toxiproxy.kill().unwrap();
            runtime.shutdown_timeout(Duration::from_millis(100));
        }
    }
}

async fn save_results(id: &str, time: u64) {
    let path = format!("traces/{}/{}", id, time);
    fs::create_dir_all(&path).unwrap();
    [
        "execution.txt",
        "subscription_0.json",
        "subscription_1.json",
        "subscription_2.json",
        "subscription_3.json",
        "subscription_4.json",
        "subscription_5.json",
        "subscription_6.json",
        "results.txt",
    ]
    .into_iter()
    .for_each(|file| move_file(&path, file));

    let docker = Docker::connect_with_local_defaults().unwrap();
    let containers = docker
        .list_containers(Some(ListContainersOptions {
            filters: HashMap::from([("name", vec!["validator"])]),
            ..Default::default()
        }))
        .await
        .unwrap();
    for container in containers {
        let container_id = container.id.unwrap();
        let container_names = container.names.unwrap();
        let container_name = container_names.first().unwrap();
        let mut stream = docker.logs(
            container_id.as_str(),
            Some(LogsOptions::<&str> {
                stdout: true,
                ..Default::default()
            }),
        );
        let mut file = fs::File::create(format!("{}{}.txt", &path, container_name)).unwrap();
        while let Some(Ok(log)) = stream.next().await {
            if let LogOutput::Console { message } = log {
                file.write_all(&message).unwrap();
            }
        }
    }
}

#[inline]
fn move_file(path: &String, filename: &str) {
    fs::copy(filename, format!("{}/{}", &path, filename)).unwrap();
}
