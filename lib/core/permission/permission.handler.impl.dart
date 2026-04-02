import 'package:arosaina/core/permission/i.permission.handler.dart';
import 'package:arosaina/core/permission/location.permission.dart';
import 'package:arosaina/core/permission/nearby.permission.dart';
import 'package:permission_handler/permission_handler.dart' as ph;

class PermissionHandlerImpl implements IPermissionHandler {
  @override
  Future<PermissionStatus> requestPermission(
    PermissionStrategy strategy,
  ) async {
    ph.PermissionStatus status;
    if (strategy is LocationPermissionStrategy) {
      status = await ph.Permission.location.request();
    } else if (strategy is MediaPermissionStrategy) {
      // Request Manage External Storage for Android 11+
      if (await ph.Permission.manageExternalStorage.status.isDenied) {
        await ph.Permission.manageExternalStorage.request();
      }
      status = await ph.Permission.storage.request();
    } else if (strategy is NearbyDevicesPermissionStrategy) {
      // Local Network permissions
      // On Android, typical network permissions are manifest-only, but Location is needed for Wi-Fi info (SSID) in some cases.
      // nsd plugin might need it.
      // On iOS 14+, user must grant "Local Network" permission. This is triggered by usage, but we can't force request easily via permission_handler yet?
      // permission_handler has 'nearbyWifiDevices' but that's for P2P.

      // For a simple HTTP server on local network:
      // Android: Internet (manifest). Location (optional for some checks).
      // iOS: Local Network usage triggers dialog.

      // Let's keep Location as a fallback if we want to confirm Wi-Fi connection.

      final result = await <ph.Permission>[
        ph.Permission.location,
        ph.Permission.nearbyWifiDevices,
      ].request();

      final locationGranted =
          (result[ph.Permission.location] ?? ph.PermissionStatus.denied)
              .isGranted;
      final nearbyGranted =
          (result[ph.Permission.nearbyWifiDevices] ??
                  ph.PermissionStatus.denied)
              .isGranted;

      status = (locationGranted || nearbyGranted)
          ? ph.PermissionStatus.granted
          : (result[ph.Permission.location] ?? ph.PermissionStatus.denied);
      // On Android 13+, NEARBY_WIFI_DEVICES is technially for Wi-Fi Aware/Direct.
      // We are using standard Local Network (Hotspot/Wi-Fi).
      // So mainly Location.
    } else {
      throw UnsupportedError("Unknown permission strategy");
    }
    return _toPermissionStatus(status);
  }

  @override
  Future<bool> checkPermission(PermissionStrategy strategy) async {
    ph.PermissionStatus status;
    if (strategy is LocationPermissionStrategy) {
      status = await ph.Permission.location.status;
    } else if (strategy is MediaPermissionStrategy) {
      status = await ph.Permission.storage.status;
    } else if (strategy is NearbyDevicesPermissionStrategy) {
      final locationGranted = (await ph.Permission.location.status).isGranted;
      final nearbyGranted =
          (await ph.Permission.nearbyWifiDevices.status).isGranted;
      return locationGranted || nearbyGranted;
    } else {
      throw UnsupportedError("Unknown permission strategy");
    }
    return status == ph.PermissionStatus.granted;
  }

  PermissionStatus _toPermissionStatus(ph.PermissionStatus status) {
    switch (status) {
      case ph.PermissionStatus.granted:
        return PermissionStatus.granted;
      case ph.PermissionStatus.denied:
        return PermissionStatus.denied;
      case ph.PermissionStatus.restricted:
        return PermissionStatus.restricted;
      case ph.PermissionStatus.permanentlyDenied:
        return PermissionStatus.permanentlyDenied;
      case ph.PermissionStatus.provisional:
        return PermissionStatus.granted; // Treat provisional as granted for now
      case ph.PermissionStatus.limited:
        return PermissionStatus.granted; // Treat limited as granted for now
    }
  }
}

class MediaPermissionStrategy implements PermissionStrategy {}
