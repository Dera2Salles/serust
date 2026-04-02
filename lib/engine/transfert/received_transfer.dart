import 'package:equatable/equatable.dart';

class ReceivedTransfer extends Equatable {
  final String id;
  final String fileId;
  final String fileName;
  final int fileSize;
  final String senderUsername;
  final DateTime createdAt;

  const ReceivedTransfer({
    required this.id,
    required this.fileId,
    required this.fileName,
    required this.fileSize,
    required this.senderUsername,
    required this.createdAt,
  });

  factory ReceivedTransfer.fromJson(Map<String, dynamic> json) {
    return ReceivedTransfer(
      id: json['id'].toString(),
      fileId: json['file_id'].toString(),
      fileName: json['file_name'] ?? 'Unknown',
      fileSize: json['file_size'] ?? 0,
      senderUsername: json['sender_username'] ?? 'Unknown',
      createdAt: DateTime.parse(json['created_at']),
    );
  }

  @override
  List<Object?> get props => [
    id,
    fileId,
    fileName,
    fileSize,
    senderUsername,
    createdAt,
  ];
}
