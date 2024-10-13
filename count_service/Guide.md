# 演示计算词频功能

本文档将指导如何演示 gRPC 服务通过 `text_id` 和 `keyword` 计算单词出现频率的功能。

## 1. 设置环境和服务

### 1.1 启动 Redis 和 gRPC 服务
确保你的 Docker Compose 已正确配置 Redis 和 `count_service`。

使用以下命令启动 Redis 和 `count_service`：
```bash
docker-compose up --build
```

检查容器是否成功启动：
```bash
docker ps -a
```

### 1.2 确保 Redis 中存储了文本
1. 进入 Redis 容器：
```bash
docker exec -it lab-redis-1 redis-cli
```

2. 使用 SET 命令存储一些示例文本：
```bash
SET "1" "this is a sample text for counting the sample word"
SET "2" "another example with different words and counting"
```

3. 验证存储是否成功：
```bash
GET "1"
GET "2"
```


## 2. 演示 gRPC 词频统计服务

### 2.1 发送第一个请求，查询 text_id: 1 中关键词 "sample" 出现的次数：
```bash
grpcurl -plaintext -d '{"keyword": "sample", "text_id": "1"}' localhost:50051 WordCount/CountWords
```

预期返回结果：
```bash
{
  "count": 2
}
```
### 2.2 发送第二个请求，查询 text_id: 2 中关键词 "counting" 出现的次数：
```bash
grpcurl -plaintext -d '{"keyword": "counting", "text_id": "2"}' localhost:50051 WordCount/CountWords
```

预期返回结果：
```bash
{
  "count": 1
}
```

### 2.3 发送第三个请求，查询不存在的 text_id，如 text_id: 3：
```bash
grpcurl -plaintext -d '{"keyword": "sample", "text_id": "3"}' localhost:50051 WordCount/CountWords
```

## 3. 检查日志和容器状态
为了展示服务的正确运行，可以进入 count_service 容器并查看日志：
```bash
docker logs lab-count_service-1
```

