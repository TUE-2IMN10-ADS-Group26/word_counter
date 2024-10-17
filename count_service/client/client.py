import grpc
import asyncio
from count_service.proto_gen import word_counter_pb2_grpc
from count_service.proto_gen import word_counter_pb2  # 从共享目录引入proto文件import asyncio
import argparse  


async def run(word, file_name, phase):
    # 创建gRPC频道
    async with grpc.aio.insecure_channel('server:50051') as channel:
        # 创建客户端
        stub = word_counter_pb2_grpc.WordCounterStub(channel)
        # 读取文本文件
        with open(file_name, 'r') as f:
            text = f.read()
        # 创建请求
        request = word_counter_pb2.WordCountRequest(
            word=word,
            text_id=file_name,
            text=text
        )
        
        # 根据phase的不同，进行不同的处理
        if phase == 1:
            response = await stub.Count(request)
        elif phase == 2:
            # 如果有负载均衡的逻辑，则实现
            response = await stub.LoadBalanceCount(request)
        else:
            print(f"Unknown phase: {phase}")
            return
        print(f"Count: {response.count}, Status: {response.status}")

def parse_arguments():
    # 定义命令行参数
    parser = argparse.ArgumentParser(description="gRPC Client for Word Counting")
    parser.add_argument('word', type=str, help="The word to count")
    parser.add_argument('file_name', type=str, help="The text file name (e.g., 1.txt)")
    parser.add_argument('phase', type=int, choices=[1, 2], help="Phase: 1 for default server, 2 for load balancing")
    return parser.parse_args()

if __name__ == "__main__":
    # 解析命令行参数
    args = parse_arguments()

    # 运行客户端
    asyncio.run(run(args.word, args.file_name, args.phase))