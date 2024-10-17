import grpc
from concurrent import futures
import redis

# 然后导入生成的模块
import word_counter_pb2
import word_counter_pb2_grpc
import os
import time
from grpc_health.v1 import health, health_pb2_grpc, health_pb2

r = redis.Redis(host='redis', port=6379, db=0)

class CounterServicer(word_counter_pb2_grpc.CounterServicer):
    def Count(self, request, context):
        word = request.word
        text_id = request.text_id
        cache_key = f"{text_id}:{word}"

        # 先检查缓存
        cached_value = r.get(cache_key)
        if cached_value is not None:
            return word_counter_pb2.WordCountResponse(count=int(cached_value), status_message="缓存命中")

        # 如果缓存中不存在，则读取文件
        file_path = f"texts/{text_id}.txt"

        # 检查文件是否存在
        if not os.path.exists(file_path):
            return word_counter_pb2.WordCountResponse(count=0, status_message="文件不存在")

        count = 0
        # 使用 buffered read 方法
        with open(file_path, 'r', buffering=1024 * 1024) as f:  # 1MB 的缓冲区
            content = f.read()  # 一次性读取整个文件
            count = content.lower().split().count(word.lower())  # 计算词频

        # 存储到 Redis 中，设置不同的缓存时间
        if count > 0:  
            r.set(cache_key, count, ex=3600)  # 1小时
            return word_counter_pb2.WordCountResponse(count=count, status_message="计算并存储的结果")
        else:
            r.set(cache_key, 0, ex=15)  
            return word_counter_pb2.WordCountResponse(count=0, status_message="词不存在")

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))

    # 注册 WordCountService 服务
    word_counter_pb2_grpc.add_CounterServicer_to_server(CounterServicer(), server)

    # 健康检查服务
    health_servicer = health.HealthServicer()  # 创建健康检查服务
    health_pb2_grpc.add_HealthServicer_to_server(health_servicer, server)

    # 设置健康状态为 SERVING
    health_servicer.set("grpc.health.v1.Health", health_pb2.HealthCheckResponse.SERVING)

    server.add_insecure_port('[::]:50051')
    server.start()
    print("Server started on port 50051.")

    try:
        while True:
            time.sleep(86400)
    except KeyboardInterrupt:
        server.stop(0)

if __name__ == "__main__":
    serve()
