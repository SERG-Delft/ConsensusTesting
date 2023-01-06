#! /bin/bash

n=${1:-300}

for ((i=0; i<$n; i++))
do
    timeout --foreground 15m \
    docker rm byzzfuzz || true && \
    docker run -v /var/run/docker.sock:/var/run/docker.sock -i --init --net host --name byzzfuzz byzzfuzz && \
    docker cp byzzfuzz:/home/traces . && \
    docker rm byzzfuzz
done
