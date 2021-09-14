docker run -it --name ripple-container --mount type=bind,source=${pwd}/../config,target="/.config/ripple" mvanmeerten/rippled-boost-cmake
