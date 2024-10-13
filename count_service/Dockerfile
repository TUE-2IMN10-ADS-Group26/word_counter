# 使用官方 Python 镜像
FROM python:3.9-slim

# 设置工作目录
WORKDIR /app

# 复制项目文件到工作目录
COPY . /app

# 安装 Python 依赖
RUN pip install --no-cache-dir -r requirements.txt

# 暴露 gRPC 服务端口
EXPOSE 50051

# 启动 gRPC 服务器
CMD ["python", "server.py"]
