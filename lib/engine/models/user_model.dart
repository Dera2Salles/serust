import 'package:hive/hive.dart';

part 'user_model.g.dart';

@HiveType(typeId: 0)
class UserModel extends HiveObject {
  @HiveField(0)
  final String username;

  @HiveField(1)
  final String email;

  @HiveField(2)
  final String identifier;

  @HiveField(3)
  final String? profilePicPath;

  @HiveField(4)
  final DateTime createdAt;

  @HiveField(5)
  final String? passwordHash;

  UserModel({
    required this.username,
    required this.email,
    required this.identifier,
    this.profilePicPath,
    required this.createdAt,
    this.passwordHash,
  });
}
