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
        file_name = request.file_name
        cache_key = f"{word}:{file_name}"

        # 检查缓存
        cached_result = r.get(cache_key)
        if cached_result:
            return word_count_pb2.WordCountResponse(count=int(cached_result), status_message="从缓存中读取的结果")
        
        # 如果缓存中没有结果，读取文件并计算词频
        try:
            with open(f'texts/{file_name}', 'r') as f:
                text = f.read()
                count = text.split().count(word)

            # 存储到redis缓存
            r.set(cache_key, count)
            return word_count_pb2.WordCountResponse(count=count, status_message="计算并存储的结果")
        
        except FileNotFoundError:
            return word_count_pb2.WordCountResponse(count=0, status_message="文件未找到")

def serve():
    # 创建 gRPC 服务
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))

    # 注册词频统计服务
    word_count_pb2_grpc.add_WordCountServicer_to_server(WordCountService(), server)

    # 注册健康检查服务
    health_servicer = health.HealthServicer()
    health_pb2_grpc.add_HealthServicer_to_server(health_servicer, server)

    # 设置健康状态
    health_servicer.set('', health_pb2.HealthCheckResponse.SERVING)

    # 监听端口
    server.add_insecure_port('[::]:50051')
    server.start()

    print("Server is running on port 50051")
    
    try:
        while True:
            time.sleep(60 * 60 * 24)  # 每天休眠一次
    except KeyboardInterrupt:
        print("Server stopping...")
        server.stop(0)

if __name__ == "__main__":
    serve()
