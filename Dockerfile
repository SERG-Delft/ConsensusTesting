FROM golang:1.19 AS toxiproxy
WORKDIR /usr/src
RUN git clone https://github.com/Shopify/toxiproxy.git && cd toxiproxy && make build

FROM rust:1.66 AS byzzfuzz
WORKDIR /usr/src/byzzfuzz
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/byzzfuzz/target \
    cargo build --package rust-ripple-p2p --release && \
    cp /usr/src/byzzfuzz/target/release/rust-ripple-p2p /home

FROM debian:bullseye-slim AS docker-runtime
WORKDIR /home
RUN apt-get update && \
    apt-get install -y ca-certificates curl gnupg lsb-release && \
    mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y docker-ce-cli
COPY ./config ./config
COPY --from=toxiproxy /usr/src/toxiproxy/dist/toxiproxy-server .
COPY --from=byzzfuzz /home/rust-ripple-p2p .
COPY serialize/src/deserialization/definitions.json serialize/src/deserialization/definitions.json

CMD [ "./rust-ripple-p2p", "--toxiproxy-path", "./toxiproxy-server" ]
