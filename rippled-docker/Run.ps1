& $PSScriptRoot\Down.ps1

& $PSScriptRoot\ConfigCreator.ps1 @args

For ($i=1; $i -lt $args[0]+1; $i++) {
	$peer_port = 51235 + $i -1
	$ws_port = 6005 + ${i} -1
	docker run `
		-dit --name ripple${i} `
		--net ripple-net `
		-p ${peer_port}:51235 -p ${ws_port}:6005 `
		--mount type=bind,source=${pwd}/../config/validator_${i},target="/.config/ripple" `
		--mount type=bind,source=${pwd}/../db/validator_${i},target="/var/lib/rippled/db" `
		mvanmeerten/rippled-boost-cmake
}