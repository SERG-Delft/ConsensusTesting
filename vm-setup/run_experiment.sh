#! /usr/bin/bash
n=$1
config=$2
result_folder_name=$3

for ((i=1;i<=$n;i++));
do
	echo "Starting test $i of config: $config"
	/root/snap/lxd/current/ConsensusTesting/rust-ripple-p2p/target/release/rust-ripple-p2p $config $result_folder_name$i
done
