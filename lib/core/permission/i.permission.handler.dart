enum PermissionStatus { granted, denied, restricted, permanentlyDenied }

abstract class PermissionStrategy {}

abstract class IPermissionHandler {
  Future<PermissionStatus> requestPermission(PermissionStrategy strategy);
  Future<bool> checkPermission(PermissionStrategy strategy);
}
