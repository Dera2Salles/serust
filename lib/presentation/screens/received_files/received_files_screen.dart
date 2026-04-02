import 'dart:io';

import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';
import 'package:open_filex/open_filex.dart';
import 'package:path_provider/path_provider.dart';
import 'package:intl/intl.dart';

/// ── Received Files Screen — File Explorer ─────────────────────────────────
/// Affiche tous les fichiers reçus dans le dossier Arosaina
class ReceivedFilesScreen extends StatefulWidget {
  const ReceivedFilesScreen({super.key});

  @override
  State<ReceivedFilesScreen> createState() => _ReceivedFilesScreenState();
}

class _ReceivedFilesScreenState extends State<ReceivedFilesScreen> {
  List<_FileItem> _files = [];
  bool _loading = true;
  String _search = '';
  _SortMode _sortMode = _SortMode.dateDesc;
  _FilterMode _filterMode = _FilterMode.all;

  @override
  void initState() {
    super.initState();
    _loadFiles();
  }

  Future<void> _loadFiles() async {
    setState(() => _loading = true);
    try {
      final dir = await _getReceiveDirectory();
      if (!await dir.exists()) {
        setState(() {
          _files = [];
          _loading = false;
        });
        return;
      }
      final entities = dir.listSync(recursive: false);
      final files = entities
          .whereType<File>()
          .map((f) => _FileItem.fromFile(f))
          .toList();
      setState(() {
        _files = files;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _files = [];
        _loading = false;
      });
    }
  }

  Future<Directory> _getReceiveDirectory() async {
    if (Platform.isAndroid) {
      final ext = await getExternalStorageDirectory();
      return Directory('${ext?.path ?? ''}/Arosaina');
    } else if (Platform.isIOS) {
      final docs = await getApplicationDocumentsDirectory();
      return Directory('${docs.path}/Arosaina');
    } else {
      final dl = await getDownloadsDirectory();
      return Directory('${dl?.path ?? ''}/Arosaina');
    }
  }

