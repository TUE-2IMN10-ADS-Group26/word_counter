FROM rust:latest
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app
COPY counter_client/ .

RUN cargo build --release
