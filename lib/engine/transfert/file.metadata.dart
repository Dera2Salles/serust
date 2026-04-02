import 'dart:convert';

// ignore: depend_on_referenced_packages
import 'package:crypto/crypto.dart';

class TransferMetadata {
  final String senderId;
  final String receiverId;
  final int maxTransferCount;
  final String fileName;
  final String fileHash;
  final int timestamp;

  TransferMetadata({
    required this.senderId,
    required this.receiverId,
    required this.maxTransferCount,
    required this.fileName,
    required this.fileHash,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() => {
    'senderId': senderId,
    'receiverId': receiverId,
    'maxTransferCount': maxTransferCount,
    'fileName': fileName,
    'fileHash': fileHash,
    'timestamp': timestamp,
  };

  factory TransferMetadata.fromJson(Map<String, dynamic> json) {
    return TransferMetadata(
      senderId: json['senderId'],
      receiverId: json['receiverId'],
      maxTransferCount: json['maxTransferCount'],
      fileName: json['fileName'],
      fileHash: json['fileHash'],
      timestamp: json['timestamp'],
    );
  }

  String generateSignature(List<int> sessionKey) {
    final hmac = Hmac(sha256, sessionKey);
    final payload = jsonEncode(toJson());
    return hmac.convert(utf8.encode(payload)).toString();
  }
}
