import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/models/ftp_entry.dart';
import 'package:arosaina/presentation/bloc/explorer/explorer_bloc.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class ExplorerScreen extends StatelessWidget {
  const ExplorerScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => sl<ExplorerBloc>()..add(ExplorerNavigateTo('/')),
      child: const _ExplorerView(),
    );
  }
}

class _ExplorerView extends StatelessWidget {
  const _ExplorerView();

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final bg = isDark ? AppTheme.notionBlack : Colors.white;
    final surface = isDark ? AppTheme.notionSurface : AppTheme.notionLightSurface;
    final textPrimary = isDark ? Colors.white : AppTheme.notionBlack;
    final textSecondary = isDark ? AppTheme.notionTextSecondaryDark : AppTheme.notionTextSecondaryLight;
    final divider = isDark ? AppTheme.notionDividerDark : AppTheme.notionDividerLight;

    return BlocConsumer<ExplorerBloc, ExplorerState>(
      listener: (context, state) {
        final messenger = ScaffoldMessenger.of(context);

        if (state.successMessage != null) {
          messenger.showSnackBar(
            SnackBar(
              backgroundColor: Theme.of(context).brightness == Brightness.dark
                  ? AppTheme.notionSurface
                  : AppTheme.notionBlack,
              behavior: SnackBarBehavior.floating,
              duration: const Duration(seconds: 2),
              content: Text(
                state.successMessage!,
                style: const TextStyle(color: Colors.white),
              ),
            ),
          );
        }

        if (state.errorMessage != null && state.status == ExplorerStatus.error) {
          messenger.showSnackBar(
            SnackBar(
              backgroundColor: Colors.red.shade700,
              behavior: SnackBarBehavior.floating,
              duration: const Duration(seconds: 2),
              content: Text(
                state.errorMessage!,
                style: const TextStyle(color: Colors.white),
              ),
            ),
          );
        }
      },
      builder: (context, state) {
        final bloc = context.read<ExplorerBloc>();

        return Scaffold(
          backgroundColor: bg,
          appBar: AppBar(
            backgroundColor: bg,
            elevation: 0,
            titleSpacing: 0,
            title: _BreadcrumbBar(
              segments: state.pathSegments,
              onTap: (path) => bloc.add(ExplorerNavigateTo(path)),
              textPrimary: textPrimary,
              textSecondary: textSecondary,
            ),
            leading: state.pathSegments.length > 1
                ? IconButton(
                    icon: Icon(Icons.arrow_back_ios_new, size: 18, color: textPrimary),
                    onPressed: () => bloc.add(ExplorerNavigateUp()),
                  )
                : const SizedBox.shrink(),
            bottom: PreferredSize(
              preferredSize: const Size.fromHeight(1),
              child: Container(color: divider, height: 1),
            ),
          ),
          body: Builder(builder: (_) {
            if (state.status == ExplorerStatus.loading) {
              return Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: isDark ? Colors.white : AppTheme.notionBlack,
                  ),
                ),
              );
            }
            if (state.status == ExplorerStatus.error) {
              return Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.error_outline, color: textSecondary, size: 40),
                    const SizedBox(height: 12),
                    Text(
                      state.errorMessage ?? 'Something went wrong',
                      style: TextStyle(color: textSecondary, fontSize: 13),
                    ),
                    const SizedBox(height: 16),
                    GestureDetector(
                      onTap: () => bloc.add(ExplorerNavigateTo(state.currentPath)),
                      child: Text(
                        'Retry',
                        style: TextStyle(
                          color: textPrimary,
                          fontWeight: FontWeight.w700,
                          fontSize: 13,
                          decoration: TextDecoration.underline,
                        ),
                      ),
                    ),
                  ],
                ),
              );
            }
            if (state.entries.isEmpty) {
              return Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.folder_open, color: textSecondary, size: 44),
                    const SizedBox(height: 12),
                    Text(
                      'This folder is empty',
                      style: TextStyle(color: textSecondary, fontSize: 14, fontWeight: FontWeight.w500),
                    ),
                  ],
                ),
              );
            }

            final dirs = state.entries.where((e) => e.isDirectory).toList();
            final files = state.entries.where((e) => !e.isDirectory).toList();
            final sorted = [...dirs, ...files];

            return ListView.builder(
              padding: EdgeInsets.zero,
              itemCount: sorted.length,
              itemBuilder: (context, i) {
                final entry = sorted[i];
                return _EntryRow(
                  entry: entry,
                  surface: surface,
                  textPrimary: textPrimary,
                  textSecondary: textSecondary,
                  divider: divider,
                  onTap: () {
                    if (entry.isDirectory) {
                      final path = state.currentPath == '/'
                          ? '/${entry.name}'
                          : '${state.currentPath}/${entry.name}';
                      bloc.add(ExplorerNavigateTo(path));
                    } else {
                      bloc.add(ExplorerDownloadFile(entry));
                    }
                  },
                  onDelete: () => _confirmDelete(context, bloc, entry, isDark, textPrimary, textSecondary),
                );
              },
            );
          }),
          floatingActionButton: _ExplorerFab(
            isDark: isDark,
            textPrimary: textPrimary,
            onUpload: () => bloc.add(ExplorerUploadFile()),
            onNewFolder: () => _showNewFolderDialog(context, bloc, isDark, textPrimary, textSecondary),
          ),
        );
      },
    );
  }

  void _confirmDelete(
    BuildContext context,
    ExplorerBloc bloc,
    FtpEntry entry,
    bool isDark,
    Color textPrimary,
    Color textSecondary,
  ) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: isDark ? AppTheme.notionSurface : Colors.white,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        title: Text('Delete "${entry.name}"?',
            style: TextStyle(color: textPrimary, fontSize: 16, fontWeight: FontWeight.w700)),
        content: Text(
          entry.isDirectory
              ? 'This folder and all its contents will be deleted.'
              : 'This file will be permanently deleted.',
          style: TextStyle(color: textSecondary, fontSize: 13),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text('Cancel', style: TextStyle(color: textSecondary)),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              bloc.add(ExplorerDeleteEntry(entry));
            },
            child: const Text('Delete', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }

  void _showNewFolderDialog(
    BuildContext context,
    ExplorerBloc bloc,
    bool isDark,
    Color textPrimary,
    Color textSecondary,
  ) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: isDark ? AppTheme.notionSurface : Colors.white,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        title: Text('New Folder', style: TextStyle(color: textPrimary, fontSize: 16, fontWeight: FontWeight.w700)),
        content: TextField(
          controller: controller,
          autofocus: true,
          style: TextStyle(color: textPrimary, fontSize: 14),
          decoration: InputDecoration(
            hintText: 'Folder name',
            hintStyle: TextStyle(color: textSecondary),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide(color: isDark ? AppTheme.notionDividerDark : AppTheme.notionDividerLight),
            ),
            enabledBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide(color: isDark ? AppTheme.notionDividerDark : AppTheme.notionDividerLight),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide(color: textPrimary),
            ),
            fillColor: isDark ? AppTheme.notionBlack : Colors.white,
            filled: true,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text('Cancel', style: TextStyle(color: textSecondary)),
          ),
          TextButton(
            onPressed: () {
              final name = controller.text.trim();
              if (name.isNotEmpty) {
                Navigator.pop(ctx);
                bloc.add(ExplorerCreateFolder(name));
              }
            },
            child: Text('Create', style: TextStyle(color: textPrimary, fontWeight: FontWeight.w700)),
          ),
        ],
      ),
    );
  }
}

