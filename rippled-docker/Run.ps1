& $PSScriptRoot\Down.ps1

For ($i=1; $i -lt $args[0]+1; $i++) {
	$port = 51235 + $i -1
	docker run `
		-dit --name ripple${i} `
		--net ripple-net `
		-p ${port}:51235 --mount type=bind,source=${pwd}/../config/validator_${i},target="/.config/ripple" `
		mvanmeerten/rippled-boost-cmake
}