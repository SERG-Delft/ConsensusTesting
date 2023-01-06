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
mod message_handler;
mod peer_connection;
mod scheduler;
mod specs;
mod toxiproxy;
mod utils;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

#[derive(Parser)]
struct Cli {
    #[clap(long, value_parser)]
    toxiproxy_path: String,
}

struct Args {
    n: usize,
    c: usize,
    d: usize,
    r: usize,
    any_scope: bool,
    baseline: bool,
}

fn main() {
    let cli: Cli = Cli::parse();

    env_logger::Builder::new().parse_default_env().init();

    let table = [
        // baseline
        (7, 0, 0, 0, true, true),
        //  d, c, r, any-scope
        (7, 0, 1, 6, false, false),
        (7, 0, 1, 6, true, false),
        (7, 0, 2, 6, false, false),
        (7, 0, 2, 6, true, false),
        (7, 1, 0, 6, true, false),
        (7, 1, 1, 6, false, false),
        (7, 1, 1, 6, true, false),
        (7, 2, 0, 6, true, false),
        (7, 2, 1, 6, false, false),
        (7, 2, 1, 6, true, false),
    ];

    for (n, d, c, r, any, base) in table {
        let args = Args {
            n,
            c,
            d,
            r,
            any_scope: any,
            baseline: base,
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
                loop {
                    match flags_rx.recv().await {
                        Ok(flag) => flags.push(flag),
                        Err(broadcast::error::RecvError::Closed) => return flags,
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            panic!("flag collector lagged {} messages behind", n)
                        }
                    }
                }
            });

            let (shutdown_tx, shutdown_rx) = broadcast::channel(16);
            let mut results = shutdown_rx.resubscribe();

            let mut toxiproxy = Command::new(&cli.toxiproxy_path)
                .stdout(Stdio::null())
                .spawn()
                .unwrap();

            let node_keys = start_docker_containers(n, &bug_unls);

            let byzz_fuzz = runtime.block_on(ByzzFuzz::new(
                args.n,
                args.c,
                args.d,
                args.r,
                args.any_scope,
                args.baseline,
                node_keys.clone(),
            ));

            println!(
                "d = {}, c = {}, {}",
                args.d,
                args.c,
                if args.baseline {
                    "baseline"
                } else if args.any_scope {
                    "any-scope"
                } else {
                    "small-scope"
                }
            );
            println!("process faults: {:?}", &byzz_fuzz.process_faults);
            println!("network faults: {:?}", &byzz_fuzz.network_faults);

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

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
