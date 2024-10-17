import grpc
import asyncio

import word_counter_pb2
import word_counter_pb2_grpc
import argparse  
import nltk
from collections import Counter

nltk.download('words')
from nltk.corpus import words

async def run(word, file_name, phase):
    # 创建gRPC频道
    async with grpc.aio.insecure_channel('server:50051') as channel:
        # 创建客户端
        stub = word_counter_pb2_grpc.CounterStub(channel)
        
        # 创建请求
        request = word_counter_pb2.WordCountRequest(
            word=word,
            file_name=file_name,
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


async def batchRun(file_name, n, phase):
    
    words = get_most_frequent_words(n)
    tasks = []
    for word in words:
        tasks.append(run(word, file_name, phase))
        await asyncio.sleep(0.02)
    results = await asyncio.gather(*tasks)
    for result in results:
        print(result)



def get_most_frequent_words(n):
    # List of all words in the nltk word corpus
    word_list = words.words()
    
    # Create a Counter object to count the frequency of each word
    word_counts = Counter(word_list)
    
    # Get the 'n' most common words
    most_common_words = [word for word, count in word_counts.most_common(n)]
    print("most_common_words: ", most_common_words)
    
    return most_common_words

# asyncio.run(batchRun("test.txt", 30, 1))

if __name__ == "__main__":
    # 解析命令行参数
    args = parse_arguments()

    # 运行客户端
    asyncio.run(run(args.word, args.file_name, args.phase))

