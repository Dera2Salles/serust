import 'package:hive/hive.dart';
import '../../models/user_model.dart';
import '../../models/transfer_history_model.dart';

abstract class IDbService {
  Future<void> init();
  Box<UserModel> get userBox;
  Box<TransferHistoryModel> get historyBox;
  Box get settingsBox;
}
