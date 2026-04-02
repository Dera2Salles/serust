import 'dart:convert';
import 'package:crypto/crypto.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:arosaina/engine/connection/i.secure.local.datasource.dart';
import 'package:arosaina/engine/connection/ftp_client.dart';

import '../models/user_model.dart';
import '../../core/api_client.dart';
import 'interfaces/i_auth_service.dart';
import 'interfaces/i_db_service.dart';

class AuthServiceImpl implements IAuthService {
  final IDbService _dbService;
  final ApiClient _apiClient;
  final ISecureLocalDatasource _secureLocalDatasource;

  AuthServiceImpl(
    this._dbService,
    this._apiClient,
    this._secureLocalDatasource,
  );

  @override
  Future<String> generateUserIdentifier(String username, String email) async {
    String deviceFingerprint = await _getDeviceFingerprint();
    final String rawId = "$username:$email:$deviceFingerprint";
    final bytes = utf8.encode(rawId);
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  Future<String> _getDeviceFingerprint() async {
    final DeviceInfoPlugin deviceInfo = DeviceInfoPlugin();
    String fingerprint = "unknown_device";
    try {
      if (Platform.isAndroid) {
        final androidInfo = await deviceInfo.androidInfo;
        fingerprint = androidInfo.id;
      } else if (Platform.isIOS) {
        final iosInfo = await deviceInfo.iosInfo;
        fingerprint = iosInfo.identifierForVendor ?? "ios_device";
      } else if (Platform.isLinux) {
        final linuxInfo = await deviceInfo.linuxInfo;
        fingerprint = linuxInfo.machineId ?? "linux_machine";
      }
    } catch (e) {
      debugPrint("Error getting device info: $e");
    }
    return fingerprint;
  }

  @override
  Future<UserModel> register(
    String username,
    String email,
    String password, {
    String? profilePicPath,
  }) async {
    // FTP server handles users out-of-band and does not support remote registration
    // We mock success in the local database to satisfy the login process
    try {
      final identifier = await generateUserIdentifier(username, email);
      final passwordHash = _hashPassword(password);

      final user = UserModel(
        username: username,
        email: email,
        identifier: identifier,
        profilePicPath: profilePicPath,
        createdAt: DateTime.now(),
        passwordHash: passwordHash,
      );

      await _dbService.userBox.put('currentUser', user);
      
      final creds = jsonEncode({'username': username, 'password': password});
      await _secureLocalDatasource.saveToken(creds);
      
      return user;
    } catch (e) {
      debugPrint('Register error: $e');
      rethrow;
    }
  }

  @override
  UserModel? getCurrentUser() {
    return _dbService.userBox.get('currentUser');
  }

  @override
  Future<bool> login(String email, String password) async {
    try {
      // In this FTP architecture, the "email" field serves as the username (alice, bob)
      final username = email.contains('@') ? email.split('@').first : email;
      
      // Perform a real FTP login check
      final uri = Uri.parse(_apiClient.baseUrl);
      final client = FtpClient(uri.host, port: uri.port.toInt() > 0 ? uri.port : 8080);
      
      await client.connect();
      await client.login(username, password);
      await client.disconnect();

      // IF successful, save credentials locally
      final creds = jsonEncode({'username': username, 'password': password});
      await _secureLocalDatasource.saveToken(creds);

      final identifier = await generateUserIdentifier(username, email);
      final user = UserModel(
        username: username,
        email: email,
        identifier: identifier,
        createdAt: DateTime.now(),
        passwordHash: _hashPassword(password),
      );
      await _dbService.userBox.put('currentUser', user);

      return true;
    } catch (e) {
      debugPrint('FTP Login error: $e');
      return false;
    }
  }

  String _hashPassword(String password) {
    final bytes = utf8.encode(password);
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  @override
  Future<void> updateProfilePicture(String path) async {
    final user = getCurrentUser();
    if (user != null) {
      final updatedUser = UserModel(
        username: user.username,
        email: user.email,
        identifier: user.identifier,
        profilePicPath: path,
        createdAt: user.createdAt,
        passwordHash: user.passwordHash,
      );
      await _dbService.userBox.put('currentUser', updatedUser);
    }
  }

  @override
  bool isLoggedIn() {
    return _dbService.userBox.containsKey('currentUser');
  }

  @override
  Future<void> logout() async {
    await _secureLocalDatasource.deleteToken();
    await _dbService.userBox.delete('currentUser');
  }
}
