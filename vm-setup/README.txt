Running experiments on VM
1. Install docker: sudo apt-get install docker-ce docker-ce-cli containerd.io docker-compose-plugin
2. pull docker image: docker pull mvanmeerten/rippled-bug-benchmark:[tag]
3. Use PAT to clone repo: git clone --recurse-submodules https://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@github.com/SERG-Delft/ConsensusTesting
4. Install rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
5. 	sudo apt-get update 
	sudo apt install build-essential checkinstall zlib1g-dev pkg-config libssl-dev -y
6. Install OpenSSL: 
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
7. Build DisCoTest:
	Go to directory
	cargo build --release