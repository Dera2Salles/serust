import 'package:hive_flutter/hive_flutter.dart';
import '../models/user_model.dart';
import '../models/transfer_history_model.dart';

import 'interfaces/i_db_service.dart';

class DbServiceImpl implements IDbService {
  static const String userBoxName = 'userBox';
  static const String historyBoxName = 'historyBox';
  static const String settingsBoxName = 'settingsBox';

  @override
  Future<void> init() async {
    // Adapters will be registered in main.dart or injection.dart after code generation
    // But we can also register them here if we import the generated files
    // For now, we assume adapters are registered before opening boxes

    await Hive.initFlutter();
    await Hive.openBox<UserModel>(userBoxName);
    await Hive.openBox<TransferHistoryModel>(historyBoxName);
    await Hive.openBox(settingsBoxName);
  }

  @override
  Box<UserModel> get userBox => Hive.box<UserModel>(userBoxName);
  @override
  Box<TransferHistoryModel> get historyBox =>
      Hive.box<TransferHistoryModel>(historyBoxName);
  @override
  Box get settingsBox => Hive.box(settingsBoxName);
}
