import 'dart:async';
import 'dart:io';

import 'package:arosaina/presentation/bloc/connection/connection_bloc.dart';
import 'package:arosaina/presentation/bloc/transfer/transfer_bloc.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart' hide ConnectionState;
import 'package:flutter_bloc/flutter_bloc.dart';

class SendScreen extends StatefulWidget {
  final List<File> selectedFiles;
  const SendScreen({super.key, required this.selectedFiles});

  @override
  State<SendScreen> createState() => _SendScreenState();
}

class _SendScreenState extends State<SendScreen> {
  bool _filesSent = false;

  @override
  void initState() {
    super.initState();
    _startConnection();
  }

  Future<void> _startConnection() async {
    final bloc = context.read<ConnectionBloc>();
    if (!bloc.hasActiveConnection) {
      bloc.add(ConnectToServer());
    } else {
      _sendFiles();
    }
  }

  void _sendFiles() {
    if (_filesSent) return;
    _filesSent = true;
    for (final file in widget.selectedFiles) {
      context.read<TransferBloc>().add(SendFile(file));
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return BlocListener<ConnectionBloc, ConnectionState>(
      listener: (context, state) {
        if (state is ConnectionFailure && mounted) {
          ScaffoldMessenger.of(
            context,
          ).showSnackBar(SnackBar(content: Text(state.error)));
        }
        if (state is ConnectionConnected) {
          _sendFiles();
        }
      },
      child: BlocConsumer<TransferBloc, TransferState>(
        listener: (context, state) {
          if (state is TransferFailure && mounted) {
            ScaffoldMessenger.of(
              context,
            ).showSnackBar(SnackBar(content: Text(state.message)));
          }
          if (state is TransferSuccess && mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text(state.message ?? "Transfer Success")),
            );
          }
          if (state is TransferInProgress) {
            // We usually hook the transfer stream directly in FTP context now,
            // but for simple UI representation:
          }
        },
        builder: (context, transferState) {
          final connState = context.watch<ConnectionBloc>().state;

          return Scaffold(
            backgroundColor: isDark
                ? AppTheme.backgroundDark
                : AppTheme.backgroundLight,
            body: SafeArea(
              child: Column(
                children: [
                  // Header
                  <Widget>[
                    Padding(
                      padding: EdgeInsets.fromLTRB(
                        context.rs(20),
                        context.rs(20),
                        context.rs(20),
                        0,
                      ),
                      child: Row(
                        children: [
                          AroIconButton(
                            icon: Icons.arrow_back_rounded,
                            onTap: () => Navigator.pop(context),
                          ),
                          SizedBox(width: context.rs(14)),
                          Text(
                            'Uploading Files',
                            style: TextStyle(
                              fontSize: context.rf(20),
                              fontWeight: FontWeight.w800,
                              letterSpacing: -0.3,
                              color: isDark
                                  ? Colors.white
                                  : AppTheme.neutral900,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ].first,

                  Expanded(
                    child: connState is ConnectionLoading
                        ? _buildLoadingState(context)
                        : _buildTransfer(context, transferState),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildLoadingState(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          CircularProgressIndicator(
            color: Theme.of(context).primaryColor,
            strokeWidth: 2.5,
          ),
          SizedBox(height: context.rs(20)),
          Text(
            'Connecting to FTP Server...',
            style: TextStyle(
              fontSize: context.rf(16),
              fontWeight: FontWeight.w700,
              color: isDark ? Colors.white : AppTheme.neutral900,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTransfer(BuildContext context, TransferState transferState) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(context.rs(20)),
      child: Column(
        children: [
          Container(
            padding: EdgeInsets.all(context.rs(16)),
            decoration: BoxDecoration(
              color: AppTheme.emerald.withValues(alpha: 0.08),
              borderRadius: BorderRadius.circular(14),
              border: Border.all(
                color: AppTheme.emerald.withValues(alpha: 0.2),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  Icons.check_circle_rounded,
                  color: AppTheme.emerald,
                  size: 22,
                ),
                SizedBox(width: context.rs(10)),
                Text(
                  'Connected — Sending files',
                  style: TextStyle(
                    fontSize: context.rf(14),
                    fontWeight: FontWeight.w700,
                    color: AppTheme.emerald,
                  ),
                ),
              ],
            ),
          ),
          SizedBox(height: context.rs(16)),
          Text(
            "Transfer progress tracking is abstracted pending server metrics.",
          ),
        ],
      ),
    );
  }
}
