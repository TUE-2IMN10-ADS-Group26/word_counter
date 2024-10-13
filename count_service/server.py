# 实现词频统计功能、Redis 缓存以及 gRPC 服务端。每次客户端请求到来时，服务器会检查缓存，若无缓存则计算词频并将结果缓存。

import grpc
from concurrent import futures
import word_count_pb2
import word_count_pb2_grpc
from grpc_reflection.v1alpha import reflection
import redis

# 创建 Redis 连接
r = redis.Redis(host='redis', port=6379, db=0)  # 使用 Docker 服务名 'redis'

class WordCountServicer(word_count_pb2_grpc.WordCountServicer):
    def CountWords(self, request, context):
        text_id = request.text_id
        keyword = request.keyword
        
        # 从 Redis 获取文本内容
        text = r.get(text_id)
        if text is None:
            context.set_details('Text not found')  # 设置错误消息
            context.set_code(grpc.StatusCode.NOT_FOUND)  # 设置 gRPC 状态码
            return word_count_pb2.WordCountResponse(count=0)  # 或者返回一个特殊的响应
        
        # 计算关键词的出现频率
        decoded_text = text.decode('utf-8')  # 将 Redis 返回的字节数据解码成字符串
        count = decoded_text.split().count(keyword)  # 计算关键词出现的次数

        # 返回结果
        return word_count_pb2.WordCountResponse(count=count)

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    word_count_pb2_grpc.add_WordCountServicer_to_server(WordCountServicer(), server)
    
    # 启用反射
    SERVICE_NAMES = (
        word_count_pb2.DESCRIPTOR.services_by_name['WordCount'].full_name,
        reflection.SERVICE_NAME,
    )
    reflection.enable_server_reflection(SERVICE_NAMES, server)

    server.add_insecure_port('[::]:50051')
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
