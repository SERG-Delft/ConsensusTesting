docker run `
	-it --name ripple-container1 `
	-p 51235:51235 --mount type=bind,source=${pwd}/../config/validator_1,target="/.config/ripple" `
	mvanmeerten/rippled-boost-cmake