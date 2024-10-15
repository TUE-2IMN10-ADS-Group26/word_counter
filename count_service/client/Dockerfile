# 使用Python 3.9作为基础镜像
FROM python:3.9-slim

# 设置工作目录
WORKDIR /app

# 复制当前目录的内容到容器内
COPY . .

# 安装依赖
RUN pip install --no-cache-dir -r requirements.txt

# 运行client.py
CMD ["python", "client.py"]
