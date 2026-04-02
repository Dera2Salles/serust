import 'dart:io';
import 'package:arosaina/engine/models/transfer_update.dart';
import 'package:arosaina/engine/connection/connection_lifecycle_event.dart';

abstract class IServerConnectionRepository {
  Stream<TransferUpdate> get transferUpdates;
  Stream<ConnectionLifecycleEvent> get connectionLifecycle;

  Future<void> connect();
  Future<void> disconnect();

  Future<void> sendFile(
    String endpointId,
    File file, {
    String? originalFileName,
    int? maxShareCount,
  });

  Future<String?> getLocalIp();

  /// Port actif du serveur.
  int? get activeServerPort;
}
