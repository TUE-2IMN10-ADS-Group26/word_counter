FROM python:3.9-slim

WORKDIR /app

# 复制 proto 目录
COPY ./proto /app/proto

# 安装依赖
RUN pip install --no-cache-dir -r requirements.txt

EXPOSE 50051

CMD ["python", "server.py"]
