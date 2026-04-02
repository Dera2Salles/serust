import 'dart:io';
import 'received_transfer.dart';
import 'transfer_tree_node.dart';

abstract class ITransferRepository {
  Future<void> sendFile(
    File file, {
    required String receiverId,
    int? maxShareCount,
  });

  Future<List<ReceivedTransfer>> getReceivedTransfers();

  Future<void> downloadFile(String fileId, String fileName);

  Future<List<TransferTreeNode>> getTransferTree(String fileId);
}
