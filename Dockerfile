FROM rust:1.76.0 AS builder

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends clang cmake build-essential protobuf-compiler libprotobuf-dev libssl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

ENTRYPOINT ["/bin/bash", "-c", "./target/release/ldk-node-cashu-mint ${FLAGS}"]
