# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc
import warnings

import word_count_pb2 as word__count__pb2

GRPC_GENERATED_VERSION = '1.66.2'
GRPC_VERSION = grpc.__version__
_version_not_supported = False

try:
    from grpc._utilities import first_version_is_lower
    _version_not_supported = first_version_is_lower(GRPC_VERSION, GRPC_GENERATED_VERSION)
except ImportError:
    _version_not_supported = True

if _version_not_supported:
    raise RuntimeError(
        f'The grpc package installed is at version {GRPC_VERSION},'
        + f' but the generated code in word_count_pb2_grpc.py depends on'
        + f' grpcio>={GRPC_GENERATED_VERSION}.'
        + f' Please upgrade your grpc module to grpcio>={GRPC_GENERATED_VERSION}'
        + f' or downgrade your generated code using grpcio-tools<={GRPC_VERSION}.'
    )


class WordCountStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.CountWords = channel.unary_unary(
                '/WordCount/CountWords',
                request_serializer=word__count__pb2.WordCountRequest.SerializeToString,
                response_deserializer=word__count__pb2.WordCountResponse.FromString,
                _registered_method=True)


class WordCountServicer(object):
    """Missing associated documentation comment in .proto file."""

    def CountWords(self, request, context):
        """定义词频统计服务的接口
        """
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_WordCountServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'CountWords': grpc.unary_unary_rpc_method_handler(
                    servicer.CountWords,
                    request_deserializer=word__count__pb2.WordCountRequest.FromString,
                    response_serializer=word__count__pb2.WordCountResponse.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'WordCount', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))
    server.add_registered_method_handlers('WordCount', rpc_method_handlers)


 # This class is part of an EXPERIMENTAL API.
class WordCount(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def CountWords(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(
            request,
            target,
            '/WordCount/CountWords',
            word__count__pb2.WordCountRequest.SerializeToString,
            word__count__pb2.WordCountResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True)
