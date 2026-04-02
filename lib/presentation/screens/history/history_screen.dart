import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/models/transfer_history_model.dart';
import 'package:arosaina/engine/services/interfaces/i_history_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/screens/transfer/transfer_tree_screen.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

class HistoryScreen extends StatelessWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      backgroundColor: isDark
          ? AppTheme.backgroundDark
          : AppTheme.backgroundLight,
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
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
                      'History',
                      style: TextStyle(
                        fontSize: context.rf(22),
                        fontWeight: FontWeight.w800,
                        letterSpacing: -0.5,
                        color: isDark ? Colors.white : AppTheme.neutral900,
                      ),
                    ),
                  ),
                  AroIconButton(
                    icon: Icons.delete_outline_rounded,
                    onTap: () => _showClearDialog(context),
                    bgColor: AppTheme.rose.withValues(alpha: 0.1),
                    iconColor: AppTheme.rose,
                  ),
                ],
              ),
            ),

            SizedBox(height: context.rs(16)),

            Expanded(
              child: StreamBuilder<List<TransferHistoryModel>>(
                stream: sl<IHistoryService>().historyStream,
                initialData: sl<IHistoryService>().getHistory(),
                builder: (context, snap) {
                  final list = snap.data ?? [];
                  if (list.isEmpty) {
                    return Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Container(
                            width: 72,
                            height: 72,
                            decoration: BoxDecoration(
                              color: isDark
                                  ? AppTheme.neutral700
                                  : AppTheme.neutral100,
                              borderRadius: BorderRadius.circular(18),
                            ),
                            child: Icon(
                              Icons.history_rounded,
                              size: 36,
                              color: AppTheme.neutral300,
                            ),
                          ),
                          SizedBox(height: context.rs(16)),
                          Text(
                            'No transfers yet',
                            style: TextStyle(
                              fontSize: context.rf(15),
                              fontWeight: FontWeight.w700,
                              color: AppTheme.neutral500,
                            ),
                          ),
                        ],
                      ),
                    );
                  }

                  // Group by date
                  final grouped = _groupByDate(list);

                  return ListView.builder(
                    padding: EdgeInsets.symmetric(horizontal: context.rs(20)),
                    itemCount: grouped.length,
                    itemBuilder: (_, i) {
                      final entry = grouped[i];
                      if (entry is String) {
                        return Padding(
                          padding: EdgeInsets.symmetric(
                            vertical: context.rs(12),
                          ),
                          child: Text(
                            entry,
                            style: TextStyle(
                              fontSize: context.rf(11),
                              fontWeight: FontWeight.w800,
                              letterSpacing: 1.2,
                              color: AppTheme.neutral400,
                            ),
                          ),
                        );
                      }
                      return Padding(
                        padding: EdgeInsets.only(bottom: context.rs(8)),
                        child: _HistoryCard(
                          history: entry as TransferHistoryModel,
                        ),
                      );
                    },
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  List<dynamic> _groupByDate(List<TransferHistoryModel> list) {
    final result = <dynamic>[];
    String? lastDate;
    for (final h in list) {
      final dateStr = _dateLabel(h.timestamp);
      if (dateStr != lastDate) {
        result.add(dateStr);
        lastDate = dateStr;
      }
      result.add(h);
    }
    return result;
  }

  String _dateLabel(DateTime d) {
    final now = DateTime.now();
    if (now.difference(d).inDays == 0) return 'TODAY';
    if (now.difference(d).inDays == 1) return 'YESTERDAY';
    return DateFormat('MMM d, yyyy').format(d).toUpperCase();
  }

  void _showClearDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (_) => AlertDialog(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
        title: const Text(
          'Clear History',
          style: TextStyle(fontWeight: FontWeight.w800),
        ),
        content: const Text('This will remove all transfer records.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              sl<IHistoryService>().clearHistory();
              Navigator.pop(context);
            },
            child: const Text('Clear', style: TextStyle(color: AppTheme.rose)),
          ),
        ],
      ),
    );
  }
}

class _HistoryCard extends StatelessWidget {
  final TransferHistoryModel history;
  const _HistoryCard({required this.history});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final primary = Theme.of(context).primaryColor;
    final isSent = history.isSent;
    final color = isSent ? primary : AppTheme.emerald;

    return Container(
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
          Container(
            width: context.rs(44),
            height: context.rs(44),
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(
              isSent ? Icons.upload_rounded : Icons.download_rounded,
              color: color,
              size: context.rs(22),
            ),
          ),
          SizedBox(width: context.rs(12)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  history.fileName,
                  style: TextStyle(
                    fontSize: context.rf(13),
                    fontWeight: FontWeight.w700,
                    color: isDark ? Colors.white : AppTheme.neutral900,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                SizedBox(height: context.rs(2)),
                Text(
                  '${_formatSize(history.fileSize)}  ·  ${DateFormat('HH:mm').format(history.timestamp)}',
                  style: TextStyle(
                    fontSize: context.rf(11),
                    color: AppTheme.neutral500,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
          ),
          AroChip(label: isSent ? 'SENT' : 'RECV', color: color, small: true),
          if (history.fileId != null) ...[
            SizedBox(width: context.rs(8)),
            IconButton(
              icon: Icon(
                Icons.account_tree_outlined,
                color: AppTheme.neutral400,
                size: 20,
              ),
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (context) => TransferTreeScreen(
                      fileId: history.fileId!,
                      fileName: history.fileName,
                    ),
                  ),
                );
              },
            ),
          ],
        ],
      ),
    );
  }

  String _formatSize(int b) {
    if (b < 1024) return '$b B';
    if (b < 1024 * 1024) return '${(b / 1024).toStringAsFixed(1)} KB';
    return '${(b / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
