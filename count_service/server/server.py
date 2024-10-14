import grpc
from concurrent import futures
import redis
import word_count_pb2
import word_count_pb2_grpc

# 初始化 Redis 客户端
r = redis.Redis(host='redis', port=6379, db=0)  # 使用服务名称 'redis'

class CounterServicer(word_count_pb2_grpc.CounterServicer):
    def Count(self, request, context):
        file_name = request.file_name
        word = request.word

        # 从本地读取文本文件
        try:
            with open(f"texts/{file_name}", "r", encoding="utf-8") as file:
                text = file.read()
        except FileNotFoundError:
            context.set_code(grpc.StatusCode.NOT_FOUND)
            context.set_details(f"File {file_name} not found")
            return word_count_pb2.WordCountResponse(
                count=0,
                status_code=1,
                status_message="File not found",
                log_id=""
            )

        # 计算关键词的出现次数（忽略大小写）
        count = text.lower().split().count(word.lower())

        # 将结果缓存到 Redis
        r.set(f"{file_name}:{word}", count)

        # 返回计数结果
        return word_count_pb2.WordCountResponse(
            count=count,
            status_code=0,
            status_message="Success",
            log_id=""
        )

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    word_count_pb2_grpc.add_CounterServicer_to_server(CounterServicer(), server)
    server.add_insecure_port('[::]:50051')
    server.start()
    print("Server started on port 50051")
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
