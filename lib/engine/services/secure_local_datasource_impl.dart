import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import '../connection/i.secure.local.datasource.dart';

class SecureLocalDatasourceImpl implements ISecureLocalDatasource {
  final FlutterSecureStorage _storage = const FlutterSecureStorage();
  static const String _tokenKey = 'auth_token';

  @override
  Future<void> saveToken(String token) async {
    await _storage.write(key: _tokenKey, value: token);
  }

  @override
  Future<void> deleteToken() async {
    await _storage.delete(key: _tokenKey);
  }

  @override
  Future<String?> getToken() async {
    return await _storage.read(key: _tokenKey);
  }
}
