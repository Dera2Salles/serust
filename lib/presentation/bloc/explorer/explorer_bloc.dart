import 'dart:async';
import 'dart:io';
import 'package:arosaina/engine/connection/i_ftp_datasource.dart';
import 'package:arosaina/engine/models/ftp_entry.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:path_provider/path_provider.dart';

// ── Events ────────────────────────────────────────────────────────────────────

abstract class ExplorerEvent {}

class ExplorerLoadDir extends ExplorerEvent {
  final String path;
  ExplorerLoadDir(this.path);
}

class ExplorerNavigateTo extends ExplorerEvent {
  final String path;
  ExplorerNavigateTo(this.path);
}

class ExplorerNavigateUp extends ExplorerEvent {}

class ExplorerRefresh extends ExplorerEvent {}

class ExplorerCreateFolder extends ExplorerEvent {
  final String name;
  ExplorerCreateFolder(this.name);
}

class ExplorerDeleteEntry extends ExplorerEvent {
  final FtpEntry entry;
  ExplorerDeleteEntry(this.entry);
}

class ExplorerRenameEntry extends ExplorerEvent {
  final FtpEntry entry;
  final String newName;
  ExplorerRenameEntry(this.entry, this.newName);
}

class ExplorerUploadFile extends ExplorerEvent {}

class ExplorerDownloadFile extends ExplorerEvent {
  final FtpEntry entry;
  ExplorerDownloadFile(this.entry);
}

// ── State ─────────────────────────────────────────────────────────────────────

enum ExplorerStatus { idle, loading, loaded, error }

class ExplorerState {
  final ExplorerStatus status;
  final List<FtpEntry> entries;
  final List<String> pathSegments; // breadcrumb: ['/', 'photos', 'summer']
  final String? errorMessage;
  final String? successMessage;

  const ExplorerState({
    this.status = ExplorerStatus.idle,
    this.entries = const [],
    this.pathSegments = const ['/'],
    this.errorMessage,
    this.successMessage,
  });

  String get currentPath {
    if (pathSegments.length == 1) return '/';
    return '/${pathSegments.sublist(1).join('/')}';
  }

  ExplorerState copyWith({
    ExplorerStatus? status,
    List<FtpEntry>? entries,
    List<String>? pathSegments,
    String? errorMessage,
    String? successMessage,
  }) =>
      ExplorerState(
        status: status ?? this.status,
        entries: entries ?? this.entries,
        pathSegments: pathSegments ?? this.pathSegments,
        errorMessage: errorMessage,
        successMessage: successMessage,
      );
}

// ── Bloc ──────────────────────────────────────────────────────────────────────

class ExplorerBloc extends Bloc<ExplorerEvent, ExplorerState> {
  final IFtpDataSource _datasource;

  ExplorerBloc(this._datasource) : super(const ExplorerState()) {
    on<ExplorerLoadDir>(_onLoadDir);
    on<ExplorerNavigateTo>(_onNavigateTo);
    on<ExplorerNavigateUp>(_onNavigateUp);
    on<ExplorerRefresh>(_onRefresh);
    on<ExplorerCreateFolder>(_onCreateFolder);
    on<ExplorerDeleteEntry>(_onDeleteEntry);
    on<ExplorerRenameEntry>(_onRenameEntry);
    on<ExplorerUploadFile>(_onUploadFile);
    on<ExplorerDownloadFile>(_onDownloadFile);
  }

  Future<void> _ensureConnected() => _datasource.connect();

  // ── Load / navigate ────────────────────────────────────────────────────────