// ── Breadcrumb ────────────────────────────────────────────────────────────────

class _BreadcrumbBar extends StatelessWidget {
  final List<String> segments;
  final void Function(String) onTap;
  final Color textPrimary;
  final Color textSecondary;

  const _BreadcrumbBar({
    required this.segments,
    required this.onTap,
    required this.textPrimary,
    required this.textSecondary,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          for (int i = 0; i < segments.length; i++) ...[
            if (i > 0)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4),
                child: Text('/', style: TextStyle(color: textSecondary, fontSize: 13)),
              ),
            GestureDetector(
              onTap: () {
                if (i < segments.length - 1) {
                  final path = i == 0 ? '/' : segments.sublist(0, i + 1).join('/');
                  onTap(path);
                }
              },
              child: Text(
                segments[i],
                style: TextStyle(
                  color: i == segments.length - 1 ? textPrimary : textSecondary,
                  fontWeight: i == segments.length - 1 ? FontWeight.w700 : FontWeight.w400,
                  fontSize: 14,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

// ── Entry Row ─────────────────────────────────────────────────────────────────

class _EntryRow extends StatelessWidget {
  final FtpEntry entry;
  final Color surface;
  final Color textPrimary;
  final Color textSecondary;
  final Color divider;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const _EntryRow({
    required this.entry,
    required this.surface,
    required this.textPrimary,
    required this.textSecondary,
    required this.divider,
    required this.onTap,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      onLongPress: onDelete,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 13),
        decoration: BoxDecoration(
          border: Border(bottom: BorderSide(color: divider, width: 0.5)),
        ),
        child: Row(
          children: [
            Icon(
              entry.isDirectory ? Icons.folder_rounded : _fileIcon(entry.name),
              size: 20,
              color: entry.isDirectory ? textPrimary : textSecondary,
            ),
            const SizedBox(width: 14),
            Expanded(
              child: Text(
                entry.name,
                style: TextStyle(
                  color: textPrimary,
                  fontSize: 14,
                  fontWeight: entry.isDirectory ? FontWeight.w600 : FontWeight.w400,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            if (entry.isDirectory)
              Icon(Icons.chevron_right, size: 18, color: textSecondary),
            if (!entry.isDirectory)
              Icon(Icons.download_rounded, size: 16, color: textSecondary),
          ],
        ),
      ),
    );
  }

  IconData _fileIcon(String name) {
    final ext = name.split('.').last.toLowerCase();
    switch (ext) {
      case 'jpg': case 'jpeg': case 'png': case 'gif': case 'webp':
        return Icons.image_rounded;
      case 'mp4': case 'mov': case 'avi': case 'mkv':
        return Icons.video_file_rounded;
      case 'mp3': case 'wav': case 'ogg': case 'aac':
        return Icons.audio_file_rounded;
      case 'pdf':
        return Icons.picture_as_pdf_rounded;
      case 'txt': case 'md':
        return Icons.article_rounded;
      case 'zip': case 'tar': case 'gz': case 'rar':
        return Icons.folder_zip_rounded;
      default:
        return Icons.insert_drive_file_rounded;
    }
  }
}

// ── FAB ───────────────────────────────────────────────────────────────────────

class _ExplorerFab extends StatefulWidget {
  final bool isDark;
  final Color textPrimary;
  final VoidCallback onUpload;
  final VoidCallback onNewFolder;

  const _ExplorerFab({
    required this.isDark,
    required this.textPrimary,
    required this.onUpload,
    required this.onNewFolder,
  });

  @override
  State<_ExplorerFab> createState() => _ExplorerFabState();
}

class _ExplorerFabState extends State<_ExplorerFab> with SingleTickerProviderStateMixin {
  bool _expanded = false;
  late final AnimationController _ctrl;
  late final Animation<double> _fade;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(vsync: this, duration: const Duration(milliseconds: 200));
    _fade = CurvedAnimation(parent: _ctrl, curve: Curves.easeOut);
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  void _toggle() {
    setState(() => _expanded = !_expanded);
    if (_expanded) { _ctrl.forward(); } else { _ctrl.reverse(); }
  }

  @override
  Widget build(BuildContext context) {
    final bg = widget.isDark ? Colors.white : AppTheme.notionBlack;
    final fg = widget.isDark ? AppTheme.notionBlack : Colors.white;

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        FadeTransition(
          opacity: _fade,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              _MiniAction(bg: bg, fg: fg, icon: Icons.upload_rounded, label: 'Upload File',
                onTap: () { _toggle(); widget.onUpload(); }),
              const SizedBox(height: 10),
              _MiniAction(bg: bg, fg: fg, icon: Icons.create_new_folder_rounded, label: 'New Folder',
                onTap: () { _toggle(); widget.onNewFolder(); }),
              const SizedBox(height: 14),
            ],
          ),
        ),
        FloatingActionButton(
          backgroundColor: bg,
          foregroundColor: fg,
          elevation: 2,
          onPressed: _toggle,
          child: AnimatedRotation(
            turns: _expanded ? 0.125 : 0,
            duration: const Duration(milliseconds: 200),
            child: const Icon(Icons.add, size: 26),
          ),
        ),
      ],
    );
  }
}

class _MiniAction extends StatelessWidget {
  final Color bg, fg;
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  const _MiniAction({
    required this.bg, required this.fg,
    required this.icon, required this.label, required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: bg.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Text(label,
              style: TextStyle(color: bg, fontSize: 12, fontWeight: FontWeight.w600)),
          ),
          const SizedBox(width: 10),
          Material(
            color: bg,
            borderRadius: BorderRadius.circular(28),
            child: SizedBox(
              width: 40, height: 40,
              child: Icon(icon, color: fg, size: 18),
            ),
          ),
        ],
      ),
    );
  }
}
