# 演示计算词频功能


## 1. 安装

```bash
pip install -r server/requirements.txt  # 安装服务端依赖
pip install -r client/requirements.txt  # 安装客户端依赖
```


## 2. 生成 gRPC 代码
1. 
```bash
cd server  # 切换到 server 文件夹
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto
```

2. 
```bash
cd ../client  # 切换到 client 文件夹
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto
```

## 3. 构建Docker 镜像
1. 
```bash
cd ../server  # 切换到 server 文件夹
docker build -t myserver .
```

2. 
```bash
cd ../client  # 切换到 client 文件夹
docker build -t myclient .
```

## 4. 使用 Docker Compose 启动所有服务
```bash
cd ..  # 切换到 Lab 根目录
docker-compose up --build
```

## 5. 运行客户端
```bash
python client.py
```

## 6. 检查 Redis 中的缓存

```bash
docker ps  # 查看正在运行的容器

docker exec -it <redis_container_id> redis-cli

GET "1:sample"  # 查看缓存中 'sample' 关键词在 text_id 为 1 的文本中的计数
```
