import 'package:arosaina/presentation/services/selectable_item.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/screens/send/pre_send_screen.dart';
import 'package:arosaina/presentation/screens/send/tabs/category_tabs.dart';
import 'package:arosaina/presentation/screens/send/tabs/file_explorer_tab.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

class FileSelectionScreen extends StatefulWidget {
  const FileSelectionScreen({super.key});

  @override
  State<FileSelectionScreen> createState() => _FileSelectionScreenState();
}

class _FileSelectionScreenState extends State<FileSelectionScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final List<SelectableItem> _selectedFiles = [];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  void _onFileSelected(SelectableItem item) {
    setState(() {
      if (_selectedFiles.any((f) => f.path == item.path)) {
        _selectedFiles.removeWhere((f) => f.path == item.path);
      } else {
        _selectedFiles.add(item);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Container(
        decoration: BoxDecoration(gradient: AppGradients.backgroundLight),
        child: SafeArea(
          child: Column(
            children: [
              // Header
              Padding(
                padding: EdgeInsets.all(context.rs(24)),
                child: Row(
                  children: [
                    GestureDetector(
                      onTap: () => Navigator.pop(context),
                      child: Container(
                        width: context.rs(40),
                        height: context.rs(40),
                        decoration: BoxDecoration(
                          color: Theme.of(context).colorScheme.surface,
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: AppTheme.neutral200,
                            width: 1,
                          ),
                        ),
                        child: Icon(Icons.arrow_back, size: context.rs(20)),
                      ),
                    ),
                    SizedBox(width: context.rs(16)),
                    Text(
                      "Select Files",
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const Spacer(),
                    if (_selectedFiles.isNotEmpty)
                      Container(
                        padding: EdgeInsets.symmetric(
                          horizontal: context.rs(12),
                          vertical: context.rs(6),
                        ),
                        decoration: BoxDecoration(
                          gradient: AppGradients.primaryPurple,
                          borderRadius: BorderRadius.circular(context.rs(12)),
                        ),
                        child: Text(
                          "${_selectedFiles.length}",
                          style: TextStyle(
                            color: Colors.white,
                            fontWeight: FontWeight.bold,
                            fontSize: context.rf(12),
                          ),
                        ),
                      ),
                  ],
                ),
              ),

              // Tab Bar
              Padding(
                padding: EdgeInsets.symmetric(horizontal: context.rs(24)),
                child: Container(
                  padding: EdgeInsets.all(context.rs(4)),
                  decoration: BoxDecoration(
                    color: AppTheme.neutral100,
                    borderRadius: BorderRadius.circular(context.rs(16)),
                  ),
                  child: TabBar(
                    controller: _tabController,
                    indicator: BoxDecoration(
                      color: Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(context.rs(12)),
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withValues(alpha: 0.05),
                          blurRadius: 4,
                          offset: const Offset(0, 2),
                        ),
                      ],
                    ),
                    dividerColor: Colors.transparent,
                    labelColor: Theme.of(context).primaryColor,
                    unselectedLabelColor: AppTheme.neutral500,
                    labelStyle: TextStyle(
                      fontWeight: FontWeight.bold,
                      fontSize: context.rf(14),
                    ),
                    tabs: const [
                      Tab(text: "Categories"),
                      Tab(text: "Explorer"),
                    ],
                  ),
                ),
              ),

              SizedBox(height: context.rs(16)),

              // Tab Views
              Expanded(
                child: TabBarView(
                  controller: _tabController,
                  children: [
                    CategoryTabs(
                      onFileSelected: (f) => _onFileSelected(f),
                      selectedFiles: _selectedFiles,
                    ),
                    FileExplorerTab(
                      onFileSelected: (f) => _onFileSelected(f),
                      selectedFiles: _selectedFiles,
                    ),
                  ],
                ),
              ),

              // Bottom Action Bar
              if (_selectedFiles.isNotEmpty) _buildBottomBar(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomBar() {
    return Container(
      padding: EdgeInsets.all(context.rs(24)),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.only(
          topLeft: Radius.circular(context.rs(32)),
          topRight: Radius.circular(context.rs(32)),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.05),
            blurRadius: 20,
            offset: const Offset(0, -5),
          ),
        ],
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  "${_selectedFiles.length} files selected",
                  style: TextStyle(
                    fontSize: context.rf(16),
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  _calculateTotalSize(),
                  style: TextStyle(
                    fontSize: context.rf(13),
                    color: AppTheme.neutral500,
                  ),
                ),
              ],
            ),
          ),
          GradientButton(
            text: "Next",
            width: context.rs(120),
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(
                  builder: (_) => PreSendScreen(selectedFiles: _selectedFiles),
                ),
              );
            },
          ),
        ],
      ),
    );
  }

  String _calculateTotalSize() {
    int totalBytes = 0;
    for (var file in _selectedFiles) {
      totalBytes += file.size;
    }

    if (totalBytes < 1024) return '$totalBytes B';
    if (totalBytes < 1024 * 1024) {
      return '${(totalBytes / 1024).toStringAsFixed(1)} KB';
    }
    if (totalBytes < 1024 * 1024 * 1024) {
      return '${(totalBytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(totalBytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}
