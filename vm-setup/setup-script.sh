#! /usr/bin/bash

#Running experiments on VM
#1. Install docker: 
sudo apt install docker.io -y
#2. Create docker network ripple-net: 
docker network create ripple-net
#3. pull docker image: 
docker pull mvanmeerten/rippled-bug-benchmark:$1
#5. Install rustup: 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
#6. Build dependencies
sudo apt-get update
sudo apt install build-essential checkinstall zlib1g-dev pkg-config libssl-dev -y
#7. Install OpenSSL: 
sudo wget https://www.openssl.org/source/openssl-1.1.1m.tar.gz
sudo tar -xf openssl-1.1.1m.tar.gz
cd openssl-1.1.1m
sudo ./config --prefix=/usr/local/ssl --openssldir=/usr/local/ssl shared zlib
sudo make
sudo make install
echo "/usr/local/ssl/lib" > /etc/ld.so.conf.d/openssl-1.1.1m.conf
sudo ldconfig -v
sudo mv /usr/bin/c_rehash /usr/bin/c_rehash.backup
sudo mv /usr/bin/openssl /usr/bin/openssl.backup
export PATH="/usr/local/ssl/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
#8. Build DisCoTest:
cargo build --release --manifest-path=/root/snap/lxd/current/ConsensusTesting/rust-ripple-p2p/Cargo.toml
#9. Run experiment:
#bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_experiment.sh 10 /root/snap/lxd/current/ConsensusTesting/vm-setup/configs/b3/delay-time.json delay-time