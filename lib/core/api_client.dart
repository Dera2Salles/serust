import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';

class ApiClient {
  static const String defaultBaseUrl = 'http://localhost:8080';

  // For Android Emulator, localhost is 10.0.2.2
  // We could use a more dynamic approach or an environment variable later.
  String baseUrl;
  late Dio _dio;

  ApiClient({String? baseUrl}) : baseUrl = baseUrl ?? defaultBaseUrl {
    _initDio();
  }

  void _initDio() {
    _dio = Dio(
      BaseOptions(
        baseUrl: baseUrl,
        connectTimeout: const Duration(seconds: 10),
        receiveTimeout: const Duration(seconds: 30),
      ),
    );

    _dio.interceptors.add(
      LogInterceptor(
        requestBody: true,
        responseBody: true,
        logPrint: (obj) => debugPrint(obj.toString()),
      ),
    );
  }

  void updateBaseUrl(String newBaseUrl) {
    baseUrl = newBaseUrl;
    _dio.options.baseUrl = newBaseUrl;
  }

  Dio get dio => _dio;

  void addAuthInterceptor(String token) {
    _dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) {
          options.headers['Authorization'] = 'Bearer $token';
          return handler.next(options);
        },
      ),
    );
  }
}
