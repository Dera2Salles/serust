import 'dart:io';

import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/services/selectable_item.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

import 'send_screen.dart';

/// Premium Pre-Send Screen with Option Cards
class PreSendScreen extends StatefulWidget {
  final List<SelectableItem> selectedFiles;

  const PreSendScreen({super.key, required this.selectedFiles});

  @override
  State<PreSendScreen> createState() => _PreSendScreenState();
}

class _PreSendScreenState extends State<PreSendScreen> {
  int _downloadLimit = 1;

  void _showCustomLimitDialog() {
    final controller = TextEditingController(
      text: _downloadLimit > 0 && ![1, 5, 10, -1].contains(_downloadLimit)
          ? _downloadLimit.toString()
          : "",
    );
    String? errorText;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => Dialog(
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(context.rs(24)),
          ),
          child: Container(
            constraints: BoxConstraints(maxWidth: context.rs(400)),
            padding: EdgeInsets.all(context.rs(24)),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  "Custom Limit",
                  style: TextStyle(
                    fontSize: context.rf(20),
                    fontWeight: FontWeight.bold,
                  ),
                ),
                SizedBox(height: context.rs(8)),
                Text(
                  "Set maximum number of downloads",
                  style: TextStyle(
                    fontSize: context.rf(14),
                    color: AppTheme.neutral500,
                  ),
                ),
                SizedBox(height: context.rs(24)),
                ModernTextField(
                  label: "Downloads",
                  hintText: "e.g. 50",
                  keyboardType: TextInputType.number,
                  controller: controller,
                  onChanged: (value) {
                    setDialogState(() {
                      final limit = int.tryParse(value);
                      if (value.isEmpty) {
                        errorText = null;
                      } else if (limit == null || limit <= 0) {
                        errorText = "Enter a valid number";
                      } else {
                        errorText = null;
                      }
                    });
                  },
                ),
                if (errorText != null)
                  Padding(
                    padding: EdgeInsets.only(
                      top: context.rs(8),
                      left: context.rs(12),
                    ),
                    child: Text(
                      errorText!,
                      style: TextStyle(
                        color: AppTheme.coral,
                        fontSize: context.rf(12),
                      ),
                    ),
                  ),
                SizedBox(height: context.rs(32)),
                Row(
                  children: [
                    Expanded(
                      child: TextButton(
                        onPressed: () => Navigator.pop(context),
                        child: Text(
                          "Cancel",
                          style: TextStyle(color: AppTheme.neutral500),
                        ),
                      ),
                    ),
                    SizedBox(width: context.rs(16)),
                    Expanded(
                      child: GradientButton(
                        text: "Save",
                        height: context.rs(48),
                        onPressed:
                            errorText == null && controller.text.isNotEmpty
                            ? () {
                                final limit = int.tryParse(controller.text);
                                if (limit != null && limit > 0) {
                                  setState(() => _downloadLimit = limit);
                                  Navigator.pop(context);
                                }
                              }
                            : null,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
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
                      "Transfer Options",
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
              ),

              Expanded(
                child: SingleChildScrollView(
                  padding: EdgeInsets.symmetric(horizontal: context.rs(24)),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // File Summary Illustration Card
                      IllustrationCard(
                        aspectRatio: 21 / 9,
                        gradient: AppGradients.overlayPurple,
                        child: Row(
                          children: [
                            SizedBox(width: context.rs(24)),
                            Container(
                              width: context.rs(64),
                              height: context.rs(64),
                              decoration: BoxDecoration(
                                color: Colors.white.withValues(alpha: 0.2),
                                shape: BoxShape.circle,
                              ),
                              child: Icon(
                                Icons.folder_copy_rounded,
                                color: Colors.white,
                                size: context.rs(32),
                              ),
                            ),
                            SizedBox(width: context.rs(20)),
                            Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  "${widget.selectedFiles.length} items",
                                  style: TextStyle(
                                    fontSize: context.rf(20),
                                    fontWeight: FontWeight.bold,
                                    color: Colors.white,
                                  ),
                                ),
                                Text(
                                  _calculateTotalSize(),
                                  style: TextStyle(
                                    fontSize: context.rf(14),
                                    color: Colors.white.withValues(alpha: 0.8),
                                  ),
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),

                      SizedBox(height: context.rs(48)),

                      // Download Limit Section
                      Text(
                        "Download Limit",
                        style: TextStyle(
                          fontSize: context.rf(16),
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      SizedBox(height: context.rs(8)),
                      Text(
                        "How many times can these files be downloaded?",
                        style: TextStyle(
                          fontSize: context.rf(14),
                          color: AppTheme.neutral500,
                        ),
                      ),
                      SizedBox(height: context.rs(16)),

                      Wrap(
                        spacing: context.rs(12),
                        runSpacing: context.rs(12),
                        children: [
                          _LimitOption(
                            label: "1",
                            isSelected: _downloadLimit == 1,
                            onTap: () => setState(() => _downloadLimit = 1),
                          ),
                          _LimitOption(
                            label: "5",
                            isSelected: _downloadLimit == 5,
                            onTap: () => setState(() => _downloadLimit = 5),
                          ),
                          _LimitOption(
                            label: "10",
                            isSelected: _downloadLimit == 10,
                            onTap: () => setState(() => _downloadLimit = 10),
                          ),
                          _LimitOption(
                            label: "Unlimited",
                            isSelected: _downloadLimit == -1,
                            onTap: () => setState(() => _downloadLimit = -1),
                          ),
                          _LimitOption(
                            label:
                                _downloadLimit > 10 ||
                                    (_downloadLimit > 1 &&
                                        ![
                                          1,
                                          5,
                                          10,
                                          -1,
                                        ].contains(_downloadLimit))
                                ? "$_downloadLimit"
                                : "Custom",
                            isSelected:
                                _downloadLimit > 10 ||
                                (_downloadLimit > 1 &&
                                    ![1, 5, 10, -1].contains(_downloadLimit)),
                            onTap: _showCustomLimitDialog,
                            icon: Icons.edit_rounded,
                          ),
                        ],
                      ),

                      SizedBox(height: context.rs(48)),

                      // Security Info Card
                      SoftCard(
                        padding: EdgeInsets.all(context.rs(20)),
                        child: Row(
                          children: [
                            Container(
                              width: context.rs(48),
                              height: context.rs(48),
                              decoration: BoxDecoration(
                                color: AppTheme.green.withValues(alpha: 0.1),
                                borderRadius: BorderRadius.circular(
                                  context.rs(12),
                                ),
                              ),
                              child: Icon(
                                Icons.security_rounded,
                                color: AppTheme.green,
                                size: context.rs(24),
                              ),
                            ),
                            SizedBox(width: context.rs(16)),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    "Backend Transfer",
                                    style: TextStyle(
                                      fontWeight: FontWeight.bold,
                                      fontSize: context.rf(15),
                                    ),
                                  ),
                                  Text(
                                    "Server-side management enabled",
                                    style: TextStyle(
                                      fontSize: context.rf(13),
                                      color: AppTheme.neutral500,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                            Container(
                              padding: EdgeInsets.symmetric(
                                horizontal: context.rs(10),
                                vertical: context.rs(4),
                              ),
                              decoration: BoxDecoration(
                                color: AppTheme.green.withValues(alpha: 0.1),
                                borderRadius: BorderRadius.circular(
                                  context.rs(8),
                                ),
                              ),
                              child: Text(
                                "Centralized",
                                style: TextStyle(
                                  color: AppTheme.green,
                                  fontWeight: FontWeight.bold,
                                  fontSize: context.rf(10),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),

                      SizedBox(height: context.rs(48)),

                      Text(
                        "Selected Files",
                        style: TextStyle(
                          fontSize: context.rf(16),
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      SizedBox(height: context.rs(16)),

                      // File List
                      ListView.builder(
                        shrinkWrap: true,
                        physics: const NeverScrollableScrollPhysics(),
                        itemCount: widget.selectedFiles.length,
                        itemBuilder: (context, index) {
                          final item = widget.selectedFiles[index];
                          return Padding(
                            padding: EdgeInsets.only(bottom: context.rs(12)),
                            child: _FileItem(item: item),
                          );
                        },
                      ),

                      SizedBox(height: context.rs(24)),
                    ],
                  ),
                ),
              ),

              // Bottom Button
              Padding(
                padding: EdgeInsets.all(context.rs(24)),
                child: GradientButton(
                  text: "Select Recipient",
                  onPressed: () {
                    Navigator.push(
                      context,
                      MaterialPageRoute(
                        builder: (_) => SendScreen(
                          selectedFiles: widget.selectedFiles
                              .map((item) => File(item.path))
                              .toList(),
                        ),
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _calculateTotalSize() {
    int totalBytes = widget.selectedFiles.fold(
      0,
      (sum, item) => sum + item.size,
    );
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

class _LimitOption extends StatelessWidget {
  final String label;
  final bool isSelected;
  final VoidCallback onTap;
  final IconData? icon;

  const _LimitOption({
    required this.label,
    required this.isSelected,
    required this.onTap,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: EdgeInsets.symmetric(
          horizontal: context.rs(20),
          vertical: context.rs(12),
        ),
        decoration: BoxDecoration(
          color: isSelected ? Theme.of(context).primaryColor : Colors.white,
          borderRadius: BorderRadius.circular(context.rs(16)),
          border: Border.all(
            color: isSelected
                ? Theme.of(context).primaryColor
                : AppTheme.neutral200,
            width: 1.5,
          ),
          boxShadow: isSelected
              ? [
                  BoxShadow(
                    color: Theme.of(
                      context,
                    ).primaryColor.withValues(alpha: 0.3),
                    blurRadius: 8,
                    offset: const Offset(0, 4),
                  ),
                ]
              : null,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (icon != null) ...[
              Icon(
                icon,
                size: context.rs(16),
                color: isSelected ? Colors.white : AppTheme.neutral500,
              ),
              SizedBox(width: context.rs(8)),
            ],
            Text(
              label,
              style: TextStyle(
                color: isSelected ? Colors.white : AppTheme.neutral700,
                fontWeight: FontWeight.bold,
                fontSize: context.rf(15),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _FileItem extends StatelessWidget {
  final SelectableItem item;

  const _FileItem({required this.item});

  @override
  Widget build(BuildContext context) {
    return SoftCard(
      padding: EdgeInsets.all(context.rs(12)),
      child: Row(
        children: [
          Container(
            width: context.rs(48),
            height: context.rs(48),
            decoration: BoxDecoration(
              color: AppTheme.neutral100,
              borderRadius: BorderRadius.circular(context.rs(12)),
            ),
            child: _buildThumbnail(),
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
        ],
      ),
    );
  }

  Widget _buildThumbnail() {
    if (item.thumbnail != null) {
      return ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: Image.memory(item.thumbnail!, fit: BoxFit.cover),
      );
    }
    return Icon(_getIconData(), color: AppTheme.neutral500, size: 24);
  }

  IconData _getIconData() {
    switch (item.type) {
      case SelectableItemType.image:
        return Icons.image_outlined;
      case SelectableItemType.video:
        return Icons.videocam_outlined;
      case SelectableItemType.audio:
        return Icons.music_note_outlined;
      case SelectableItemType.app:
        return Icons.android_outlined;
      case SelectableItemType.folder:
        return Icons.folder_outlined;
      default:
        return Icons.insert_drive_file_outlined;
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
