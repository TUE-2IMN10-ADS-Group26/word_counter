import grpc
import word_count_pb2
import word_count_pb2_grpc

def run():
    # 连接到 gRPC 服务器
    with grpc.insecure_channel('localhost:50051') as channel:
        stub = word_count_pb2_grpc.CounterStub(channel)

        # 输入关键词和文件名
        word = input("Enter the keyword: ")
        file_name = input("Enter the file name (e.g., 1.txt): ")

        # 构造请求
        request = word_count_pb2.WordCountRequest(word=word, file_name=file_name)

        try:
            # 发送请求并获取响应
            response = stub.Count(request)

            # 输出返回结果
            print(f"Keyword count: {response.count}")
            print(f"Status code: {response.status_code}, Status message: {response.status_message}")

        except grpc.RpcError as e:
            print(f"gRPC Error: {e.code()} - {e.details()}")

if __name__ == "__main__":
    run()
