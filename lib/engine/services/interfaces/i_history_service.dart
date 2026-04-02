import '../../models/transfer_history_model.dart';

abstract class IHistoryService {
  Future<void> addTransferRecord(TransferHistoryModel record);
  List<TransferHistoryModel> getHistory();
  Stream<List<TransferHistoryModel>> get historyStream;
  Future<void> clearHistory();
  Future<void> updateStatus(String id, String status);
  Future<void> incrementShareCount(String id);
  Future<void> deleteTransfer(String id);
}
