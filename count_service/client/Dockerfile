# 使用 Python 3.9 slim 版本作为基础镜像
FROM python:3.9-slim

# 设置工作目录
WORKDIR /app

# 将当前目录的内容复制到容器的 /app 目录
COPY count_service/client/ .

# 从上一级的 proto 文件夹复制到容器中
COPY ../../proto /app/proto

# 安装依赖
RUN pip install --no-cache-dir -r requirements.txt

# 运行客户端
CMD ["python", "client.py"]
