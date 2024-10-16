# client/Dockerfile
FROM python:3.9-slim

WORKDIR /app

# 复制 proto 文件夹
COPY count_service/ .
COPY ../../proto /app/proto
ENV PROTO_PATH=/app/proto/word_counter.proto

# 复制 client.py 文件
COPY ./client.py /app/

RUN pip install --no-cache-dir grpcio grpcio-tools protobuf

CMD ["python", "client.py"]
