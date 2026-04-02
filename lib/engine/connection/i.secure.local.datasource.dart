abstract class ISecureLocalDatasource {
  Future<void> saveToken(String token);
  Future<void> deleteToken();
  Future<String?> getToken();
}
