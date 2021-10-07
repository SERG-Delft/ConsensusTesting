docker run `
  --mount type=bind,source=${pwd}/../db/genesis,target="/var/lib/rippled/db" `
  ulamlabs/ripple-regtest