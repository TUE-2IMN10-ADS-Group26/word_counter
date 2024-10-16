import grpc
import asyncio
from proto import word_count_pb2, word_count_pb2_grpc  # 从共享目录引入proto文件

async def run():
    async with grpc.aio.insecure_channel('server:50051') as channel:
        stub = word_count_pb2_grpc.CounterStub(channel)
        
        word = input("Enter the keyword: ")
        file_name = input("Enter the file name (e.g., 1.txt): ")
        phase = input("Enter the phase (1 for phase1, 2 for phase2): ")

        request = word_count_pb2.WordCountRequest(word=word, file_name=file_name)
        response = await stub.Count(request)
        
        print(f"Count: {response.count}, Status: {response.status_message}")

if __name__ == "__main__":
    asyncio.run(run())
