& $PSScriptRoot\Down.ps1 @args

#& $PSScriptRoot\ConfigCreator.ps1 @args

# Run in parallel
if ($args[1] -eq "p") {
	(1..$args[0]) | ForEach-Object -Parallel {
		$peer_port = 51235 + $_ -1
		$ws_port = 6005 + $_ -1
		docker run `
			-dit --name ripple$_ `
			--net ripple-net `
			-p ${peer_port}:51235 -p ${ws_port}:6005 `
			--mount type=bind,source=$using:pwd/../config/validator_$_,target="/.config/ripple" `
			--mount type=bind,source=$using:pwd/../logs/validator_$_,target="/var/log/rippled" `
			mvanmeerten/rippled-boost-cmake
	}
} else { # Run sequentially
	For ($i=1; $i -lt $args[0]+1; $i++) {
		$peer_port = 51235 + $i -1
		$ws_port = 6005 + ${i} -1
		docker run `
			-dit --name ripple${i} `
			--net ripple-net `
			-p ${peer_port}:51235 -p ${ws_port}:6005 `
			--mount type=bind,source=${pwd}/../config/validator_${i},target="/.config/ripple" `
			--mount type=bind,source=${pwd}/../logs/validator_${i},target="/var/log/rippled" `
			mvanmeerten/rippled-boost-cmake
	}
}


