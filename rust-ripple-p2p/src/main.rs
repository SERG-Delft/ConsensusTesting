#[allow(unused_variables)]
extern crate futures;

use std::{env, fs};
use std::io::BufReader;
use chrono::{Duration, Utc};
use log::*;
use env_logger;
use lazy_static::lazy_static;
use serde_with::{serde_as, DurationSeconds};
use crate::app::SchedulerType;
#[allow(unused_imports)]
use crate::container_manager::{NodeKeys, start_docker_containers};
#[allow(unused_imports)]
use crate::executable_manager::start_executables;
use crate::failure_writer::ConsensusPropertyTypes;

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
mod deserialization;
mod container_manager;
mod executable_manager;
mod consensus_properties;
mod locality;
mod scaling;
mod failure_writer;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

lazy_static! {
    pub static ref CONFIG: Configuration = get_config();
    pub static ref NUM_NODES: usize = CONFIG.num_nodes;
    pub static ref LOG_FOLDER: String = get_log_path();
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let config = CONFIG.clone();

    env_logger::Builder::new().parse_default_env().init();
    debug!("Starting with config: {:?}", config);

    let unls: Vec<Vec<usize>> = get_unls(config.num_nodes, config.unl_type);
    println!("Unls: {:?}", unls);

    let image_name = config.rippled_version;
    let node_keys = start_docker_containers(config.num_nodes, unls, image_name.docker_image_name());
    // let node_keys = get_static_node_keys();
    // let node_keys = start_executables(n, unls);

    let app = app::App::new(config.num_nodes as u16, node_keys);

    if let Err(error) = runtime.block_on(app.start(config.scheduler_type)) {
        error!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}

pub fn get_config() -> Configuration {
    let args: Vec<String> = env::args().collect();
    Configuration::parse_file(&args[1])
    // match (&args[1]).parse() {
    //     Ok(file_name) => Configuration::parse_file(file_name),
    //     Err(_) => Configuration::default() // default config
    // }
}

pub fn get_log_path() -> String {
    let now = Utc::now();
    let date_string = now.format("%FT%H-%M-%S").to_string();
    let log_path = format!("{}\\..\\logs\\{}", env::current_dir().unwrap().to_str().unwrap(), date_string);
    if !std::path::Path::new(&log_path).exists() {
        std::fs::create_dir_all(&log_path).expect("Creating log directory failed");
    }
    log_path
}

/// Configure UNL based on unl_type enum
/// Full: Clique graph UNL
/// Limit: The minimum unl configuration for achieving overlap = 2/5 avg(UNL_u, UNL_v)
/// Buggy: The maximum unl configuration that results in a fork
pub fn get_unls(num_nodes: usize, unl_type: UnlType) -> Vec<Vec<usize>> {
    match unl_type {
        UnlType::Full => {
            vec![(0..num_nodes).collect(); num_nodes]
        }
        UnlType::Limit => {
            let overlap: usize = ((num_nodes as f64) * 0.4).ceil() as usize;
            let isolated_nodes: usize = ((overlap as f64) * 0.4).floor() as usize;
            let g1 = vec![(0..(num_nodes - isolated_nodes)).collect(); num_nodes - overlap];
            let g2 = vec![(0..num_nodes).collect(); overlap - isolated_nodes];
            let g3 = vec![((num_nodes - overlap)..num_nodes).collect(); isolated_nodes];
            [g1, g2, g3].concat()
        }
        UnlType::Buggy => {
            let small_half = ((num_nodes as f64 * 0.5) - 0.5).ceil() as usize;
            let big_half = num_nodes - small_half;
            let overlap: usize = (((small_half as f64) * 0.4) - 0.00001).floor() as usize;
            let big_isolated_unl = big_half - (overlap as f64 * 0.5).ceil() as usize;
            let small_isolated_unl = small_half - (overlap as f64 * 0.5).floor() as usize;
            let g1 = vec![(0..(big_isolated_unl + overlap)).collect(); big_isolated_unl];
            let g2 = vec![(0..num_nodes).collect(); overlap];
            let g3 = vec![((num_nodes - (small_isolated_unl + overlap))..num_nodes).collect(); small_isolated_unl];
            [g1, g2, g3].concat()
        }
    }
}

pub fn get_static_node_keys() -> Vec<NodeKeys> {
    vec![
        NodeKeys {
            validation_key: "".to_string(),
            validation_private_key: "".to_string(),
            validation_public_key: "n9KGGaWqcLWHyitJYXLgtY7XakSz4oaGgRvPMUQ4Vpni8T9rWMy5".to_string(),
            validation_seed: "shEmJgbQaVKZU5hufLJyAtdgBCqW4".to_string()
        },
        NodeKeys {
            validation_key: "".to_string(),
            validation_private_key: "".to_string(),
            validation_public_key: "n9LhhwYhd7MciE3ZZwqwWmk911ERz5xpEFrjWdkPgm87qJRRaFdo".to_string(),
            validation_seed: "sn3Vs66YsbqwQ1etJ5Q2SEXssdr6S".to_string()
        },
        NodeKeys {
            validation_key: "".to_string(),
            validation_private_key: "".to_string(),
            validation_public_key: "n9KQ6C4uJUoKmAqmr1kinbbYAoLZAuwjEsZmKGVdMUu7eQs1nqJc".to_string(),
            validation_seed: "sascZVmiLA4keNfXx1naPbuceeA9q".to_string()
        },
        NodeKeys {
            validation_key: "".to_string(),
            validation_private_key: "".to_string(),
            validation_public_key: "n9LJJUgZ8Jyw6Ea5mX3BPRFw6poPPUGRtN6gxiz3t1bCqZ8qCAvt".to_string(),
            validation_seed: "ssJ6gd6LeBn2AiddUF42W6s6Ud9yR".to_string()
        },
        NodeKeys {
            validation_key: "".to_string(),
            validation_private_key: "".to_string(),
            validation_public_key: "n9MfqaoBG4UFBJdGPy7ir6mZwb8R3RRSuVi79npf3brQ1zf5Jhpt".to_string(),
            validation_seed: "sp1xJMz9K68gU2JbDuSygbiyweTWj".to_string()
        },
    ]
}

#[allow(unused)]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum UnlType {
    Full,
    Limit,
    Buggy,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum RippledVersion {
    Fixed,
    LivenessBug,
}

impl RippledVersion {
    pub fn docker_image_name(&self) -> &'static str {
        match self {
            RippledVersion::Fixed => "rippled-liveness-fix",
            RippledVersion::LivenessBug => "rippled-boost-cmake",
        }
    }

    pub fn termination_condition(&self) -> Option<ConsensusPropertyTypes> {
        match self {
            RippledVersion::Fixed => None,
            RippledVersion::LivenessBug => Some(ConsensusPropertyTypes::Termination)
        }
    }
}

