import 'dart:io';
import '../connection/i_ftp_datasource.dart';
import 'received_transfer.dart';
import 'transfer_tree_node.dart';
import 'i.transfer.repository.dart';

class TransferRepositoryImpl implements ITransferRepository {
  final IFtpDataSource _dataSource;

  TransferRepositoryImpl(this._dataSource);

  @override
  Future<void> sendFile(
    File file, {
    required String receiverId,
    int? maxShareCount,
  }) async {
    // receiverId and maxShareCount are ignored for proper FTP
    await _dataSource.uploadFile(file);
  }

  @override
  Future<List<ReceivedTransfer>> getReceivedTransfers() async {
    final files = await _dataSource.listDirectory();
    return files
        .where((f) => !f.isDirectory)
        .map((f) => ReceivedTransfer(
              id: f.name,
              fileId: f.name,
              fileName: f.name,
              fileSize: 0,
              createdAt: DateTime.now(),
              senderUsername: 'FTP Server',
            ))
        .toList();
  }

  @override
  Future<void> downloadFile(String fileId, String fileName) async {
    final savePath = '/storage/emulated/0/Download/$fileName';
    await _dataSource.downloadFile(fileName, savePath);
  }

  @override
  Future<List<TransferTreeNode>> getTransferTree(String fileId) async {
    return []; // Not supported naturally in basic FTP LIST
  }
}
