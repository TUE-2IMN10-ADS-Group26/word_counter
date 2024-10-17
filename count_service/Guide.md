# 演示计算词频功能


## 1. 生成protobuf文件

```bash
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto
```


## 2. 启动Docker服务
```bash
docker-compose up --build
```

## 3. 在另一个终端中运行客户端
```bash
python client.py sample 1.txt 1
```
