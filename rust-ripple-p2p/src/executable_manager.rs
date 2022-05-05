use std::{env, fs, thread};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::process::Command;
use std::time::Duration;
use log::debug;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
#[allow(unused_imports)]
use crate::{get_static_node_keys, NodeKeys};
use crate::container_manager::{configure_unls, create_log_folders, get_node_keys, remove_containers};

#[allow(unused)]
const RIPPLED_EXECUTABLE_PATH: &str = "C:\\Users\\Martijn.vanMeerten\\Documents\\rippled\\cmake-build-debug\\rippled.exe";
#[allow(unused)]
const LEDGER_PATH: &str = "C:\\Users\\Martijn.vanMeerten\\Documents\\rippled\\cfg\\ledger.json";
#[allow(unused)]
const DB_BASE_PATH: &str = "C:\\Users\\Martijn.vanMeerten\\Documents\\rippled\\cfg\\db";

#[allow(unused)]
pub fn start_executables(peers: usize, unls: Vec<Vec<usize>>) -> Vec<NodeKeys> {
    kill_executables();
    remove_containers("validator");
    let node_keys = get_node_keys(peers);
    let folders = create_log_folders(peers);
    create_db_folders(peers);
    create_executable_configs(peers, &node_keys, &folders);
    configure_unls(unls, &node_keys);
    run_executable_nodes(peers, folders);
    thread::sleep(Duration::from_secs(1));
    node_keys
}

#[allow(unused)]
fn kill_executables() {
    let output = Command::new("powershell").arg("taskkill")
        .args(["/F", "/IM", "rippled.exe"])
        .output().unwrap();
    let statuses: Vec<&str> = std::str::from_utf8(&*output.stdout).unwrap().lines().collect();
    if statuses.iter().all(|status| status.contains("SUCCESS")) {
        debug!("killed all nodes");
    } else {
        debug!("Unable to kill certain nodes: {:?}", statuses);
    }
}

#[allow(unused)]
fn run_executable_nodes(peers: usize, log_folders: Vec<String>) {
    (0..peers).into_par_iter().for_each(|i| start_executable_node(i, &log_folders[i]));

}

#[allow(unused)]
fn start_executable_node(id: usize, log_folder: &str) {
    let name = &format!("validator_{}", id);
    debug!("Starting node: {}, log_folder: {:?}", name, log_folder);
    Command::new("powershell")
        .arg(RIPPLED_EXECUTABLE_PATH)
        .args(["--ledgerfile", LEDGER_PATH])
        .args(["--conf", &format!("{}\\..\\config\\{}\\rippled.cfg", env::current_dir().unwrap().to_str().unwrap(), name)])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn().unwrap();
    debug!("started {}", name);
}

#[allow(unused)]
pub fn create_executable_configs(peers: usize, keys: &Vec<NodeKeys>, log_folders: &Vec<String>) {
    let base = read_to_string("..\\config\\rippled_exe.cfg").unwrap();
    (0..peers).into_par_iter().for_each(|i| {
        let log_file = format!("{}\\debug.log", &log_folders[i]);
        let path = format!("..\\config\\validator_{}", i);
        fs::create_dir_all(&path).unwrap();
        fs::copy("..\\config\\ledger.json", format!("{}\\ledger.json", path)).unwrap();
        let config = base
            .replace("{peer_port}", &(51235+i).to_string())
            .replace("{ws_port}", &(6005+i).to_string())
            .replace("{validation_seed}", &keys[i].validation_seed)
            .replace("{database_path}", &format!("{}{}", DB_BASE_PATH, i))
            .replace("{log_file}", &log_file);
        File::create(&format!("{}\\rippled.cfg", path)).unwrap().write(config.as_bytes()).unwrap();
        debug!("created config setup for validator {}", i);
    });
}

#[allow(unused)]
fn create_db_folders(peers: usize) {
    (0..peers).into_par_iter().for_each(|i| {
        Command::new("powershell").arg("Remove-Item")
            .args(["-Path", &format!("{}{}", DB_BASE_PATH, i), "-Recurse", "-Force"])
            .output().unwrap();
        Command::new("powershell").arg("New-Item")
            .args(["-Path", &format!("{}{}", DB_BASE_PATH, i), "-ItemType", "Directory"])
            .output().unwrap();
    });
}


