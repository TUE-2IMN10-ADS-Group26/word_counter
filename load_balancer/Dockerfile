FROM rust:latest
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app
COPY load_balancer/ .
COPY ../proto /app/proto
ENV PROTO_PATH=/app/proto/word_counter.proto

RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/load_balancer"]
