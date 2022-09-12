Running experiments on VM
1. Install docker: apt install docker.io
2. Create docker network ripple-net: docker network create ripple-net
3. pull docker image: docker pull mvanmeerten/rippled-bug-benchmark:[tag]
4. Use PAT to clone repo: git clone --recurse-submodules https://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@github.com/SERG-Delft/ConsensusTesting
5. Install rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
6. 	sudo apt-get update 
	sudo apt install build-essential checkinstall zlib1g-dev pkg-config libssl-dev -y
7. Install OpenSSL: 
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
8. Build DisCoTest:
	Go to directory
	cargo build --release
9. Run experiment:
	bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_experiment.sh 10 /root/snap/lxd/current/ConsensusTesting/vm-setup/configs/b3/delay-time.json delay-time
	
cd /root/snap/lxd/current	
git clone --recurse-submodules https://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@github.com/SERG-Delft/ConsensusTesting
export ERROR_LOG=debug 
/root/snap/lxd/current/ConsensusTesting/rust-ripple-p2p/target/release/rust-ripple-p2p /root/snap/lxd/current/ConsensusTesting/vm-setup/configs/b1/delay-time.json test
bash /root/snap/lxd/current/ConsensusTesting/vm-setup/setup-script.sh [tag]
tmux new -dsb3exp 'bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_all_experiments.sh 10 /root/snap/lxd/current/ConsensusTesting/vm-setup/configs b3'

grep -c -r Agreement2 snap/lxd/current/ConsensusTesting/logs/b2/*/failure_file.txt

find . -name "*.json" -exec rm {} \;
find . -name "*execution.txt" -exec rm {} \;
find /root/snap/lxd/current/ConsensusTesting/logs/b3/ -name "*.json" -exec rm {} \;

export RUST_LOG=debug