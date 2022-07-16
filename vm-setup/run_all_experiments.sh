#! /usr/bin/bash
n=$1
config_folder=$2
configs=("delay-proposal" "delay-rand" "delay-time" "priority-rand" "priority-proposal" "priority-time")

for config in ${configs[@]};
do
	bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_experiment.sh $n $config_folder/$config.json $config
done
