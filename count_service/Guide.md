# 演示计算词频功能


## 1. 生成protobuf文件

```bash
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto
```


## 2. 启动protobuf文件
```bash
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. word_count.proto
```

## 3. 启动Docker服务
```bash
docker-compose up --build
```

## 4. 在另一个终端中运行客户端
```bash
docker run -it --rm --network lab_default lab-client
```

## 5. 结果示例
```bash
Enter the file name (e.g., 1.txt): 1.txt
Enter the phase (1 for phase1, 2 for phase2): 1
Count: 2, Status: 计算并存储的结果
```
