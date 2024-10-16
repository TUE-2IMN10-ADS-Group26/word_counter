import grpc
from concurrent import futures
import redis
import asyncio
from proto import word_count_pb2
from proto import word_count_pb2_grpc


class CounterServicer(word_count_pb2_grpc.CounterServicer):
    def __init__(self):
        self.redis_client = redis.Redis(host='redis', port=6379, decode_responses=True)

    async def Count(self, request, context):
        word = request.word
        file_name = request.file_name

        # 从 Redis 中获取结果
        cached_result = self.redis_client.get(f"{file_name}:{word}")
        if cached_result:
            count = int(cached_result)
            status_message = "从缓存中读取"
        else:
            # 如果缓存中没有，计算词频
            count = await self.calculate_word_frequency(file_name, word)
            # 存储结果到 Redis
            self.redis_client.set(f"{file_name}:{word}", count)
            status_message = "计算并存储的结果"

        return word_count_pb2.WordCountResponse(count=count, status_message=status_message)

    async def calculate_word_frequency(self, file_name, word):
        # 这里实现你的计算逻辑
        # 例如，读取文件并计算词频
        count = 0
        try:
            with open(f"texts/{file_name}", 'r') as f:
                text = f.read()
                count = text.split().count(word)
        except Exception as e:
            print(f"Error reading file: {e}")
        return count


async def serve():
    server = grpc.aio.server()
    word_count_pb2_grpc.add_CounterServicer_to_server(CounterServicer(), server)
    server.add_insecure_port('[::]:50051')
    await server.start()
    print("Server is running on port 50051...")
    await server.wait_for_termination()


if __name__ == '__main__':
    asyncio.run(serve())
