FROM python:3.9-slim

# 设置工作目录
WORKDIR /app

# 复制代码
COPY . .

# 安装依赖
RUN pip install --no-cache-dir grpcio grpcio-tools protobuf redis

# 生成 protobuf 文件
RUN python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto

# 启动服务
CMD ["python", "server.py"]
