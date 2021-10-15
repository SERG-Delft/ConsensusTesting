# Consensus Testing
This repository contains the code for testing Ripple's consensus algorithm.

### How to run the private Ripple network on Windows
##### Requirements
[Docker](https://docs.docker.com/get-started/) \
[Rust](https://www.rust-lang.org/learn/get-started)

##### Run the network
To run the network, follow the steps below.
1. Clone this repository
`git clone https://github.com/SERG-Delft/ConsensusTesting`
2. Change directory to rippled-docker `cd ConsensusTesting/rippled-docker`

##### Without proxy
3. Run the powershell script `.\Run.ps1 [number of nodes (max 5)] [1 for connected network without proxy]` \
`.\Run.ps1 5 1` will run a private network with 5 nodes connected and without proxy

##### With proxy
3. `.\Run.ps1 5` will run a private network with 5 nodes and proxy
4. Change directory to rust-ripple-p2p `cd ../rust-ripple-p2p`
5. Run the proxy `cargo run [number of nodes]`, so to run 5 nodes: `cargo run 5`
