# server/Dockerfile
FROM python:3.9-slim

WORKDIR /app

# 复制 proto 文件夹
COPY ../../proto /app/proto

# 复制 server.py 和 texts 文件夹
COPY ./server.py /app/
COPY ./texts /app/texts

RUN pip install --no-cache-dir grpcio grpcio-tools protobuf redis

CMD ["python", "server.py"]
