use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use std::time::Duration;
use std::{env, fs, thread};

use log::debug;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub fn start_docker_containers(peers: usize, unls: &Vec<Vec<usize>>) -> Vec<NodeKeys> {
    remove_containers("validator");
    let node_keys = get_node_keys(peers);
    create_configs(peers, &node_keys);
    configure_unls(unls, &node_keys);
    run_nodes(peers);
    thread::sleep(Duration::from_secs(5));
    node_keys
}

fn remove_containers(name: &str) {
    let leftovers = Command::new("docker")
        .arg("ps")
        .args(["--all", "--quiet"])
        .args(["--filter", "ancestor=mvanmeerten/rippled-boost-cmake"])
        .args(["--filter", &format!("name={}", name)])
        .output()
        .unwrap();
    let ids: Vec<&str> = from_utf8(&*leftovers.stdout).unwrap().lines().collect();
    debug!("found following nodes to kill: {:?}", ids);
    Command::new("docker")
        .args(["rm", "-f", "-v"])
        .args(&ids)
        .output()
        .unwrap();
    debug!("killed all nodes");
}

#[derive(Deserialize)]
struct NodeKeysResult {
    pub result: NodeKeys,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeKeys {
    pub validation_key: String,
    pub validation_private_key: String,
    pub validation_public_key: String,
    pub validation_seed: String,
}

fn get_node_keys(n: usize) -> Vec<NodeKeys> {
    let already_running = Command::new("docker")
        .args(["ps", "--filter", "name=key_generator", "--quiet"])
        .output()
        .unwrap()
        .stdout;
    if already_running.is_empty() {
        debug!("trying to start key generator");
        remove_containers("key_generator");
        start_node_with_options("key_generator", 0, false);
    }
    let keys: Vec<NodeKeys> = (0..n)
        .into_par_iter()
        .map(|i| {
            let path = format!("./config/validator_{}", i);
            if Path::new(&format!("{}/keys.json", path)).exists() {
                let keys: NodeKeys =
                    serde_json::from_str(&*read_to_string(&format!("{}/keys.json", path)).unwrap())
                        .unwrap();
                keys
            } else {
                let keys = acquire_keys();
                File::create(&format!("{}/keys.json", path))
                    .unwrap()
                    .write_all(serde_json::to_string(&keys).unwrap().as_bytes())
                    .unwrap();
                keys
            }
        })
        .collect();
    debug!("acquired {} node keys", keys.len());
    keys
}

fn acquire_keys() -> NodeKeys {
    let output = Command::new("docker")
        .arg("exec")
        .args(["key_generator", "/bin/sh", "-c"])
        .args(["./rippled/my_build/rippled validation_create"])
        .output()
        .unwrap()
        .stdout;
    let keys = std::str::from_utf8(&output).unwrap();
    let result: NodeKeysResult = serde_json::from_str(keys).unwrap();
    debug!("acquired keys {:?}", result.result);
    result.result
}

fn create_configs(peers: usize, keys: &[NodeKeys]) {
    let base = read_to_string("./config/rippled.cfg").unwrap();
    (0..peers).into_par_iter().for_each(|i| {
        let path = format!("./config/validator_{}", i);
        fs::create_dir_all(&path).unwrap();
        fs::copy("./config/ledger.json", format!("{}/ledger.json", path)).unwrap();
        let config = base.replace("{validation_seed}", &keys[i].validation_seed);
        File::create(&format!("{}/rippled.cfg", path))
            .unwrap()
            .write_all(config.as_bytes())
            .unwrap();
        debug!("created config setup for validator {}", i);
    });
}

fn configure_unls(unls: &Vec<Vec<usize>>, keys: &[NodeKeys]) {
    (0..unls.len()).into_par_iter().for_each(|i| {
        let path = format!("./config/validator_{}/validators.txt", i);
        let mut validators = "[validators]\n".to_owned();
        for (node, key) in keys.iter().enumerate() {
            if i != node && unls[i].contains(&node) {
                validators.push_str(&*key.validation_public_key);
                validators.push('\n');
            }
        }
        File::create(path)
            .unwrap()
            .write_all(validators.as_bytes())
            .unwrap();
        debug!("wrote UNL for validator {}", i)
    });
}

fn run_nodes(peers: usize) {
    (0..peers).into_par_iter().for_each(start_node);
}

fn start_node(id: usize) {
    start_node_with_options(&format!("validator_{}", id), id, true);
}

fn start_node_with_options(name: &str, offset: usize, expose_to_network: bool) {
    let mut command = Command::new("docker");
    let mut command = command
        .arg("run")
        .args(["-dit", "--name", name])
        .args(["--mount", &format!("type=bind,source={}/./config/{},target=/.config/ripple", env::current_dir().unwrap().to_str().unwrap(), name)])
        // .args(["--mount", &format!("type=bind,source={}/../logs/{},target=/var/log/rippled", env::current_dir().unwrap().to_str().unwrap(), name)])
    ;
    if expose_to_network {
        command = command
            .args(["--net", "ripple-net"])
            .args(["--hostname", name])
            .args(["-p", &format!("{}:6005", 6005 + offset)])
            .args(["-p", &format!("{}:51235", 51235 + offset)])
    }
    let result = command
        .arg("mvanmeerten/rippled-boost-cmake")
        .output()
        .unwrap();
    println!("{}", from_utf8(result.stdout.as_slice()).unwrap());
    println!("{}", from_utf8(result.stderr.as_slice()).unwrap());
    debug!("started {}", name);
}
