import 'dart:io';

import 'package:arosaina/core/permission/i.permission.handler.dart';
import 'package:arosaina/core/permission/permission.handler.impl.dart'; // Import MediaPermissionStrategy
import 'package:arosaina/presentation/services/selectable_item.dart';
import 'package:mime/mime.dart';
import 'package:path_provider/path_provider.dart';
import 'package:photo_manager/photo_manager.dart';

enum MediaType { image, video, audio, file, app }

class MediaService {
  final IPermissionHandler _permissionHandler;

  MediaService(this._permissionHandler);

  Future<List<SelectableItem>> getMedia(MediaType type) async {
    final PermissionStatus status = await _permissionHandler.requestPermission(
      MediaPermissionStrategy(), // Use MediaPermissionStrategy
    );

    if (status != PermissionStatus.granted) {
      // Handle permission denied
      return [];
    }

    switch (type) {
      case MediaType.image:
        return _getAssets(RequestType.image);
      case MediaType.video:
        return _getAssets(RequestType.video);
      case MediaType.audio:
        return _getAssets(RequestType.audio);
      case MediaType.file:
        return getFiles(); // Call public getFiles
      case MediaType.app:
        return _getApps();
    }
  }

  Future<List<SelectableItem>> _getAssets(RequestType type) async {
    final List<AssetPathEntity> albums = await PhotoManager.getAssetPathList(
      type: type,
      filterOption: FilterOptionGroup(
        imageOption: const FilterOption(
          needTitle: true,
          sizeConstraint: SizeConstraint(minHeight: 100, minWidth: 100),
        ),
        videoOption: const FilterOption(
          needTitle: true,
          sizeConstraint: SizeConstraint(minHeight: 100, minWidth: 100),
        ),
        audioOption: const FilterOption(needTitle: true),
      ),
    );

    final List<SelectableItem> mediaList = [];
    for (final AssetPathEntity album in albums) {
      final List<AssetEntity> assets = await album.getAssetListRange(
        start: 0,
        end: await album.assetCountAsync,
      );
      for (final AssetEntity asset in assets) {
        final File? file = await asset.file;
        if (file != null) {
          mediaList.add(
            SelectableItem(
              id: asset.id,
              name: asset.title ?? file.path.split('/').last,
              path: file.path,
              type: _getSelectableItemType(asset.type),
              size: file.lengthSync(),
              lastModified: asset.modifiedDateTime,
              thumbnail: await asset.thumbnailDataWithSize(
                const ThumbnailSize(200, 200),
              ),
            ),
          );
        }
      }
    }
    return mediaList;
  }

  SelectableItemType _getSelectableItemType(AssetType assetType) {
    switch (assetType) {
      case AssetType.image:
        return SelectableItemType.image;
      case AssetType.video:
        return SelectableItemType.video;
      case AssetType.audio:
        return SelectableItemType.audio;
      case AssetType.other:
        return SelectableItemType.file;
    }
  }

  Future<List<SelectableItem>> getFiles({String? directoryPath}) async {
    await _permissionHandler.requestPermission(MediaPermissionStrategy());
    final List<SelectableItem> files = [];
    final Directory directory = Directory(
      directoryPath ?? (await getExternalStorageDirectory())!.path,
    );

    if (!await directory.exists()) {
      return [];
    }

    final List<FileSystemEntity> entities = directory.listSync(
      recursive: false,
    );

    for (final FileSystemEntity entity in entities) {
      if (entity is File) {
        final FileStat stat = entity.statSync();
        files.add(
          SelectableItem(
            id: entity.path,
            name: entity.path.split('/').last,
            path: entity.path,
            type: _getFileType(entity.path),
            size: stat.size,
            lastModified: stat.modified,
          ),
        );
      } else if (entity is Directory) {
        final FileStat stat = entity.statSync();
        files.add(
          SelectableItem(
            id: entity.path,
            name: entity.path.split('/').last,
            path: entity.path,
            type: SelectableItemType.folder,
            size: stat.size,
            lastModified: stat.modified,
          ),
        );
      }
    }
    return files;
  }

  SelectableItemType _getFileType(String path) {
    final String? mimeType = lookupMimeType(path);
    if (mimeType != null) {
      if (mimeType.startsWith('image')) {
        return SelectableItemType.image;
      } else if (mimeType.startsWith('video')) {
        return SelectableItemType.video;
      } else if (mimeType.startsWith('audio')) {
        return SelectableItemType.audio;
      }
    }
    return SelectableItemType.file;
  }

  Future<List<SelectableItem>> _getApps() async {
    // This is a placeholder. Getting a list of installed apps is platform-specific
    // and requires additional packages or native code.
    return [];
  }
}
