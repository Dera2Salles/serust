import 'package:arosaina/engine/transfert/transfer_tree_node.dart';
import 'package:arosaina/presentation/bloc/transfer/transfer_bloc.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class TransferTreeScreen extends StatelessWidget {
  final String fileId;
  final String fileName;

  const TransferTreeScreen({
    super.key,
    required this.fileId,
    required this.fileName,
  });

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
                    AroIconButton(
                      icon: Icons.arrow_back,
                      onTap: () => Navigator.pop(context),
                    ),
                    SizedBox(width: context.rs(16)),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            "Transfer Tree",
                            style: Theme.of(context).textTheme.titleLarge
                                ?.copyWith(fontWeight: FontWeight.bold),
                          ),
                          Text(
                            fileName,
                            style: TextStyle(
                              fontSize: context.rf(14),
                              color: AppTheme.neutral500,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),

              Expanded(
                child: BlocBuilder<TransferBloc, TransferState>(
                  builder: (context, state) {
                    if (state is TransferInProgress) {
                      return const Center(child: CircularProgressIndicator());
                    } else if (state is TransferTreeLoaded) {
                      if (state.tree.isEmpty) {
                        return _buildEmptyState(context);
                      }
                      return SingleChildScrollView(
                        padding: EdgeInsets.all(context.rs(24)),
                        scrollDirection: Axis.horizontal,
                        child: SingleChildScrollView(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: state.tree
                                .map((node) => _TreeNodeWidget(node: node))
                                .toList(),
                          ),
                        ),
                      );
                    } else if (state is TransferFailure) {
                      return Center(child: Text("Error: ${state.message}"));
                    }

                    // Trigger load if initial or unrelated state
                    context.read<TransferBloc>().add(FetchTransferTree(fileId));
                    return const Center(child: CircularProgressIndicator());
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.account_tree_outlined,
            size: 64,
            color: AppTheme.neutral300,
          ),
          SizedBox(height: 16),
          Text(
            "No propagation data found",
            style: TextStyle(color: AppTheme.neutral500),
          ),
        ],
      ),
    );
  }
}

class _TreeNodeWidget extends StatelessWidget {
  final TransferTreeNode node;

  const _TreeNodeWidget({required this.node});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            // Node Card
            SoftCard(
              padding: EdgeInsets.symmetric(
                horizontal: context.rs(16),
                vertical: context.rs(12),
              ),
              borderRadius: 12,
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: context.rs(32),
                    height: context.rs(32),
                    decoration: BoxDecoration(
                      color: _getStatusColor(
                        node.status,
                      ).withValues(alpha: 0.1),
                      shape: BoxShape.circle,
                    ),
                    child: Icon(
                      _getStatusIcon(node.status),
                      color: _getStatusColor(node.status),
                      size: context.rs(16),
                    ),
                  ),
                  SizedBox(width: context.rs(12)),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        "User ${node.receiverId.substring(0, 8)}",
                        style: TextStyle(
                          fontSize: context.rf(14),
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      Text(
                        "Level ${node.transferLevel} • ${node.status}",
                        style: TextStyle(
                          fontSize: context.rf(11),
                          color: AppTheme.neutral500,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),

        if (node.children.isNotEmpty)
          Padding(
            padding: EdgeInsets.only(left: context.rs(24)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Vertical connector line
                Container(
                  width: 2,
                  height: context.rs(20),
                  color: AppTheme.neutral300,
                ),
                ...node.children.map(
                  (child) => Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Horizontal connector line
                      Container(
                        width: context.rs(20),
                        height: 2,
                        margin: EdgeInsets.only(top: context.rs(24)),
                        color: AppTheme.neutral300,
                      ),
                      _TreeNodeWidget(node: child),
                    ],
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }

  Color _getStatusColor(String status) {
    switch (status.toLowerCase()) {
      case 'completed':
        return AppTheme.emerald;
      case 'failed':
        return AppTheme.rose;
      case 'pending':
        return AppTheme.amber;
      default:
        return AppTheme.indigo;
    }
  }

  IconData _getStatusIcon(String status) {
    switch (status.toLowerCase()) {
      case 'completed':
        return Icons.check_circle_rounded;
      case 'failed':
        return Icons.error_rounded;
      case 'pending':
        return Icons.hourglass_bottom_rounded;
      default:
        return Icons.share_rounded;
    }
  }
}
