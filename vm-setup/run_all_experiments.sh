#! /usr/bin/bash
n=$1
config_folder=$2
bug=$3
configs=("delay-proposal" "delay-rand" "delay-time" "priority-rand" "priority-proposal" "priority-time")

for config in ${configs[@]};
do
	echo "Starting config $config"
	bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_experiment.sh $n $config_folder/$bug/$config.json $bug/$config
done
