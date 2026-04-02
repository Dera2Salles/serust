import 'package:hive/hive.dart';

part 'transfer_history_model.g.dart';

@HiveType(typeId: 1)
class TransferHistoryModel extends HiveObject {
  @HiveField(0)
  final String id;

  @HiveField(1)
  final String fileName;

  @HiveField(2)
  final int fileSize;

  @HiveField(3)
  final String senderId;

  @HiveField(4)
  final String receiverId;

  @HiveField(5)
  final DateTime timestamp;

  @HiveField(6)
  final String status; // 'completed', 'failed', 'pending'

  @HiveField(7)
  final bool isSent; // true if sent, false if received

  @HiveField(11)
  final String? fileId;

  TransferHistoryModel({
    required this.id,
    this.fileId,
    required this.fileName,
    required this.fileSize,
    required this.senderId,
    required this.receiverId,
    required this.timestamp,
    required this.status,
    required this.isSent,
    this.filePath,
    this.maxShareCount,
    this.shareCount = 0,
  });

  @HiveField(8)
  final String? filePath;

  @HiveField(9)
  final int? maxShareCount;

  @HiveField(10)
  final int shareCount;
}

class TransferHistoryStatus {
  static const String completed = 'completed';
  static const String failed = 'failed';
  static const String pending = 'pending';
}
