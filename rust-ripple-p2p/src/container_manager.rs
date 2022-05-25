#[allow(unused)]
use std::{env, fs, thread};
use std::fs::{create_dir_all, File, read_to_string};
use std::io::Write;
use std::process::Command;
use std::time::Duration;

use log::{debug, error};
use rayon::prelude::*;
use serde::{Deserialize};
use crate::LOG_FOLDER;

#[allow(unused)]
pub fn start_docker_containers(peers: usize, unls: Vec<Vec<usize>>) -> Vec<NodeKeys> {
    remove_containers("validator");
    let node_keys = get_node_keys(peers);
    create_configs(peers, &node_keys);
    configure_unls(unls, &node_keys);
    let folders = create_log_folders(peers);
    run_nodes(peers, folders);
    thread::sleep(Duration::from_secs(3));
    node_keys
}

pub fn remove_containers(name: &str) {
    let leftovers = Command::new("docker").arg("ps")
        .args(["--all", "--quiet"])
        .args(["--filter", "ancestor=mvanmeerten/rippled-boost-cmake"])
        .args(["--filter", &format!("name={}", name)])
        .output().unwrap();
    let ids: Vec<&str> = std::str::from_utf8(&*leftovers.stdout).unwrap().lines().collect();
    debug!("found following nodes to kill: {:?}", ids);
    Command::new("docker").args(["rm", "-f", "-v"]).args(&ids).output().unwrap();
    debug!("killed all nodes");
}

#[derive(Deserialize)]
struct NodeKeysResult {
    pub result: NodeKeys,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeKeys {
    pub validation_key: String,
    pub validation_private_key: String,
    pub validation_public_key: String,
    pub validation_seed: String,
}

pub fn get_node_keys(n: usize) -> Vec<NodeKeys> {
    start_key_generator();
    debug!("acquiring node keys");
    let keys: Vec<NodeKeys> = (0..n).into_par_iter().map(|_| acquire_keys()).collect();
    debug!("acquired {} node keys", keys.len());
    keys
}

pub fn start_key_generator() {
    let already_running = Command::new("docker")
        .args(["ps", "--filter", "name=key_generator", "--quiet"])
        .output().unwrap().stdout;
    if already_running.len() == 0 {
        debug!("trying to start key generator");
        remove_containers("key_generator");
        start_node_with_options("key_generator", 0, false, None);
        thread::sleep(Duration::from_secs(1));
    }
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


pub fn create_configs(peers: usize, keys: &Vec<NodeKeys>) {
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

pub fn configure_unls(unls: Vec<Vec<usize>>, keys: &Vec<NodeKeys>) {
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

pub fn create_log_folders(peers: usize) -> Vec<String> {
    let mut folders = vec![];
    for i in 0..peers {
        let folder_name = format!("{}\\validator_{}", *LOG_FOLDER, i);
        println!("{}", folder_name);
        match create_dir_all(&folder_name) {
            Ok(_) => folders.push(folder_name),
            Err(err) => error!("Could not create log folder, err: {}", err)
        }
    }
    folders
}

#[allow(unused)]
fn run_nodes(peers: usize, log_folders: Vec<String>) {
    (0..peers).into_par_iter().for_each(|i| start_node(i, &log_folders[i]));
}

#[allow(unused)]
fn start_node(id: usize, log_folder: &str) {
    start_node_with_options(&format!("validator_{}", id), id, true, Some(log_folder));
}

fn start_node_with_options(name: &str, offset: usize, expose_to_network: bool, log_folder: Option<&str>) {
    debug!("Starting node: {}, expose_to_network: {}, log_folder: {:?}", name, expose_to_network, log_folder);
    let mut command = Command::new("docker");
    let mut command = command
        .arg("run")
        .args(["-dit", "--name", name])
        .args(["--mount", &format!("type=bind,source={}/../config/{},target=/.config/ripple", env::current_dir().unwrap().to_str().unwrap(), name)]);
    if let Some(folder) = log_folder {
        command = command.args(["--mount", &format!("type=bind,source={},target=/var/log/rippled", folder)]);
    }
    if expose_to_network {
        command = command
            .args(["--net", "ripple-net"])
            .args(["-p", &format!("{}:6005", 6005 + offset)])
            .args(["-p", &format!("{}:51235", 51235 + offset)])
    }
    command.arg("mvanmeerten/rippled-boost-cmake").output().unwrap();
    debug!("started {}", name);
}

pub fn create_account() -> AccountKeys {
    start_key_generator();
    let output = Command::new("docker").arg("exec")
        .args(["key_generator", "/bin/sh", "-c"])
        .args(["./rippled/my_build/rippled wallet_propose"])
        .output().unwrap().stdout;
    let keys = std::str::from_utf8(&output).unwrap();
    let result: AccountKeysResult = serde_json::from_str(&keys).unwrap();
    debug!("acquired account keys {:?}", result.result);
    result.result
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountKeys {
    pub account_id: String,
    pub master_seed: String,
}

#[derive(Deserialize)]
pub struct AccountKeysResult {
    result: AccountKeys
}