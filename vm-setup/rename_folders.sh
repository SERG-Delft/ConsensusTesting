#!/bin/bash
folder=C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/logs/b3/batch4-new-logic
configs=("delay-proposal" "delay-rand" "delay-time")

readarray -d '' entries < <(printf '%s\0' $folder/* | sort -zV)

for ((conf=0;conf<3;conf++));do
	for ((i=19;i>=0;i--)); do
		#echo "${entries[conf*20 + i]}"
		old="${entries[conf*20 + i]}"
		name=$(dirname $old)/${configs[conf]}"$((i+11))"
		echo $old
		echo $name
		mv $old $name
	done
done
	#echo "Starting test $i of config: $config"
	#/root/snap/lxd/current/ConsensusTesting/rust-ripple-p2p/target/release/rust-ripple-p2p $config $result_folder_name$i
#done
