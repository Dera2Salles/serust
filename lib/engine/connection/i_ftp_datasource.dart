import 'dart:io';
import '../models/transfer_update.dart';
import '../models/ftp_entry.dart';
import 'connection_lifecycle_event.dart';

abstract class IFtpDataSource {
  Stream<TransferUpdate> get transferUpdates;
  Stream<ConnectionLifecycleEvent> get connectionLifecycle;

  Future<void> connect();
  Future<void> disconnect();

  /// Current server-side working directory path
  Future<String> currentDir();
  Future<void> changeDir(String path);
  Future<void> navigateUp();

  Future<List<FtpEntry>> listDirectory();
  Future<void> uploadFile(File file, {String? originalFileName});
  Future<void> downloadFile(String remoteFileName, String localPath);

  Future<void> mkdir(String name);
  Future<void> rmdir(String name);
  Future<void> deleteFile(String name);
  Future<void> rename(String from, String to);
  Future<int?> getFileSize(String name);
}
