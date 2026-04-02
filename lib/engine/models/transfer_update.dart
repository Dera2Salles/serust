enum TransferStatus {
  inProgress,
  completed,
  failed,
  encrypting,
}

class TransferUpdate {
  final String id;
  final String? fileId;
  final String fileName;
  final int bytesTransferred;
  final int totalBytes;
  final TransferStatus status;
  final bool isIncoming;

  TransferUpdate({
    required this.id,
    this.fileId,
    required this.fileName,
    required this.bytesTransferred,
    required this.totalBytes,
    required this.status,
    required this.isIncoming,
  });
}
