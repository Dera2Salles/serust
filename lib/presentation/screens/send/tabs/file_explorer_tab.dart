import 'dart:io';

import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_history_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/services/media_service.dart';
import 'package:arosaina/presentation/services/selectable_item.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

class FileExplorerTab extends StatefulWidget {
  final Function(SelectableItem) onFileSelected;
  final List<SelectableItem> selectedFiles;

  const FileExplorerTab({
    super.key,
    required this.onFileSelected,
    required this.selectedFiles,
  });

  @override
  State<FileExplorerTab> createState() => _FileExplorerTabState();
}

class _FileExplorerTabState extends State<FileExplorerTab>
    with AutomaticKeepAliveClientMixin {
  String? _currentPath;
  List<SelectableItem> _files = [];
  bool _isLoading = true;
  bool _isReceivedView = false;

  final MediaService _mediaService = sl<MediaService>();
  final IHistoryService _historyService = sl<IHistoryService>();

  @override
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    _initDir();
  }

  Future<void> _initDir() async {
    setState(() {
      _isLoading = true;
    });
    try {
      if (_isReceivedView) {
        await _refreshReceivedFiles();
      } else {
        if (Platform.isAndroid) {
          _currentPath = '/storage/emulated/0';
        } else {
          _currentPath = (await getApplicationDocumentsDirectory()).path;
        }
        await _refreshFiles();
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _refreshReceivedFiles() async {
    setState(() {
      _isLoading = true;
    });
    try {
      final history = _historyService.getHistory();
      final List<SelectableItem> receivedFiles = [];

      for (final record in history) {
        if (!record.isSent &&
            record.status == 'completed' &&
            record.filePath != null) {
          final file = File(record.filePath!);
          if (await file.exists()) {
            receivedFiles.add(
              SelectableItem(
                id: record.id,
                name: record.fileName,
                path: record.filePath!,
                type: SelectableItemType.file,
                size: record.fileSize,
                lastModified: record.timestamp,
              ),
            );
          }
        }
      }
      if (mounted) {
        setState(() {
          _files = receivedFiles;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _refreshFiles() async {
    if (_currentPath == null) return;
    setState(() {
      _isLoading = true;
    });
    try {
      final List<SelectableItem> fetchedItems = await _mediaService.getFiles(
        directoryPath: _currentPath,
      );
      fetchedItems.sort((a, b) {
        if (a.type == SelectableItemType.folder &&
            b.type != SelectableItemType.folder) {
          return -1;
        }
        if (a.type != SelectableItemType.folder &&
            b.type == SelectableItemType.folder) {
          return 1;
        }
        return a.name.toLowerCase().compareTo(b.name.toLowerCase());
      });
      if (mounted) {
        setState(() {
          _files = fetchedItems;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  void _navigateTo(SelectableItem item) {
    if (item.type == SelectableItemType.folder) {
      setState(() {
        _currentPath = item.path;
      });
      _refreshFiles();
    }
  }

  void _navigateBack() {
    if (_currentPath == null) return;
    final parent = Directory(_currentPath!).parent;
    if (parent.path == _currentPath) return;
    setState(() {
      _currentPath = parent.path;
    });
    _refreshFiles();
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);

    if (_isLoading) {
      return Center(
        child: CircularProgressIndicator(color: Theme.of(context).primaryColor),
      );
    }

    return Column(
      children: [
        // View Toggle
        Padding(
          padding: EdgeInsets.symmetric(
            horizontal: context.rs(24),
            vertical: context.rs(8),
          ),
          child: Container(
            padding: EdgeInsets.all(context.rs(4)),
            decoration: BoxDecoration(
              color: AppTheme.neutral100,
              borderRadius: BorderRadius.circular(context.rs(16)),
            ),
            child: Row(
              children: [
                Expanded(
                  child: _ViewToggleButton(
                    label: "Explore",
                    isSelected: !_isReceivedView,
                    onTap: () {
                      if (_isReceivedView) {
                        setState(() => _isReceivedView = false);
                        _initDir();
                      }
                    },
                  ),
                ),
                Expanded(
                  child: _ViewToggleButton(
                    label: "Received",
                    isSelected: _isReceivedView,
                    onTap: () {
                      if (!_isReceivedView) {
                        setState(() => _isReceivedView = true);
                        _initDir();
                      }
                    },
                  ),
                ),
              ],
            ),
          ),
        ),

        // Path Header
        if (!_isReceivedView)
          Padding(
            padding: EdgeInsets.symmetric(
              horizontal: context.rs(24),
              vertical: context.rs(4),
            ),
            child: Row(
              children: [
                GestureDetector(
                  onTap: _navigateBack,
                  child: Container(
                    padding: EdgeInsets.all(context.rs(8)),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      shape: BoxShape.circle,
                      border: Border.all(color: AppTheme.neutral200),
                    ),
                    child: Icon(
                      Icons.arrow_upward_rounded,
                      size: context.rs(16),
                    ),
                  ),
                ),
                SizedBox(width: context.rs(12)),
                Expanded(
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: Text(
                      _currentPath ?? "",
                      style: TextStyle(
                        fontSize: context.rf(12),
                        color: AppTheme.neutral500,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),

        Expanded(
          child: ListView.builder(
            padding: EdgeInsets.all(context.rs(24)),
            itemCount: _files.length,
            itemBuilder: (context, index) {
              final item = _files[index];
              final isSelected =
                  item.type != SelectableItemType.folder &&
                  widget.selectedFiles.any((f) => f.path == item.path);

              return Padding(
                padding: EdgeInsets.only(bottom: context.rs(12)),
                child: _FileExplorerItem(
                  item: item,
                  isSelected: isSelected,
                  onTap: () {
                    if (item.type == SelectableItemType.folder) {
                      _navigateTo(item);
                    } else {
                      widget.onFileSelected(item);
                    }
                  },
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

class _FileExplorerItem extends StatelessWidget {
  final SelectableItem item;
  final bool isSelected;
  final VoidCallback onTap;

  const _FileExplorerItem({
    required this.item,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return SoftCard(
      padding: EdgeInsets.all(context.rs(12)),
      child: InkWell(
        onTap: onTap,
        child: Row(
          children: [
            Container(
              width: context.rs(48),
              height: context.rs(48),
              decoration: BoxDecoration(
                color: _getIconColor().withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(context.rs(12)),
              ),
              child: Icon(_getIconData(), color: _getIconColor(), size: 24),
            ),
            SizedBox(width: context.rs(16)),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    item.name,
                    style: TextStyle(
                      fontSize: context.rf(14),
                      fontWeight: FontWeight.w600,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  if (item.type != SelectableItemType.folder)
                    Text(
                      _formatFileSize(item.size),
                      style: TextStyle(
                        fontSize: context.rf(12),
                        color: AppTheme.neutral500,
                      ),
                    ),
                ],
              ),
            ),
            if (item.type == SelectableItemType.folder)
              Icon(Icons.chevron_right_rounded, color: AppTheme.neutral300)
            else
              Container(
                width: context.rs(24),
                height: context.rs(24),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  border: Border.all(
                    color: isSelected
                        ? Theme.of(context).primaryColor
                        : AppTheme.neutral200,
                    width: 2,
                  ),
                  color: isSelected
                      ? Theme.of(context).primaryColor
                      : Colors.transparent,
                ),
                child: isSelected
                    ? Icon(
                        Icons.check,
                        size: context.rs(14),
                        color: Colors.white,
                      )
                    : null,
              ),
          ],
        ),
      ),
    );
  }

  IconData _getIconData() {
    if (item.type == SelectableItemType.folder) return Icons.folder_rounded;
    final ext = item.name.split('.').last.toLowerCase();
    switch (ext) {
      case 'pdf':
        return Icons.picture_as_pdf_rounded;
      case 'doc':
      case 'docx':
        return Icons.description_rounded;
      case 'mp3':
        return Icons.music_note_rounded;
      case 'mp4':
        return Icons.videocam_rounded;
      case 'jpg':
      case 'png':
        return Icons.image_rounded;
      default:
        return Icons.insert_drive_file_rounded;
    }
  }

  Color _getIconColor() {
    if (item.type == SelectableItemType.folder) return Colors.amber;
    final ext = item.name.split('.').last.toLowerCase();
    switch (ext) {
      case 'pdf':
        return Colors.red;
      case 'mp3':
        return Colors.purple;
      case 'mp4':
        return Colors.blue;
      case 'jpg':
      case 'png':
        return Colors.teal;
      default:
        return AppTheme.neutral500;
    }
  }

  String _formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}

class _ViewToggleButton extends StatelessWidget {
  final String label;
  final bool isSelected;
  final VoidCallback onTap;

  const _ViewToggleButton({
    required this.label,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: EdgeInsets.symmetric(vertical: context.rs(10)),
        decoration: BoxDecoration(
          color: isSelected
              ? Theme.of(context).colorScheme.surface
              : Colors.transparent,
          borderRadius: BorderRadius.circular(context.rs(12)),
          boxShadow: isSelected
              ? [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.05),
                    blurRadius: 4,
                    offset: const Offset(0, 2),
                  ),
                ]
              : null,
        ),
        child: Text(
          label,
          textAlign: TextAlign.center,
          style: TextStyle(
            color: isSelected
                ? Theme.of(context).primaryColor
                : AppTheme.neutral500,
            fontWeight: FontWeight.bold,
            fontSize: context.rf(14),
          ),
        ),
      ),
    );
  }
}
