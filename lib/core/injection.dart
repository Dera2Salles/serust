import 'package:arosaina/core/api_client.dart';
import 'package:arosaina/core/permission/i.permission.handler.dart';
import 'package:arosaina/core/permission/permission.handler.impl.dart';
import 'package:arosaina/engine/connection/server_connection_repository_impl.dart';
import 'package:arosaina/engine/connection/i_server_connection_repository.dart';
import 'package:arosaina/engine/connection/ftp_datasource_impl.dart';
import 'package:arosaina/engine/connection/i_ftp_datasource.dart';
import 'package:arosaina/engine/connection/i.secure.local.datasource.dart';
import 'package:arosaina/engine/services/secure_local_datasource_impl.dart';
import 'package:arosaina/engine/models/transfer_history_model.dart';
import 'package:arosaina/engine/models/user_model.dart';
import 'package:arosaina/engine/services/auth_service_impl.dart';
import 'package:arosaina/engine/services/db_service_impl.dart';
import 'package:arosaina/engine/services/history_service_impl.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/engine/services/interfaces/i_db_service.dart';
import 'package:arosaina/engine/services/interfaces/i_history_service.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';
import 'package:arosaina/engine/services/settings_service_impl.dart';
import 'package:arosaina/engine/transfert/i.transfer.repository.dart';
import 'package:arosaina/engine/transfert/transfer.repository.impl.dart';
import 'package:arosaina/presentation/bloc/connection/connection_bloc.dart';
import 'package:arosaina/presentation/bloc/explorer/explorer_bloc.dart';
import 'package:arosaina/presentation/bloc/transfer/transfer_bloc.dart';
import 'package:arosaina/presentation/services/media_service.dart';
import 'package:get_it/get_it.dart';
import 'package:hive_flutter/hive_flutter.dart';

final sl = GetIt.instance;

Future<void> setupLocator() async {
  // ── Database & Settings ──────────────────────────────────────────────────
  final dbService = DbServiceImpl();
  Hive.registerAdapter(UserModelAdapter());
  Hive.registerAdapter(TransferHistoryModelAdapter());
  await dbService.init();

  sl.registerSingleton<IDbService>(dbService);
  sl.registerLazySingleton<ISettingsService>(() => SettingsServiceImpl(sl()));

  // ── Permissions & Essentials ──────────────────────────────────────────────
  sl.registerLazySingleton<IPermissionHandler>(() => PermissionHandlerImpl());
  sl.registerLazySingleton<MediaService>(() => MediaService(sl()));
  
  sl.registerLazySingleton<ApiClient>(
    () => ApiClient(baseUrl: sl<ISettingsService>().apiBaseUrl),
  );

  sl.registerLazySingleton<ISecureLocalDatasource>(
    () => SecureLocalDatasourceImpl(),
  );

  // ── Network & Datasources ────────────────────────────────────────────────
  sl.registerLazySingleton<IFtpDataSource>(
    () => FtpDataSourceImpl(sl(), sl()),
  );

  // ── Services ─────────────────────────────────────────────────────────────
  sl.registerLazySingleton<IAuthService>(
    () => AuthServiceImpl(sl(), sl(), sl()),
  );
  sl.registerLazySingleton<IHistoryService>(() => HistoryServiceImpl(sl()));

  // ── Repository ───────────────────────────────────────────────────────────
  sl.registerLazySingleton<IServerConnectionRepository>(
    () => ServerConnectionRepositoryImpl(sl()),
  );

  sl.registerLazySingleton<ITransferRepository>(
    () => TransferRepositoryImpl(sl()),
  );

  // ── Blocs ────────────────────────────────────────────────────────────────
  sl.registerFactory(() => ConnectionBloc(sl()));
  sl.registerFactory(() => TransferBloc(sl()));
  sl.registerFactory(() => ExplorerBloc(sl()));
}
