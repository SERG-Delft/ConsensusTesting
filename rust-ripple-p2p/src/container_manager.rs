use std::{env, fs, io, thread};
use std::fmt::format;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::process::Command;
use std::str::from_utf8;
use std::time::Duration;

use itertools::Itertools;
use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

pub fn start_docker_containers(peers: usize, unls: Vec<Vec<usize>>) {
    remove_containers();
    let node_keys = get_node_keys(peers);
    create_configs(peers, &node_keys);
    configure_unls(unls, &node_keys);
    run_nodes(peers);
    thread::sleep(Duration::from_secs(1));
}

fn remove_containers() {
    let leftovers = Command::new("docker").arg("ps")
        .args(["--all", "--quiet"])
        .args(["--filter", "ancestor=mvanmeerten/rippled-boost-cmake"])
        .args(["--filter", "name=validator"])
        .output().unwrap();
    let ids: Vec<&str> = from_utf8(&*leftovers.stdout).unwrap().lines().collect();
    debug!("found following nodes to kill: {:?}", ids);
    Command::new("docker").args(["rm", "-f", "-v"]).args(&ids).output().unwrap();
    debug!("killed all nodes");
}

#[derive(Deserialize)]
struct NodeKeysResult {
    pub result: NodeKeys,
}

#[derive(Deserialize, Debug)]
struct NodeKeys {
    validation_key: String,
    validation_private_key: String,
    validation_public_key: String,
    validation_seed: String,
}

fn get_node_keys(n: usize) -> Vec<NodeKeys> {
    let already_running = Command::new("docker")
        .args(["ps", "--filter", "name=key_generator", "--quiet"])
        .output().unwrap().stdout;
    if already_running.len() == 0 {
        debug!("trying to start key generator");
        start_node_with_options("key_generator", 0, false);
    }
    let keys: Vec<NodeKeys> = (0..n).into_par_iter().map(|i| acquire_keys()).collect();
    debug!("acquired {} node keys", keys.len());
    keys
}

fn acquire_keys() -> NodeKeys {
    let output = Command::new("docker").arg("exec")
        .args(["key_generator", "/bin/sh", "-c"])
        .args(["./rippled/my_build/rippled validation_create"])
        .output().unwrap().stdout;
    let keys = std::str::from_utf8(&output).unwrap();
    let result: NodeKeysResult = serde_json::from_str(&keys).unwrap();
    debug!("acquired keys {:?}", result.result);
    result.result
}

fn create_configs(peers: usize, keys: &Vec<NodeKeys>) {
    let base = read_to_string("..\\config\\rippled.cfg").unwrap();
    (0..peers).into_par_iter().for_each(|i| {
        let path = format!("..\\config\\validator_{}", i);
        fs::create_dir_all(&path).unwrap();
        fs::copy("..\\config\\ledger.json", format!("{}\\ledger.json", path)).unwrap();
        let config = base.replace("{validation_seed}", &keys[i].validation_seed);
        File::create(&format!("{}\\rippled.cfg", path)).unwrap().write(config.as_bytes()).unwrap();
        debug!("created config setup for validator {}", i);
    });
}

fn configure_unls(unls: Vec<Vec<usize>>, keys: &Vec<NodeKeys>) {
    (0..unls.len()).into_par_iter().for_each(|i| {
        let path = format!("..\\config\\validator_{}\\validators.txt", i);
        let mut validators = "[validators]\n".to_owned();
        for node in 0..unls.len() {
            if i != node && unls[i].contains(&node) {
                validators.push_str(&*keys[node].validation_public_key);
                validators.push_str("\n");
            }
        }
        File::create(path).unwrap().write(validators.as_bytes()).unwrap();
        debug!("wrote UNL for validator {}", i)
    });
}

fn run_nodes(peers: usize) {
    (0..peers).into_par_iter().for_each(|i| start_node(i));
}

fn start_node(id: usize) {
    start_node_with_options(&format!("validator_{}", id), id, true);
}

fn start_node_with_options(name: &str, offset: usize, expose_to_network: bool) {
    let mut command = Command::new("docker");
    let mut command = command
        .arg("run")
        .args(["-dit", "--name", name])
        .args(["--mount", &format!("type=bind,source={}/../config/{},target=/.config/ripple", env::current_dir().unwrap().to_str().unwrap(), name)]);
    if expose_to_network {
        command = command
            .args(["--net", "ripple-net"])
            .args(["-p", &format!("{}:6005", 6005 + offset)])
            .args(["-p", &format!("{}:51235", 51235 + offset)])
    }
    command.arg("mvanmeerten/rippled-boost-cmake").output().unwrap();
    debug!("started {}", name);
}
