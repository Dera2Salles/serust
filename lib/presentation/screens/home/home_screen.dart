import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/models/transfer_history_model.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/engine/services/interfaces/i_history_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/screens/send/file_selection_screen.dart';
import 'package:arosaina/presentation/screens/explorer/explorer_screen.dart';
import 'package:arosaina/presentation/screens/history/history_screen.dart';
import 'package:arosaina/presentation/screens/received_files/received_files_screen.dart';
import 'package:arosaina/presentation/screens/settings/settings_screen.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _currentIndex = 0;

  final List<Widget> _screens = const [
    _HomeTab(),
    ExplorerScreen(),
    HistoryScreen(),
    SettingsScreen(),
  ];

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      body: IndexedStack(index: _currentIndex, children: _screens),
      bottomNavigationBar: Container(
        decoration: BoxDecoration(
          color: isDark ? AppTheme.surfaceDark : Colors.white,
          border: Border(
            top: BorderSide(
              color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
              width: 1,
            ),
          ),
        ),
        child: SafeArea(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceAround,
              children: [
                _NavItem(
                  icon: Icons.home_rounded,
                  label: 'Home',
                  index: 0,
                  current: _currentIndex,
                  onTap: (i) => setState(() => _currentIndex = i),
                ),
                _NavItem(
                  icon: Icons.folder_open_rounded,
                  label: 'Explorer',
                  index: 1,
                  current: _currentIndex,
                  onTap: (i) => setState(() => _currentIndex = i),
                ),
                _NavItem(
                  icon: Icons.history_rounded,
                  label: 'History',
                  index: 2,
                  current: _currentIndex,
                  onTap: (i) => setState(() => _currentIndex = i),
                ),
                _NavItem(
                  icon: Icons.settings_rounded,
                  label: 'Settings',
                  index: 3,
                  current: _currentIndex,
                  onTap: (i) => setState(() => _currentIndex = i),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _NavItem extends StatelessWidget {
  final IconData icon;
  final String label;
  final int index;
  final int current;
  final void Function(int) onTap;

  const _NavItem({
    required this.icon,
    required this.label,
    required this.index,
    required this.current,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final active = index == current;
    final color = active ? Theme.of(context).primaryColor : AppTheme.neutral400;

    return GestureDetector(
      onTap: () => onTap(index),
      behavior: HitTestBehavior.opaque,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 6),
            decoration: BoxDecoration(
              color: active
                  ? Theme.of(context).primaryColor.withValues(alpha: 0.12)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Icon(icon, color: color, size: 22),
          ),
          const SizedBox(height: 2),
          Text(
            label,
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.w600,
              color: color,
              letterSpacing: 0.3,
            ),
          ),
        ],
      ),
    );
  }
}

// ─── Home Tab ─────────────────────────────────────────────────────────────────
class _HomeTab extends StatelessWidget {
  const _HomeTab();

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final user = sl<IAuthService>().getCurrentUser();
    final firstName = user?.username.split(' ').first ?? 'User';
    final primary = Theme.of(context).primaryColor;

    return Scaffold(
      backgroundColor: isDark
          ? AppTheme.backgroundDark
          : AppTheme.backgroundLight,
      body: SafeArea(
        child: Column(
          children: [
            // ── Top Bar ──────────────────────────────────────────────────────
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
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Hello, $firstName 👋',
                          style: TextStyle(
                            fontSize: context.rf(22),
                            fontWeight: FontWeight.w800,
                            color: isDark ? Colors.white : AppTheme.neutral900,
                            letterSpacing: -0.5,
                          ),
                        ),
                        Text(
                          'Secure file transfer',
                          style: TextStyle(
                            fontSize: context.rf(13),
                            color: AppTheme.neutral500,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                      ],
                    ),
                  ),
                  // Avatar placeholder
                  Container(
                    width: context.rs(44),
                    height: context.rs(44),
                    decoration: BoxDecoration(
                      color: primary,
                      borderRadius: BorderRadius.circular(14),
                    ),
                    child: Center(
                      child: Text(
                        firstName[0].toUpperCase(),
                        style: const TextStyle(
                          color: Colors.white,
                          fontWeight: FontWeight.w800,
                          fontSize: 18,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),

            Expanded(
              child: SingleChildScrollView(
                padding: EdgeInsets.all(context.rs(20)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(height: context.rs(24)),

                    // ── Hero Banner ──────────────────────────────────────────
                    Container(
                      width: double.infinity,
                      padding: EdgeInsets.all(context.rs(24)),
                      decoration: BoxDecoration(
                        color: primary,
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Row(
                        children: [
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  'AES-256\nEncryption',
                                  style: TextStyle(
                                    fontSize: context.rf(22),
                                    fontWeight: FontWeight.w800,
                                    color: Colors.white,
                                    height: 1.2,
                                    letterSpacing: -0.5,
                                  ),
                                ),
                                SizedBox(height: context.rs(8)),
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 10,
                                    vertical: 4,
                                  ),
                                  decoration: BoxDecoration(
                                    color: Colors.white.withValues(alpha: 0.2),
                                    borderRadius: BorderRadius.circular(8),
                                  ),
                                  child: const Text(
                                    'End-to-End Secure',
                                    style: TextStyle(
                                      color: Colors.white,
                                      fontSize: 11,
                                      fontWeight: FontWeight.w600,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                          Icon(
                            Icons.shield_rounded,
                            size: context.rs(72),
                            color: Colors.white.withValues(alpha: 0.25),
                          ),
                        ],
                      ),
                    ),

                    SizedBox(height: context.rs(28)),

                    // ── Main Actions ─────────────────────────────────────────
                    Text(
                      'TRANSFER',
                      style: TextStyle(
                        fontSize: context.rf(11),
                        fontWeight: FontWeight.w800,
                        letterSpacing: 1.4,
                        color: AppTheme.neutral400,
                      ),
                    ),
                    SizedBox(height: context.rs(12)),

                    Row(
                      children: [
                        Expanded(
                          child: _ActionCard(
                            icon: Icons.upload_rounded,
                            label: 'Send',
                            subtitle: 'Encrypt & send',
                            color: primary,
                            onTap: () => Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (_) => const FileSelectionScreen(),
                              ),
                            ),
                          ),
                        ),
                        SizedBox(width: context.rs(12)),
                        Expanded(
                          child: _ActionCard(
                            icon: Icons.download_rounded,
                            label: 'Receive',
                            subtitle: 'Show QR code',
                            color: AppTheme.emerald,
                            onTap: () => Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (_) => const ReceivedFilesScreen(),
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),

                    SizedBox(height: context.rs(28)),

                    // ── Stats Row ────────────────────────────────────────────
                    _StatsRow(),

                    SizedBox(height: context.rs(28)),

                    // ── Recent transfers ─────────────────────────────────────
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'RECENT',
                          style: TextStyle(
                            fontSize: context.rf(11),
                            fontWeight: FontWeight.w800,
                            letterSpacing: 1.4,
                            color: AppTheme.neutral400,
                          ),
                        ),
                        GestureDetector(
                          onTap: () => Navigator.push(
                            context,
                            MaterialPageRoute(
                              builder: (_) => const HistoryScreen(),
                            ),
                          ),
                          child: Text(
                            'See all',
                            style: TextStyle(
                              fontSize: context.rf(12),
                              fontWeight: FontWeight.w700,
                              color: primary,
                            ),
                          ),
                        ),
                      ],
                    ),
                    SizedBox(height: context.rs(12)),

                    StreamBuilder<List<TransferHistoryModel>>(
                      stream: sl<IHistoryService>().historyStream,
                      initialData: sl<IHistoryService>().getHistory(),
                      builder: (context, snap) {
                        final list = snap.data ?? [];
                        if (list.isEmpty) {
                          return _EmptyRecent();
                        }
                        final recent = list.take(4).toList();
                        return Column(
                          children: recent
                              .map((h) => _RecentItem(history: h))
                              .toList(),
                        );
                      },
                    ),

                    SizedBox(height: context.rs(16)),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ActionCard extends StatelessWidget {
  final IconData icon;
  final String label;
  final String subtitle;
  final Color color;
  final VoidCallback onTap;

  const _ActionCard({
    required this.icon,
    required this.label,
    required this.subtitle,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: EdgeInsets.all(context.rs(18)),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(18),
          border: Border.all(color: color.withValues(alpha: 0.15), width: 1),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: context.rs(44),
              height: context.rs(44),
              decoration: BoxDecoration(
                color: color,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Icon(icon, color: Colors.white, size: context.rs(22)),
            ),
            SizedBox(height: context.rs(12)),
            Text(
              label,
              style: TextStyle(
                fontSize: context.rf(16),
                fontWeight: FontWeight.w800,
                color: color,
              ),
            ),
            Text(
              subtitle,
              style: TextStyle(
                fontSize: context.rf(11),
                color: AppTheme.neutral500,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatsRow extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final history = sl<IHistoryService>().getHistory();
    final sent = history.where((h) => h.isSent).length;
    final received = history.where((h) => !h.isSent).length;
    final totalMb =
        history.fold<int>(0, (s, h) => s + h.fileSize) ~/ (1024 * 1024);

    return Row(
      children: [
        Expanded(
          child: _StatCard(
            value: '$sent',
            label: 'Sent',
            color: Theme.of(context).primaryColor,
          ),
        ),
        SizedBox(width: context.rs(10)),
        Expanded(
          child: _StatCard(
            value: '$received',
            label: 'Received',
            color: AppTheme.emerald,
          ),
        ),
        SizedBox(width: context.rs(10)),
        Expanded(
          child: _StatCard(
            value: '${totalMb}MB',
            label: 'Total',
            color: AppTheme.violet,
          ),
        ),
      ],
    );
  }
}

class _StatCard extends StatelessWidget {
  final String value;
  final String label;
  final Color color;

  const _StatCard({
    required this.value,
    required this.label,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Container(
      padding: EdgeInsets.symmetric(
        vertical: context.rs(14),
        horizontal: context.rs(12),
      ),
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
          Text(
            value,
            style: TextStyle(
              fontSize: context.rf(20),
              fontWeight: FontWeight.w800,
              color: color,
              letterSpacing: -0.5,
            ),
          ),
          Text(
            label,
            style: TextStyle(
              fontSize: context.rf(11),
              color: AppTheme.neutral500,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyRecent extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Container(
      padding: EdgeInsets.all(context.rs(32)),
      decoration: BoxDecoration(
        color: isDark ? AppTheme.neutral800 : AppTheme.neutral100,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Center(
        child: Column(
          children: [
            Icon(
              Icons.inbox_rounded,
              size: 40,
              color: isDark ? AppTheme.neutral600 : AppTheme.neutral300,
            ),
            SizedBox(height: context.rs(8)),
            Text(
              'No transfers yet',
              style: TextStyle(
                color: AppTheme.neutral500,
                fontWeight: FontWeight.w600,
                fontSize: context.rf(13),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _RecentItem extends StatelessWidget {
  final TransferHistoryModel history;

  const _RecentItem({required this.history});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final isSent = history.isSent;
    final color = isSent ? Theme.of(context).primaryColor : AppTheme.emerald;
    final icon = isSent ? Icons.upload_rounded : Icons.download_rounded;

    return Container(
      margin: EdgeInsets.only(bottom: context.rs(8)),
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
            width: context.rs(40),
            height: context.rs(40),
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(icon, color: color, size: context.rs(20)),
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
                Text(
                  _formatSize(history.fileSize),
                  style: TextStyle(
                    fontSize: context.rf(11),
                    color: AppTheme.neutral500,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
          ),
          AroChip(
            label: isSent ? 'Sent' : 'Received',
            color: color,
            small: true,
          ),
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