  List<_FileItem> get _filtered {
    var list = _files.where((f) {
      final matchSearch = f.name.toLowerCase().contains(_search.toLowerCase());
      final matchFilter =
          _filterMode == _FilterMode.all || f.type == _filterMode;
      return matchSearch && matchFilter;
    }).toList();

    switch (_sortMode) {
      case _SortMode.nameAsc:
        list.sort((a, b) => a.name.compareTo(b.name));
        break;
      case _SortMode.nameDesc:
        list.sort((a, b) => b.name.compareTo(a.name));
        break;
      case _SortMode.sizeDesc:
        list.sort((a, b) => b.size.compareTo(a.size));
        break;
      case _SortMode.dateDesc:
        list.sort((a, b) => b.modified.compareTo(a.modified));
        break;
      case _SortMode.dateAsc:
        list.sort((a, b) => a.modified.compareTo(b.modified));
        break;
    }
    return list;
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final primary = Theme.of(context).primaryColor;
    final filtered = _filtered;

    return Scaffold(
      backgroundColor: isDark
          ? AppTheme.backgroundDark
          : AppTheme.backgroundLight,
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // ── Header ─────────────────────────────────────────────────────
            Padding(
              padding: EdgeInsets.fromLTRB(
                context.rs(20),
                context.rs(20),
                context.rs(20),
                0,
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      'Received Files',
                      style: TextStyle(
                        fontSize: context.rf(22),
                        fontWeight: FontWeight.w800,
                        letterSpacing: -0.5,
                        color: isDark ? Colors.white : AppTheme.neutral900,
                      ),
                    ),
                  ),
                  AroIconButton(
                    icon: Icons.refresh_rounded,
                    onTap: _loadFiles,
                    bgColor: primary.withValues(alpha: 0.1),
                    iconColor: primary,
                  ),
                  SizedBox(width: context.rs(8)),
                  AroIconButton(
                    icon: Icons.sort_rounded,
                    onTap: () => _showSortSheet(context),
                    bgColor: isDark ? AppTheme.neutral800 : AppTheme.neutral100,
                    iconColor: isDark ? Colors.white : AppTheme.neutral700,
                  ),
                ],
              ),
            ),

            SizedBox(height: context.rs(16)),

            // ── Search ─────────────────────────────────────────────────────
            Padding(
              padding: EdgeInsets.symmetric(horizontal: context.rs(20)),
              child: Container(
                decoration: BoxDecoration(
                  color: isDark ? AppTheme.neutral800 : Colors.white,
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(
                    color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
                  ),
                ),
                child: TextField(
                  onChanged: (v) => setState(() => _search = v),
                  decoration: InputDecoration(
                    hintText: 'Search files...',
                    hintStyle: TextStyle(
                      color: AppTheme.neutral400,
                      fontSize: 14,
                    ),
                    prefixIcon: Icon(
                      Icons.search_rounded,
                      color: AppTheme.neutral400,
                      size: 20,
                    ),
                    border: InputBorder.none,
                    contentPadding: EdgeInsets.symmetric(
                      horizontal: context.rs(16),
                      vertical: context.rs(12),
                    ),
                  ),
                  style: TextStyle(
                    fontSize: context.rf(14),
                    color: isDark ? Colors.white : AppTheme.neutral900,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
            ),

            SizedBox(height: context.rs(14)),

            // ── Filter chips ───────────────────────────────────────────────
            SizedBox(
              height: 36,
              child: ListView(
                scrollDirection: Axis.horizontal,
                padding: EdgeInsets.symmetric(horizontal: context.rs(20)),
                children: _FilterMode.values.map((f) {
                  final active = _filterMode == f;
                  return Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: GestureDetector(
                      onTap: () => setState(() => _filterMode = f),
                      child: AnimatedContainer(
                        duration: const Duration(milliseconds: 180),
                        padding: const EdgeInsets.symmetric(
                          horizontal: 14,
                          vertical: 6,
                        ),
                        decoration: BoxDecoration(
                          color: active
                              ? primary
                              : (isDark ? AppTheme.neutral800 : Colors.white),
                          borderRadius: BorderRadius.circular(20),
                          border: Border.all(
                            color: active
                                ? primary
                                : (isDark
                                      ? AppTheme.neutral700
                                      : AppTheme.neutral200),
                          ),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              _filterIcon(f),
                              size: 14,
                              color: active
                                  ? Colors.white
                                  : AppTheme.neutral500,
                            ),
                            const SizedBox(width: 6),
                            Text(
                              _filterLabel(f),
                              style: TextStyle(
                                fontSize: 12,
                                fontWeight: FontWeight.w700,
                                color: active
                                    ? Colors.white
                                    : AppTheme.neutral500,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),

            SizedBox(height: context.rs(14)),

            // ── Count badge ────────────────────────────────────────────────
            Padding(
              padding: EdgeInsets.symmetric(horizontal: context.rs(20)),
              child: Text(
                '${filtered.length} file${filtered.length != 1 ? 's' : ''}',
                style: TextStyle(
                  fontSize: context.rf(12),
                  fontWeight: FontWeight.w600,
                  color: AppTheme.neutral400,
                  letterSpacing: 0.3,
                ),
              ),
            ),

            SizedBox(height: context.rs(10)),

            // ── File List ──────────────────────────────────────────────────
            Expanded(
              child: _loading
                  ? Center(
                      child: CircularProgressIndicator(
                        color: primary,
                        strokeWidth: 2,
                      ),
                    )
                  : filtered.isEmpty
                  ? _EmptyState(search: _search)
                  : RefreshIndicator(
                      onRefresh: _loadFiles,
                      color: primary,
                      child: ListView.separated(
                        padding: EdgeInsets.symmetric(
                          horizontal: context.rs(20),
                          vertical: context.rs(4),
                        ),
                        itemCount: filtered.length,
                        separatorBuilder: (context, index) =>
                            SizedBox(height: context.rs(8)),
                        itemBuilder: (ctx, i) => _FileCard(
                          item: filtered[i],
                          onDelete: () async {
                            await _confirmDelete(context, filtered[i]);
                          },
                        ),
                      ),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _confirmDelete(BuildContext context, _FileItem item) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (_) => AlertDialog(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
        title: const Text(
          'Delete File',
          style: TextStyle(fontWeight: FontWeight.w800),
        ),
        content: Text('Delete "${item.name}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete', style: TextStyle(color: AppTheme.rose)),
          ),
        ],
      ),
    );
    if (confirmed == true) {
      try {
        await File(item.path).delete();
        _loadFiles();
      } catch (_) {}
    }
  }

  void _showSortSheet(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    showModalBottomSheet(
      context: context,
      backgroundColor: isDark ? AppTheme.surfaceDark : Colors.white,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (_) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 36,
            height: 4,
            margin: const EdgeInsets.only(top: 12, bottom: 16),
            decoration: BoxDecoration(
              color: AppTheme.neutral300,
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20),
            child: Text(
              'Sort by',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w800,
                color: isDark ? Colors.white : AppTheme.neutral900,
              ),
            ),
          ),
          const SizedBox(height: 12),
          ..._SortMode.values.map(
            (s) => ListTile(
              leading: Icon(
                _sortIcon(s),
                color: _sortMode == s
                    ? Theme.of(context).primaryColor
                    : AppTheme.neutral500,
              ),
              title: Text(
                _sortLabel(s),
                style: TextStyle(
                  fontWeight: FontWeight.w600,
                  color: _sortMode == s ? Theme.of(context).primaryColor : null,
                ),
              ),
              trailing: _sortMode == s
                  ? Icon(
                      Icons.check_rounded,
                      color: Theme.of(context).primaryColor,
                    )
                  : null,
              onTap: () {
                setState(() => _sortMode = s);
                Navigator.pop(context);
              },
            ),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  IconData _filterIcon(_FilterMode f) {
    switch (f) {
      case _FilterMode.all:
        return Icons.all_inclusive_rounded;
      case _FilterMode.image:
        return Icons.image_rounded;
      case _FilterMode.video:
        return Icons.videocam_rounded;
      case _FilterMode.audio:
        return Icons.music_note_rounded;
      case _FilterMode.doc:
        return Icons.description_rounded;
      case _FilterMode.other:
        return Icons.folder_rounded;
    }
  }

  String _filterLabel(_FilterMode f) {
    switch (f) {
      case _FilterMode.all:
        return 'All';
      case _FilterMode.image:
        return 'Images';
      case _FilterMode.video:
        return 'Videos';
      case _FilterMode.audio:
        return 'Audio';
      case _FilterMode.doc:
        return 'Docs';
      case _FilterMode.other:
        return 'Other';
    }
  }

  IconData _sortIcon(_SortMode s) {
    switch (s) {
      case _SortMode.nameAsc:
        return Icons.sort_by_alpha_rounded;
      case _SortMode.nameDesc:
        return Icons.sort_by_alpha_rounded;
      case _SortMode.sizeDesc:
        return Icons.data_usage_rounded;
      case _SortMode.dateDesc:
        return Icons.access_time_rounded;
      case _SortMode.dateAsc:
        return Icons.access_time_rounded;
    }
  }

  String _sortLabel(_SortMode s) {
    switch (s) {
      case _SortMode.nameAsc:
        return 'Name (A → Z)';
      case _SortMode.nameDesc:
        return 'Name (Z → A)';
      case _SortMode.sizeDesc:
        return 'Size (largest)';
      case _SortMode.dateDesc:
        return 'Date (newest)';
      case _SortMode.dateAsc:
        return 'Date (oldest)';
    }
  }
}

// ─── File Card ────────────────────────────────────────────────────────────────
class _FileCard extends StatelessWidget {
  final _FileItem item;
  final VoidCallback onDelete;

  const _FileCard({required this.item, required this.onDelete});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final color = _typeColor(item.type);

    return GestureDetector(
      onTap: () => OpenFilex.open(item.path),
      onLongPress: onDelete,
      child: Container(
        padding: EdgeInsets.all(context.rs(14)),
        decoration: BoxDecoration(
          color: isDark ? AppTheme.neutral800 : Colors.white,
          borderRadius: BorderRadius.circular(14),
          border: Border.all(
            color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
          ),
        ),
        child: Row(
          children: [
            // File type icon block
            Container(
              width: context.rs(48),
              height: context.rs(48),
              decoration: BoxDecoration(
                color: color.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Icon(
                _typeIcon(item.type),
                color: color,
                size: context.rs(24),
              ),
            ),

            SizedBox(width: context.rs(12)),

            // File info
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    item.name,
                    style: TextStyle(
                      fontSize: context.rf(13),
                      fontWeight: FontWeight.w700,
                      color: isDark ? Colors.white : AppTheme.neutral900,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  SizedBox(height: context.rs(3)),
                  Row(
                    children: [
                      Text(
                        _formatSize(item.size),
                        style: TextStyle(
                          fontSize: context.rf(11),
                          color: AppTheme.neutral500,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                      Text(
                        '  ·  ${_formatDate(item.modified)}',
                        style: TextStyle(
                          fontSize: context.rf(11),
                          color: AppTheme.neutral400,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),

            // Extension tag
            AroChip(label: item.ext.toUpperCase(), color: color, small: true),

            SizedBox(width: context.rs(8)),

            // Open icon
            Icon(
              Icons.open_in_new_rounded,
              size: context.rs(16),
              color: AppTheme.neutral400,
            ),
          ],
        ),
      ),
    );
  }

  Color _typeColor(_FilterMode t) {
    switch (t) {
      case _FilterMode.image:
        return AppTheme.violet;
      case _FilterMode.video:
        return AppTheme.rose;
      case _FilterMode.audio:
        return AppTheme.amber;
      case _FilterMode.doc:
        return AppTheme.indigo;
      default:
        return AppTheme.neutral500;
    }
  }

  IconData _typeIcon(_FilterMode t) {
    switch (t) {
      case _FilterMode.image:
        return Icons.image_rounded;
      case _FilterMode.video:
        return Icons.videocam_rounded;
      case _FilterMode.audio:
        return Icons.music_note_rounded;
      case _FilterMode.doc:
        return Icons.description_rounded;
      default:
        return Icons.insert_drive_file_rounded;
    }
  }

  String _formatSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }

  String _formatDate(DateTime d) {
    final now = DateTime.now();
    if (now.difference(d).inDays == 0) return 'Today';
    if (now.difference(d).inDays == 1) return 'Yesterday';
    return DateFormat('MMM d').format(d);
  }
}

// ─── Empty State ──────────────────────────────────────────────────────────────
class _EmptyState extends StatelessWidget {
  final String search;
  const _EmptyState({required this.search});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            width: 80,
            height: 80,
            decoration: BoxDecoration(
              color: AppTheme.neutral100,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Icon(
              search.isEmpty
                  ? Icons.folder_open_rounded
                  : Icons.search_off_rounded,
              size: 40,
              color: AppTheme.neutral300,
            ),
          ),
          SizedBox(height: context.rs(16)),
          Text(
            search.isEmpty
                ? 'No files received yet'
                : 'No results for "$search"',
            style: TextStyle(
              fontSize: context.rf(15),
              fontWeight: FontWeight.w700,
              color: AppTheme.neutral500,
            ),
          ),
          SizedBox(height: context.rs(8)),
          Text(
            search.isEmpty
                ? 'Files you receive will appear here'
                : 'Try a different search term',
            style: TextStyle(
              fontSize: context.rf(13),
              color: AppTheme.neutral400,
            ),
          ),
        ],
      ),
    );
  }
}

// ─── Data Models ──────────────────────────────────────────────────────────────
enum _SortMode { nameAsc, nameDesc, sizeDesc, dateDesc, dateAsc }

enum _FilterMode { all, image, video, audio, doc, other }

class _FileItem {
  final String name;
  final String path;
  final int size;
  final DateTime modified;
  final String ext;
  final _FilterMode type;

  _FileItem({
    required this.name,
    required this.path,
    required this.size,
    required this.modified,
    required this.ext,
    required this.type,
  });

  factory _FileItem.fromFile(File f) {
    final stat = f.statSync();
    final name = f.path.split('/').last;
    final ext = name.contains('.') ? name.split('.').last.toLowerCase() : '';
    return _FileItem(
      name: name,
      path: f.path,
      size: stat.size,
      modified: stat.modified,
      ext: ext,
      type: _detectType(ext),
    );
  }

  static _FilterMode _detectType(String ext) {
    const images = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'heic', 'svg'];
    const videos = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm'];
    const audios = ['mp3', 'wav', 'aac', 'ogg', 'flac', 'm4a', 'opus'];
    const docs = [
      'pdf',
      'doc',
      'docx',
      'xls',
      'xlsx',
      'ppt',
      'pptx',
      'txt',
      'csv',
    ];
    if (images.contains(ext)) return _FilterMode.image;
    if (videos.contains(ext)) return _FilterMode.video;
    if (audios.contains(ext)) return _FilterMode.audio;
    if (docs.contains(ext)) return _FilterMode.doc;
    return _FilterMode.other;
  }
}
