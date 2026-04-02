import 'package:arosaina/engine/models/transfer_update.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:flutter/material.dart';

class TransferProgressCard extends StatelessWidget {
  final TransferUpdate update;
  const TransferProgressCard({super.key, required this.update});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final primary = Theme.of(context).primaryColor;

    final isEncrypting = update.status == TransferStatus.encrypting;
    final isCompleted = update.status == TransferStatus.completed;
    final isFailed = update.status == TransferStatus.failed;
    final progress = update.totalBytes > 0
        ? (update.bytesTransferred / update.totalBytes).clamp(0.0, 1.0)
        : 0.0;

    Color statusColor;
    if (isCompleted) {
      statusColor = AppTheme.emerald;
    } else if (isFailed) {
      statusColor = AppTheme.rose;
    } else if (isEncrypting) {
      statusColor = AppTheme.amber;
    } else {
      statusColor = primary;
    }

    String statusLabel;
    if (isCompleted) {
      statusLabel = 'Done';
    } else if (isFailed) {
      statusLabel = 'Failed';
    } else if (isEncrypting) {
      statusLabel = 'Encrypting...';
    } else {
      statusLabel = '${(progress * 100).toStringAsFixed(0)}%';
    }

    return Container(
      padding: EdgeInsets.all(context.rs(14)),
      decoration: BoxDecoration(
        color: isDark ? AppTheme.neutral800 : Colors.white,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(
          color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: context.rs(36),
                height: context.rs(36),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(
                  update.isIncoming
                      ? Icons.download_rounded
                      : Icons.upload_rounded,
                  color: statusColor,
                  size: context.rs(18),
                ),
              ),
              SizedBox(width: context.rs(10)),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      update.fileName,
                      style: TextStyle(
                        fontSize: context.rf(13),
                        fontWeight: FontWeight.w700,
                        color: isDark ? Colors.white : AppTheme.neutral900,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    Text(
                      _formatSize(update.totalBytes),
                      style: TextStyle(
                        fontSize: context.rf(11),
                        color: AppTheme.neutral500,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ],
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  statusLabel,
                  style: TextStyle(
                    fontSize: context.rf(11),
                    fontWeight: FontWeight.w700,
                    color: statusColor,
                  ),
                ),
              ),
            ],
          ),

          if (!isCompleted && !isFailed) ...[
            SizedBox(height: context.rs(10)),
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: isEncrypting ? null : progress,
                minHeight: 4,
                backgroundColor: statusColor.withValues(alpha: 0.1),
                valueColor: AlwaysStoppedAnimation(statusColor),
              ),
            ),
          ],
        ],
      ),
    );
  }

  String _formatSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