  Future<void> _onLoadDir(ExplorerLoadDir event, Emitter<ExplorerState> emit) async {
    emit(state.copyWith(status: ExplorerStatus.loading));
    try {
      await _ensureConnected();
      await _datasource.changeDir(event.path);
      final entries = await _datasource.listDirectory();
      final currentPath = await _datasource.currentDir();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        pathSegments: _pathToSegments(currentPath),
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Cannot load directory: $e',
      ));
    }
  }

  Future<void> _onNavigateTo(ExplorerNavigateTo event, Emitter<ExplorerState> emit) async {
    emit(state.copyWith(status: ExplorerStatus.loading));
    try {
      await _ensureConnected();
      await _datasource.changeDir(event.path);
      final entries = await _datasource.listDirectory();
      final currentPath = await _datasource.currentDir();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        pathSegments: _pathToSegments(currentPath),
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Navigation failed: $e',
      ));
    }
  }

  Future<void> _onNavigateUp(ExplorerNavigateUp event, Emitter<ExplorerState> emit) async {
    if (state.pathSegments.length <= 1) return; // already at root
    emit(state.copyWith(status: ExplorerStatus.loading));
    try {
      await _ensureConnected();
      await _datasource.navigateUp();
      final entries = await _datasource.listDirectory();
      final currentPath = await _datasource.currentDir();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        pathSegments: _pathToSegments(currentPath),
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Navigate up failed: $e',
      ));
    }
  }

  Future<void> _onRefresh(ExplorerRefresh event, Emitter<ExplorerState> emit) async {
    add(ExplorerNavigateTo(state.currentPath));
  }

  // ── Folder creation ────────────────────────────────────────────────────────

  Future<void> _onCreateFolder(
      ExplorerCreateFolder event, Emitter<ExplorerState> emit) async {
    // Validate name
    if (event.name.trim().isEmpty) {
      emit(state.copyWith(errorMessage: 'Folder name cannot be empty.'));
      return;
    }
    if (event.name.contains('/') || event.name.contains('\\')) {
      emit(state.copyWith(errorMessage: 'Folder name cannot contain slashes.'));
      return;
    }

    try {
      await _ensureConnected();
      await _datasource.mkdir(event.name.trim());
      // Reload current directory
      final entries = await _datasource.listDirectory();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        successMessage: 'Folder "${event.name}" created.',
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Failed to create folder: $e',
      ));
    }
  }

  // ── Delete ─────────────────────────────────────────────────────────────────

  Future<void> _onDeleteEntry(
      ExplorerDeleteEntry event, Emitter<ExplorerState> emit) async {
    try {
      await _ensureConnected();
      if (event.entry.isDirectory) {
        await _datasource.rmdir(event.entry.name);
      } else {
        await _datasource.deleteFile(event.entry.name);
      }
      final entries = await _datasource.listDirectory();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        successMessage: '"${event.entry.name}" deleted.',
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Failed to delete: $e',
      ));
    }
  }

  // ── Rename ─────────────────────────────────────────────────────────────────

  Future<void> _onRenameEntry(
      ExplorerRenameEntry event, Emitter<ExplorerState> emit) async {
    if (event.newName.trim().isEmpty) {
      emit(state.copyWith(errorMessage: 'New name cannot be empty.'));
      return;
    }
    try {
      await _ensureConnected();
      await _datasource.rename(event.entry.name, event.newName.trim());
      final entries = await _datasource.listDirectory();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        successMessage: 'Renamed to "${event.newName}".',
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Failed to rename: $e',
      ));
    }
  }

  // ── Upload ─────────────────────────────────────────────────────────────────

  Future<void> _onUploadFile(
      ExplorerUploadFile event, Emitter<ExplorerState> emit) async {
    try {
      await _ensureConnected();
      final result = await FilePicker.platform.pickFiles();
      if (result == null || result.files.single.path == null) return;
      final file = File(result.files.single.path!);
      await _datasource.uploadFile(file);
      final entries = await _datasource.listDirectory();
      emit(state.copyWith(
        status: ExplorerStatus.loaded,
        entries: _sortEntries(entries),
        successMessage: '${result.files.single.name} uploaded.',
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Upload failed: $e',
      ));
    }
  }

  // ── Download ───────────────────────────────────────────────────────────────

  Future<void> _onDownloadFile(
      ExplorerDownloadFile event, Emitter<ExplorerState> emit) async {
    try {
      await _ensureConnected();
      final dir = await getApplicationDocumentsDirectory();
      final localPath = '${dir.path}/${event.entry.name}';
      await _datasource.downloadFile(event.entry.name, localPath);
      emit(state.copyWith(
        successMessage: '"${event.entry.name}" saved to documents.',
      ));
    } catch (e) {
      emit(state.copyWith(
        status: ExplorerStatus.error,
        errorMessage: 'Download failed: $e',
      ));
    }
  }

  // ── Helpers ────────────────────────────────────────────────────────────────

  List<String> _pathToSegments(String path) {
    if (path == '/') return ['/'];
    final parts = path.split('/').where((s) => s.isNotEmpty).toList();
    return ['/', ...parts];
  }

  /// Directories first, then files, each group sorted alphabetically.
  List<FtpEntry> _sortEntries(List<FtpEntry> entries) {
    final dirs = entries.where((e) => e.isDirectory).toList()
      ..sort((a, b) => a.name.compareTo(b.name));
    final files = entries.where((e) => !e.isDirectory).toList()
      ..sort((a, b) => a.name.compareTo(b.name));
    return [...dirs, ...files];
  }
}
