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

##### Without proxy
3. Run the powershell script `.\Run.ps1 [n (1-5)] [1 for connected network without proxy]` \
`.\Run.ps1 5 1` will run a private network with 5 nodes connected and without proxy

##### With proxy
3. `.\Run.ps1 5` will run a private network with 5 nodes connected by the proxy
4. Change directory to rust-ripple-p2p `cd ../rust-ripple-p2p`
5. Run the proxy
    - PowerShell: `$Env:RUST_LOG="trace";$Env:OPENSSL_DIR="[path/to/openssl/dir]"; cargo run [n (1-5)]`
    - Other: `RUST_LOG=trace;OPENSSL_DIR=[path/to/openssl/dir] cargo run [n (1-5)]`

### Is the network validating ledgers?
If the network is running as expected, it should be validating ledgers. \
Run the script `.\LastValidatedLedger.ps1 [i (1-5)]` to see the latest ledger that has been validated by the network according to node i.
If the output of the script is similar to the image below, the network is valiating correctly.
![image](https://user-images.githubusercontent.com/9784016/137471993-fbc688db-73e3-4961-8f43-9588f31653ed.png)

If the output is similar to the image below, the node is either not yet synced to the network correctly or the network is not running correctly. The first ledger should be validated after ~5 seconds.
![image](https://user-images.githubusercontent.com/9784016/137471932-06099354-987c-4532-9e8a-5c8beca98eec.png)

