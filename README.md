# Consensus Testing
This repository contains the code for a tool that tests Ripple's consensus algorithm (RCA).
The tool tests RCA for distributed concurrency bugs. It creates schedules
based on an evolutionary algorithm, which are executed by running a test case.
This test case consists of submitting transactions to the network, and waiting
for these transactions to be included in a validated ledger.

## How to run the tool on Windows
### Requirements
[Docker](https://docs.docker.com/get-started/) \
[Rust](https://www.rust-lang.org/learn/get-started) \
[OpenSSL](https://www.openssl.org/source/)  (1.0.1 - 1.1.1) \
Docker containers for the different Rippled versions

### Run the network
To run the network, follow the steps below.
1. Clone this repository
`git clone https://github.com/SERG-Delft/ConsensusTesting`
3. Create the docker network ripple-net `docker network create ripple-net`
4. Change directory to rust-ripple-p2p `cd ../rust-ripple-p2p`
5. Run the tool
    - PowerShell: `$Env:RUST_LOG="error";$Env:OPENSSL_DIR="[path/to/openssl/dir]"; cargo run [path_to_config_file]`
    - Other: `RUST_LOG=error;OPENSSL_DIR=[path/to/openssl/dir] cargo run [path_to_config_file]`

## Mac
Coming soon

## Config
The settings and configurations for the tool can be set in [config.json](rust-ripple-p2p/config.json).
- num_nodes: The number of nodes in the network.
- unl_type: The type of unl configuration [Full, Limit, Buggy].
- rippled_version: The version of rippled to run [Fixed, LivenessBug]
- scheduler_type: The type of scheduler to run [Delay, Priority, RandomDelay, RandomPriority, DelayTraceGraph, PriorityTraceGraph, PredeterminedDelay, DelayLocalityExperiment, PriorityLocalityExperiment, ScalingExperiment, None].
- search_budget: The time in seconds to run the ga for.

### Logs
Logs of a run can be found in the [logs](logs) folder. Each run of the tool creates
a new folder with the current date and time [yyyy-mm-ddThh-mm-ss] in UTC.

## Code Guide

Most of the code can be found in the [rust-ripple-p2p](https://github.com/SERG-Delft/ConsensusTesting/tree/master/rust-ripple-p2p) folder. The application is started in main.rs and enters app.rs from there.
In app.rs the p2p connections to the nodes are made. Code for the handshake and individual message passing can be found in peer_connection.rs.
Other components that are started in app.rs are the scheduler, the genetic algorithm, the collector and the clients. See the [README](rust-ripple-p2p/src/README.md) in rust-ripple-p2p for a more detailed explanation of the code.

#### Graph Edit Distance
The code for calculation of the Graph Edit Distance can be found in the [ged](ged) folder. The [HED](https://www.sciencedirect.com/science/article/abs/pii/S003132031400274X) algorithm used in the comparison of trace graphs is located in [approximate_edit_distance.rs](ged/src/approximate_edit_distance.rs) in the function `approximate_hed_graph_edit_distance`.
Note that the other algorithms are not used and might not work correctly.