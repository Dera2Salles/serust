import 'dart:async';

import '../models/transfer_history_model.dart';
import 'interfaces/i_db_service.dart';

import 'interfaces/i_history_service.dart';

class HistoryServiceImpl implements IHistoryService {
  final IDbService _dbService;
  final StreamController<List<TransferHistoryModel>> _historyController =
      StreamController<List<TransferHistoryModel>>.broadcast();

  HistoryServiceImpl(this._dbService);

  @override
  Stream<List<TransferHistoryModel>> get historyStream =>
      _historyController.stream;

  void _notifyListeners() {
    _historyController.add(getHistory());
  }

  @override
  Future<void> addTransferRecord(TransferHistoryModel record) async {
    await _dbService.historyBox.add(record);
    _notifyListeners();
  }

  @override
  List<TransferHistoryModel> getHistory() {
    return _dbService.historyBox.values.toList()..sort(
      (a, b) => b.timestamp.compareTo(a.timestamp),
    ); // Sort by newest first
  }

  @override
  Future<void> clearHistory() async {
    await _dbService.historyBox.clear();
    _notifyListeners();
  }

  @override
  Future<void> updateStatus(String id, String status) async {
    // This is a bit inefficient with a list, but fine for local history
    // Ideally we'd use the key if we stored it
    final recordKey = _dbService.historyBox.keys.firstWhere(
      (k) => _dbService.historyBox.get(k)?.id == id,
      orElse: () => null,
    );

    if (recordKey != null) {
      final record = _dbService.historyBox.get(recordKey);
      if (record != null) {
        final updatedRecord = TransferHistoryModel(
          id: record.id,
          fileName: record.fileName,
          fileSize: record.fileSize,
          senderId: record.senderId,
          receiverId: record.receiverId,
          timestamp: record.timestamp,
          status: status,
          isSent: record.isSent,
          filePath: record.filePath,
          maxShareCount: record.maxShareCount,
          shareCount: record.shareCount,
        );
        await _dbService.historyBox.put(recordKey, updatedRecord);
        _notifyListeners();
      }
    }
  }

  @override
  Future<void> incrementShareCount(String id) async {
    final recordKey = _dbService.historyBox.keys.firstWhere(
      (k) => _dbService.historyBox.get(k)?.id == id,
      orElse: () => null,
    );

    if (recordKey != null) {
      final record = _dbService.historyBox.get(recordKey);
      if (record != null) {
        final updatedRecord = TransferHistoryModel(
          id: record.id,
          fileName: record.fileName,
          fileSize: record.fileSize,
          senderId: record.senderId,
          receiverId: record.receiverId,
          timestamp: record.timestamp,
          status: record.status,
          isSent: record.isSent,
          filePath: record.filePath,
          maxShareCount: record.maxShareCount,
          shareCount: record.shareCount + 1,
        );
        await _dbService.historyBox.put(recordKey, updatedRecord);
        _notifyListeners();
      }
    }
  }

  @override
  Future<void> deleteTransfer(String id) async {
    final recordKey = _dbService.historyBox.keys.firstWhere(
      (k) => _dbService.historyBox.get(k)?.id == id,
      orElse: () => null,
    );

    if (recordKey != null) {
      await _dbService.historyBox.delete(recordKey);
      _notifyListeners();
    }
  }

  // Dispose is handled by the DI container or ignored for singletons
  void dispose() {
    _historyController.close();
  }
}
