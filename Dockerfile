FROM golang:1.19 AS toxiproxy
WORKDIR /usr/src
RUN git clone https://github.com/Shopify/toxiproxy.git && cd toxiproxy && make build

FROM rust:1.66 AS byzzfuzz


# FROM debian:bullseye-slim
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
# COPY --from=byzzfuzz /home/rust-ripple-p2p .
COPY serialize/src/deserialization/definitions.json serialize/src/deserialization/definitions.json

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo install --locked tokio-console

WORKDIR /usr/src/byzzfuzz
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/byzzfuzz/target \
    # export CARGO_PROFILE_RELEASE_DEBUG=true && \
    RUSTFLAGS="--cfg tokio_unstable" cargo build --package rust-ripple-p2p --release && \
    cp /usr/src/byzzfuzz/target/release/rust-ripple-p2p /home/rust-ripple-p2p

# RUN apt-get install -y linux-perf && \
#     cp /usr/bin/perf_5.10 /usr/bin/perf_5.15 && \
#     # echo 0 > /proc/sys/kernel/kptr_restrict && \
#     git clone https://github.com/brendangregg/FlameGraph

WORKDIR /home

CMD [ "./rust-ripple-p2p", "-n", "7", "-c", "0", "-d", "0", "-r", "0", "--toxiproxy-path", "./toxiproxy-server" ]