#[serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Configuration {
    num_nodes: usize,
    unl_type: UnlType,
    rippled_version: RippledVersion,
    scheduler_type: SchedulerType,
    #[serde_as(as = "DurationSeconds<i64>")]
    search_budget: Duration,
}

impl Configuration {
    pub fn parse_file(file_name: &str) -> Self {
        let file = match fs::File::open(file_name) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed opening config file, using default config: {}", err);
                return Configuration::default()
            }
        };
        let mut reader = BufReader::new(file);
        serde_json::from_reader(&mut reader).unwrap()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            num_nodes: 5,
            unl_type: UnlType::Full,
            rippled_version: RippledVersion::Fixed,
            scheduler_type: SchedulerType::Delay,
            search_budget: Duration::seconds(3600),
        }
    }
}

#[cfg(test)]
mod config_tests {
    use std::fs::File;
    use std::io::{BufWriter};
    use std::path::Path;
    use crate::{Configuration, get_unls, UnlType};

    const FULL_5_UNL: [[usize; 5]; 5] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 4],
    ];

    #[test]
    fn unl_full_test() {
        let num_nodes = 5;
        let result = get_unls(num_nodes, UnlType::Full);
        assert_eq!(result.as_slice(), FULL_5_UNL);
    }

    #[test]
    fn unl_limit_test() {
        let result = get_unls(5, UnlType::Limit);
        assert_eq!(result, FULL_5_UNL);
        let result = get_unls(7, UnlType::Limit);
        assert_eq!(result, vec![
            vec![0, 1, 2, 3, 4, 5],
            vec![0, 1, 2, 3, 4, 5],
            vec![0, 1, 2, 3, 4, 5],
            vec![0, 1, 2, 3, 4, 5],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![4, 5, 6]
        ]);
        let result = get_unls(8, UnlType::Limit);
        assert_eq!(result, vec![
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![4, 5, 6, 7]
        ]);
    }

    #[test]
    fn unl_buggy_test() {
        let result = get_unls(5, UnlType::Buggy);
        assert_eq!(result, vec![
            vec![0, 1, 2],
            vec![0, 1, 2],
            vec![0, 1, 2],
            vec![3, 4],
            vec![3, 4],
        ]);
        let result = get_unls(7, UnlType::Buggy);
        assert_eq!(result, vec![
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![3, 4, 5, 6],
            vec![3, 4, 5, 6],
            vec![3, 4, 5, 6]
        ]);
        let result = get_unls(8, UnlType::Buggy);
        assert_eq!(result, vec![
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![3, 4, 5, 6, 7],
            vec![3, 4, 5, 6, 7],
            vec![3, 4, 5, 6, 7],
            vec![3, 4, 5, 6, 7]
        ]);
        let result = get_unls(9, UnlType::Buggy);
        assert_eq!(result, vec![
            vec![0, 1, 2, 3, 4],
            vec![0, 1, 2, 3, 4],
            vec![0, 1, 2, 3, 4],
            vec![0, 1, 2, 3, 4],
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            vec![4, 5, 6, 7, 8],
            vec![4, 5, 6, 7, 8],
            vec![4, 5, 6, 7, 8],
            vec![4, 5, 6, 7, 8]
        ]);
    }

    #[test]
    fn write_configuration() {
        let configuration = Configuration {
            num_nodes: 5,
            unl_type: UnlType::Full,
            rippled_version: crate::RippledVersion::Fixed,
            scheduler_type: crate::SchedulerType::Priority,
            search_budget: chrono::Duration::seconds(3600),
        };
        let mut config_writer = BufWriter::new(File::create(Path::new("config_example.json")).expect("Creating config file failed"));
        serde_json::to_writer(&mut config_writer, &configuration).expect("Failed writing to config file");
    }
}
