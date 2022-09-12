#!/bin/bash
folder=$1
folder1=C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/logs/b1/batch3-new-logic
folder2=C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/logs/b2/batch1
folder3=C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/logs/b3/batch3-new-logic
folders=(${folder1} ${folder2} ${folder3})

#for d in $folder/* ;
for ((i=1;i<=3;i++)); do
	readarray -d '' entries < <(printf '%s\0' ${folders[i-1]}/* | sort -zV)
	for d in "${entries[@]}"; do
		res=$(awk -F' PT' '{print$2}' $d/failure_file.txt | awk -F'.' '{print$1}')
		found=1
		if [ -z "$res" ]
		then
			res=0
			found=0
		fi
		name=$(basename $d)
		out=( $(grep -Eo '[[:digit:]]+|[^[:digit:]]+' <<<"$name") )
		bug=B$i
		echo "${bug},${out[0]},${out[1]},$res,$found"
	done
done
	
#for for ((i=1;i<=$n;i++));
#do
	#echo "Starting test $i of config: $config"
	#/root/snap/lxd/current/ConsensusTesting/rust-ripple-p2p/target/release/rust-ripple-p2p $config $result_folder_name$i
#done
