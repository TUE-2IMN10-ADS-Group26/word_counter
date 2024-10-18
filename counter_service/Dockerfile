FROM rust:latest
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app
COPY counter_service/ .
COPY ../proto /app/proto
ENV PROTO_PATH=/app/proto/word_counter.proto
ENV REDIS__URL=redis://redis:6379

RUN cargo build --release
EXPOSE 50051
CMD ["./target/release/counter_service"]
