import '../../models/user_model.dart';

abstract class IAuthService {
  Future<String> generateUserIdentifier(String username, String email);
  Future<UserModel> register(
    String username,
    String email,
    String password, {
    String? profilePicPath,
  });
  UserModel? getCurrentUser();
  Future<bool> login(String email, String password);
  Future<void> updateProfilePicture(String path);
  bool isLoggedIn();
  Future<void> logout();
}
