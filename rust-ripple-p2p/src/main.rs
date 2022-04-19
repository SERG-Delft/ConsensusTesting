#[allow(unused_variables)]
extern crate futures;

use std::{env};
use log::*;
use env_logger;
use lazy_static::lazy_static;
use crate::container_manager::{NodeKeys, start_docker_containers};

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

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type AnyResult<T> = Result<T, AnyError>;
type EmptyResult = AnyResult<()>;

lazy_static! {
    pub static ref NUM_NODES: usize = get_num_nodes();
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let args: Vec<String> = env::args().collect();
    let n: usize = (&args[1]).parse().unwrap();

    env_logger::Builder::new().parse_default_env().init();

    let unls: Vec<Vec<usize>> = get_unls(n, UnlType::Full);
    println!("Unls: {:?}", unls);

    let node_keys = start_docker_containers(n, unls);
    // let node_keys = get_static_node_keys();

    let app = app::App::new(n as u16, node_keys);

    if let Err(error) = runtime.block_on(app.start()) {
        error!("Error: {}", error);
        std::process::exit(1);
    }

    std::process::exit(0);
}

pub fn get_num_nodes() -> usize {
    let args: Vec<String> = env::args().collect();
    match (&args[1]).parse() {
        Ok(n) => n,
        Err(_) => 5 // default 5 nodes
    }
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
pub enum UnlType {
    Full,
    Limit,
    Buggy,
}

#[cfg(test)]
mod config_tests {
    use crate::{get_unls, UnlType};

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
}
