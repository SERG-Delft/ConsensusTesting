# Consensus Testing
This repository contains the code for testing Ripple's consensus algorithm.

## How to run a private Ripple network on Windows
### Requirements
[Docker](https://docs.docker.com/get-started/) \
[Rust](https://www.rust-lang.org/learn/get-started) \
[PowerShell](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell-on-windows?view=powershell-7.1) (Should be installed by default on Windows) \
[OpenSSL](https://www.openssl.org/source/)  (1.0.1 - 1.1.1)

### Run the network
To run the network, follow the steps below.
1. Clone this repository
`git clone https://github.com/SERG-Delft/ConsensusTesting`
2. Change directory to rippled-docker `cd ConsensusTesting/rippled-docker`
3. Create the docker network ripple-net `docker network create ripple-net`
   1. `.\Run.ps1 5 [p to start/stop the containers in parallel]` will run a private network with 5 nodes connected by the proxy
4. Change directory to rust-ripple-p2p `cd ../rust-ripple-p2p`
5. Run the proxy
    - PowerShell: `$Env:RUST_LOG="error";$Env:OPENSSL_DIR="[path/to/openssl/dir]"; cargo run [n (1-5)]`
    - Other: `RUST_LOG=trace;OPENSSL_DIR=[path/to/openssl/dir] cargo run [n (1-5)]`

### Is the network validating ledgers?
If the network is running as expected, it should be validating ledgers. \
Run the script `.\LastValidatedLedger.ps1 [i (1-5)]` to see the latest ledger that has been validated by the network according to node i.
If the output of the script is similar to the image below, the network is valiating correctly.
![image](https://user-images.githubusercontent.com/9784016/137471993-fbc688db-73e3-4961-8f43-9588f31653ed.png)

If the output is similar to the image below, the node is either not yet synced to the network correctly or the network is not running correctly. The first ledger should be validated after ~5 seconds.
![image](https://user-images.githubusercontent.com/9784016/137471932-06099354-987c-4532-9e8a-5c8beca98eec.png)

## Code Guide

Most of the code can be found in the [rust-ripple-p2p](https://github.com/SERG-Delft/ConsensusTesting/tree/master/rust-ripple-p2p) folder. The application is started in main.rs and enters app.rs from there.
In app.rs the p2p connections to the nodes are made. Code for the handshake and individual message passing can be found in peer_connection.rs.
Other components that are started in app.rs are the scheduler, the genetic algorithm, the collector and the clients. See the [README](rust-ripple-p2p/src/README.md) in rust-ripple-p2p for a more detailed explanation of the code.

#### Graph Edit Distance
The code for calculation of the Graph Edit Distance can be found in the [ged](ged) folder. The [HED](https://www.sciencedirect.com/science/article/abs/pii/S003132031400274X) algorithm used in the comparison of trace graphs is located in [approximate_edit_distance.rs](ged/src/approximate_edit_distance.rs) in the function `approximate_hed_graph_edit_distance`.
Note that the other algorithms are not used and might not work correctly.