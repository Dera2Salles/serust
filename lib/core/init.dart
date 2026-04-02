import 'package:arosaina/core/injection.dart';
import 'package:arosaina/core/permission/i.permission.handler.dart';
import 'package:arosaina/core/permission/location.permission.dart';
import 'package:arosaina/core/permission/nearby.permission.dart';
import 'package:arosaina/core/permission/permission.handler.impl.dart';

class Init {
  static Future<void> execute() async {
    final permissionHandler = sl<IPermissionHandler>();

    // Request location permission
    if (!await permissionHandler.checkPermission(
      LocationPermissionStrategy(),
    )) {
      await permissionHandler.requestPermission(LocationPermissionStrategy());
    }

    // Request media permission
    if (!await permissionHandler.checkPermission(MediaPermissionStrategy())) {
      await permissionHandler.requestPermission(MediaPermissionStrategy());
    }

    // Request nearby devices (Bluetooth) permissions for P2P connection
    if (!await permissionHandler.checkPermission(
      NearbyDevicesPermissionStrategy(),
    )) {
      await permissionHandler.requestPermission(
        NearbyDevicesPermissionStrategy(),
      );
    }
  }
}
