& $PSScriptRoot\Down.ps1

& $PSScriptRoot\ConfigProxyCreator.ps1 @args

For ($i=1; $i -lt $args[0]+1; $i++) {
	$peer_port = 51235 + $i -1
	$ws_port = 6005 + ${i} -1
	docker run `
		-dit --name ripple${i} `
		--net ripple-net `
		-p ${peer_port}:51235 -p ${ws_port}:6005 `
		--mount type=bind,source=${pwd}/../config/validator_${i},target="/.config/ripple" `
		--mount type=bind,source=${pwd}/../db/validator_${i},target="/var/lib/rippled/db" `
		--mount type=bind,source=${pwd}/../logs/validator_${i},target="/var/log/rippled" `
		mvanmeerten/rippled-boost-cmake
}

$peer_port = 51240
$ws_port = 6010
docker run `
  -dit --name rippleproxy `
  --net ripple-net `
  -p ${peer_port}:51235 -p ${ws_port}:6005 `
  --mount type=bind,source=${pwd}/../config/proxy,target="/.config/ripple" `
  --mount type=bind,source=${pwd}/../db/proxy,target="/var/lib/rippled/db" `
  --mount type=bind,source=${pwd}/../logs/proxy,target="/var/log/rippled" `
  mvanmeerten/rippled-proxy