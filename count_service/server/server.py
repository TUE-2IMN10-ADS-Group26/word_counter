import grpc
from concurrent import futures
import redis
from proto import word_count_pb2_grpc, word_count_pb2  # 从共享目录引入proto文件
import os
import time
from grpc_health.v1 import health, health_pb2_grpc


r = redis.Redis(host='redis', port=6379, db=0)

class CounterServicer(word_count_pb2_grpc.CounterServicer):
    def Count(self, request, context):
        word = request.word
        text_id = request.text_id
        cache_key = f"{text_id}:{word}"

        # 先检查缓存
        cached_value = r.get(cache_key)
        if cached_value is not None:
            return word_count_pb2.WordCountResponse(count=int(cached_value), status_message="缓存命中")

        # 如果缓存中不存在，则读取文件
        file_path = f"texts/{text_id}.txt"

        # 检查文件是否存在
        if not os.path.exists(file_path):
            return word_count_pb2.WordCountResponse(count=0, status_message="文件不存在")

        count = 0
        with open(file_path, 'r') as f:
            for line in f:
                count += line.lower().split().count(word.lower())  # 逐行读取并计数

        # 存储到 Redis 中，设置不同的缓存时间
        if count > 0:
            r.set(cache_key, count, ex=3600)  # 1小时
            return word_count_pb2.WordCountResponse(count=count, status_message="计算并存储的结果")
        else:
            r.set(cache_key, 0, ex=300)  # 5分钟
            return word_count_pb2.WordCountResponse(count=0, status_message="词不存在")

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))

    # 注册 WordCountService 服务
    word_count_pb2_grpc.add_WordCounterServicer_to_server(CounterServicer(), server)

    # 健康检查服务
    health_servicer = health.HealthServicer()  # 创建健康检查服务
    health_pb2_grpc.add_HealthServicer_to_server(health_servicer, server)

    # 设置健康状态为 SERVING
    health_servicer.set("grpc.health.v1.Health", health_pb2_grpc.HealthCheckResponse.SERVING)

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