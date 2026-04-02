import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:arosaina/engine/connection/i.secure.local.datasource.dart';
import 'package:flutter/foundation.dart';
import '../../core/api_client.dart';
import '../models/transfer_update.dart';
import '../models/ftp_entry.dart';
import 'connection_lifecycle_event.dart';
import 'i_ftp_datasource.dart';
import 'ftp_client.dart';

class FtpDataSourceImpl implements IFtpDataSource {
  final ApiClient _apiClient;
  final ISecureLocalDatasource _secureDatasource;
  FtpClient? _ftpClient;
  bool _connected = false;

  final StreamController<TransferUpdate> _transferUpdateController =
      StreamController.broadcast();
  final StreamController<ConnectionLifecycleEvent> _lifecycleController =
      StreamController.broadcast();

  FtpDataSourceImpl(this._apiClient, this._secureDatasource) {
    final uri = Uri.parse(_apiClient.baseUrl);
    _ftpClient = FtpClient(uri.host, port: uri.port > 0 ? uri.port : 8080);
  }

  @override
  Stream<TransferUpdate> get transferUpdates => _transferUpdateController.stream;

  @override
  Stream<ConnectionLifecycleEvent> get connectionLifecycle =>
      _lifecycleController.stream;

  // ── Auth helpers ─────────────────────────────────────────────────────────

  Future<Map<String, String>> _getCreds() async {
    final token = await _secureDatasource.getToken();
    if (token == null) throw Exception('No saved credentials.');
    final creds = jsonDecode(token) as Map<String, dynamic>;
    return {
      'username': creds['username'] as String,
      'password': creds['password'] as String,
    };
  }

  // ── Connection ───────────────────────────────────────────────────────────

  @override
  Future<void> connect() async {
    if (_connected) return;
    try {
      await _ftpClient?.connect();
      final creds = await _getCreds();
      await _ftpClient?.login(creds['username']!, creds['password']!);
      _connected = true;
      _lifecycleController.add(ConnectionLifecycleEvent(
        status: ConnectionLifecycleStatus.connected,
        endpointId: 'ftp_server',
        endpointName: 'FTP Server (${creds['username']})',
      ));
    } catch (e) {
      debugPrint('FTP Connect error: $e');
      _connected = false;
      rethrow;
    }
  }

  @override
  Future<void> disconnect() async {
    await _ftpClient?.disconnect();
    _connected = false;
    _lifecycleController.add(ConnectionLifecycleEvent(
      status: ConnectionLifecycleStatus.disconnected,
      endpointId: 'all',
    ));
  }

  // ── Navigation ───────────────────────────────────────────────────────────

  @override
  Future<String> currentDir() async {
    return await _ftpClient?.pwd() ?? '/';
  }

  @override
  Future<void> changeDir(String path) async {
    await _ftpClient?.cwd(path);
  }

  @override
  Future<void> navigateUp() async {
    await _ftpClient?.cdup();
  }

  // ── Directory listing ────────────────────────────────────────────────────

  @override
  Future<List<FtpEntry>> listDirectory() async {
    if (_ftpClient == null) return [];
    try {
      final raw = await _ftpClient!.listDirectory();
      final entries = <FtpEntry>[];
      for (final line in raw) {
        if (line.trim().isEmpty) continue;
        try {
          entries.add(FtpEntry.fromListLine(line));
        } catch (e) {
          debugPrint('Failed to parse LIST line: "$line" → $e');
        }
      }
      return entries;
    } catch (e) {
      debugPrint('listDirectory error: $e');
      return [];
    }
  }

  // ── CRUD ─────────────────────────────────────────────────────────────────

  @override
  Future<void> mkdir(String name) async {
    if (_ftpClient == null) throw Exception('FTP Client not initialized');
    await _ftpClient!.mkdir(name);
  }

  @override
  Future<void> rmdir(String name) async {
    await _ftpClient?.rmdir(name);
  }

  @override
  Future<void> deleteFile(String name) async {
    await _ftpClient?.deleteFile(name);
  }

  @override
  Future<void> rename(String from, String to) async {
    await _ftpClient?.rename(from, to);
  }

  @override
  Future<int?> getFileSize(String name) async {
    return await _ftpClient?.size(name);
  }

  // ── Transfers ────────────────────────────────────────────────────────────

  @override
  Future<void> uploadFile(File file, {String? originalFileName}) async {
    final fileName = originalFileName ?? file.path.split('/').last;
    final totalBytes = await file.length();
    final fileId = 'upload_${DateTime.now().millisecondsSinceEpoch}';

    _emitTransfer(TransferUpdate(
      id: fileId,
      fileName: fileName,
      bytesTransferred: 0,
      totalBytes: totalBytes,
      status: TransferStatus.inProgress,
      isIncoming: false,
    ));

    try {
      if (_ftpClient == null) throw Exception('FTP Client not initialized');
      await _ftpClient!.uploadFile(fileName, file, onProgress: (sent, total) {
        _emitTransfer(TransferUpdate(
          id: fileId,
          fileName: fileName,
          bytesTransferred: sent,
          totalBytes: total,
          status: TransferStatus.inProgress,
          isIncoming: false,
        ));
      });
      _emitTransfer(TransferUpdate(
        id: fileId,
        fileId: fileName,
        fileName: fileName,
        bytesTransferred: totalBytes,
        totalBytes: totalBytes,
        status: TransferStatus.completed,
        isIncoming: false,
      ));
    } catch (e) {
      debugPrint('FTP upload error: $e');
      _emitTransfer(TransferUpdate(
        id: fileId,
        fileName: fileName,
        bytesTransferred: 0,
        totalBytes: totalBytes,
        status: TransferStatus.failed,
        isIncoming: false,
      ));
      rethrow;
    }
  }

  @override
  Future<void> downloadFile(String remoteFileName, String localPath) async {
    final fileId = 'download_$remoteFileName';
    try {
      _emitTransfer(TransferUpdate(
        id: fileId,
        fileName: remoteFileName,
        bytesTransferred: 0,
        totalBytes: 0,
        status: TransferStatus.inProgress,
        isIncoming: true,
      ));
      await _ftpClient?.downloadFile(
        remoteFileName,
        localPath,
        onProgress: (received, total) {
          _emitTransfer(TransferUpdate(
            id: fileId,
            fileName: remoteFileName,
            bytesTransferred: received,
            totalBytes: total,
            status: TransferStatus.inProgress,
            isIncoming: true,
          ));
        },
      );
      _emitTransfer(TransferUpdate(
        id: fileId,
        fileName: remoteFileName,
        bytesTransferred: 100,
        totalBytes: 100,
        status: TransferStatus.completed,
        isIncoming: true,
      ));
    } catch (e) {
      debugPrint('FTP download error: $e');
      _emitTransfer(TransferUpdate(
        id: fileId,
        fileName: remoteFileName,
        bytesTransferred: 0,
        totalBytes: 0,
        status: TransferStatus.failed,
        isIncoming: true,
      ));
      rethrow;
    }
  }

  // ── Helpers ──────────────────────────────────────────────────────────────

  void _emitTransfer(TransferUpdate update) {
    if (!_transferUpdateController.isClosed) {
      _transferUpdateController.add(update);
    }
  }

  Future<void> dispose() async {
    await _ftpClient?.disconnect();
    await _transferUpdateController.close();
    await _lifecycleController.close();
  }
}
