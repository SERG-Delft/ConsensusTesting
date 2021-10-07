docker run `
    -dit `
    --name genesis `
    -p 80:80 `
    --mount type=bind,source=${pwd}/../../config/validator_1,target="/.config/ripple" `
    --mount type=bind,source=${pwd}/../../db/genesis,target="/var/lib/rippled/db" `
    mvanmeerten/genesis:latest
