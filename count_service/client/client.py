import grpc
import word_count_pb2
import word_count_pb2_grpc
import asyncio

async def run():
    async with grpc.aio.insecure_channel('server:50051') as channel:
        stub = word_count_pb2_grpc.CounterStub(channel)

        # 输入参数
        word = input("Enter the keyword: ")
        file_name = input("Enter the file name (e.g., 1.txt): ")
        phase = input("Enter the phase (1 for phase1, 2 for phase2): ")

        # 构建请求
        request = word_count_pb2.WordCountRequest(word=word, file_name=file_name)
        response = await stub.Count(request)

        print(f"Count: {response.count}, Status: {response.status_message}")

if __name__ == "__main__":
    asyncio.run(run())
