import grpc
import word_count_pb2
import word_count_pb2_grpc

def run():
    # 连接到 gRPC 服务器
    with grpc.insecure_channel('localhost:50051') as channel:
        stub = word_count_pb2_grpc.WordCountStub(channel)
        
        # 向服务器发送请求
        response = stub.CountWords(word_count_pb2.WordCountRequest(keyword='sample', text_id='1'))
        print(f"Keyword found {response.count} times")

if __name__ == '__main__':
    run()
