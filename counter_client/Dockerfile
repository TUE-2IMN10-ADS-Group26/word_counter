FROM rust:latest
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app
COPY counter_client/ .

ENV WORD_LIST_PATH=/app/src/orchard-street-medium.txt

RUN cargo build --release
