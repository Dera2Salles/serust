import 'dart:io';
import 'package:arosaina/engine/connection/connection_lifecycle_event.dart';
import 'package:arosaina/engine/connection/i_server_connection_repository.dart';
import 'package:arosaina/engine/connection/i_ftp_datasource.dart';
import 'package:arosaina/engine/models/transfer_update.dart';

class ServerConnectionRepositoryImpl implements IServerConnectionRepository {
  final IFtpDataSource _dataSource;

  ServerConnectionRepositoryImpl(this._dataSource);

  @override
  Stream<ConnectionLifecycleEvent> get connectionLifecycle =>
      _dataSource.connectionLifecycle;
      
  @override
  Stream<TransferUpdate> get transferUpdates => 
      _dataSource.transferUpdates;

  @override
  Future<void> connect() async {
    await _dataSource.connect();
  }

  @override
  Future<void> disconnect() async {
    await _dataSource.disconnect();
  }
  
  @override
  Future<void> sendFile(
    String endpointId,
    File file, {
    String? originalFileName,
    int? maxShareCount,
  }) async {
    await _dataSource.uploadFile(file, originalFileName: originalFileName);
  }
  
  @override
  Future<String?> getLocalIp() async {
    return null;
  }
  
  @override
  int? get activeServerPort => null;
}
