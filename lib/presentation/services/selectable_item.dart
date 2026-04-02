import 'dart:typed_data';

enum SelectableItemType { image, video, audio, file, folder, app }

class SelectableItem {
  final String id;
  final String name;
  final String path;
  final SelectableItemType type;
  final int size;
  final DateTime lastModified;
  final Uint8List? thumbnail;
  final String? mimeType;

  SelectableItem({
    required this.id,
    required this.name,
    required this.path,
    required this.type,
    required this.size,
    required this.lastModified,
    this.thumbnail,
    this.mimeType,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SelectableItem &&
          runtimeType == other.runtimeType &&
          id == other.id;

  @override
  int get hashCode => id.hashCode;
}
