#! /usr/bin/bash
n=$1
config_folder=$2
configs=("delay-proposal-config" "delay-rand-config" "delay-time-config" "priority-rand-config" "priority-proposal-config" "priority-time-config")

for config in ${configs[@]};
do
	bash /root/snap/lxd/current/ConsensusTesting/vm-setup/run_experiment.sh $n $config_folder/$config $config
done
