import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';
import 'package:arosaina/presentation/bloc/settings/settings_cubit.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:arosaina/presentation/widgets/server_config_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final user = sl<IAuthService>().getCurrentUser();
    final primary = Theme.of(context).primaryColor;

    return Scaffold(
      backgroundColor: isDark
          ? AppTheme.backgroundDark
          : AppTheme.backgroundLight,
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: EdgeInsets.fromLTRB(
                context.rs(20),
                context.rs(20),
                context.rs(20),
                0,
              ),
              child: Text(
                'Settings',
                style: TextStyle(
                  fontSize: context.rf(22),
                  fontWeight: FontWeight.w800,
                  letterSpacing: -0.5,
                  color: isDark ? Colors.white : AppTheme.neutral900,
                ),
              ),
            ),

            Expanded(
              child: ListView(
                padding: EdgeInsets.all(context.rs(20)),
                children: [
                  SizedBox(height: context.rs(8)),

                  // Profile card
                  if (user != null) ...[
                    Container(
                      padding: EdgeInsets.all(context.rs(18)),
                      decoration: BoxDecoration(
                        color: primary,
                        borderRadius: BorderRadius.circular(18),
                      ),
                      child: Row(
                        children: [
                          Container(
                            width: context.rs(52),
                            height: context.rs(52),
                            decoration: BoxDecoration(
                              color: Colors.white.withValues(alpha: 0.2),
                              borderRadius: BorderRadius.circular(14),
                            ),
                            child: Center(
                              child: Text(
                                user.username[0].toUpperCase(),
                                style: const TextStyle(
                                  color: Colors.white,
                                  fontSize: 24,
                                  fontWeight: FontWeight.w800,
                                ),
                              ),
                            ),
                          ),
                          SizedBox(width: context.rs(14)),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  user.username,
                                  style: const TextStyle(
                                    color: Colors.white,
                                    fontSize: 16,
                                    fontWeight: FontWeight.w800,
                                  ),
                                ),
                                Text(
                                  user.email,
                                  style: TextStyle(
                                    color: Colors.white.withValues(alpha: 0.75),
                                    fontSize: 12,
                                    fontWeight: FontWeight.w500,
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                    SizedBox(height: context.rs(20)),
                  ],

                  // Theme color section
                  _SectionLabel(label: 'APPEARANCE'),
                  SizedBox(height: context.rs(10)),
                  SoftCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Theme Color',
                          style: TextStyle(
                            fontSize: context.rf(14),
                            fontWeight: FontWeight.w700,
                            color: isDark ? Colors.white : AppTheme.neutral900,
                          ),
                        ),
                        SizedBox(height: context.rs(14)),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: AppTheme.themeColors.map((color) {
                            final isSelected =
                                Theme.of(context).primaryColor == color;
                            return GestureDetector(
                              onTap: () => context
                                  .read<SettingsCubit>()
                                  .setThemeColor(color),
                              child: AnimatedContainer(
                                duration: const Duration(milliseconds: 200),
                                width: context.rs(36),
                                height: context.rs(36),
                                decoration: BoxDecoration(
                                  color: color,
                                  borderRadius: BorderRadius.circular(10),
                                  border: isSelected
                                      ? Border.all(color: color, width: 3)
                                      : null,
                                  boxShadow: isSelected
                                      ? [
                                          BoxShadow(
                                            color: color.withValues(alpha: 0.4),
                                            blurRadius: 8,
                                            offset: const Offset(0, 2),
                                          ),
                                        ]
                                      : null,
                                ),
                                child: isSelected
                                    ? const Icon(
                                        Icons.check_rounded,
                                        color: Colors.white,
                                        size: 18,
                                      )
                                    : null,
                              ),
                            );
                          }).toList(),
                        ),
                      ],
                    ),
                  ),

                  SizedBox(height: context.rs(20)),

                  _SectionLabel(label: 'LANGUAGE'),
                  SizedBox(height: context.rs(10)),
                  SoftCard(
                    child: Row(
                      children: [
                        _LangButton(
                          label: 'English',
                          locale: const Locale('en'),
                        ),
                        SizedBox(width: context.rs(8)),
                        _LangButton(
                          label: 'Français',
                          locale: const Locale('fr'),
                        ),
                      ],
                    ),
                  ),

                  SizedBox(height: context.rs(20)),

                  _SectionLabel(label: 'ABOUT'),
                  SizedBox(height: context.rs(10)),
                  SoftCard(
                    child: Column(
                      children: [
                        _InfoRow(label: 'Version', value: '1.0.0'),
                        Divider(
                          height: context.rs(20),
                          color: isDark
                              ? AppTheme.neutral700
                              : AppTheme.neutral200,
                        ),
                        _InfoRow(label: 'Encryption', value: 'AES-256-GCM'),
                        Divider(
                          height: context.rs(20),
                          color: isDark
                              ? AppTheme.neutral700
                              : AppTheme.neutral200,
                        ),
                        _InfoRow(label: 'Protocol', value: 'NSD / HTTP'),
                      ],
                    ),
                  ),

                  SizedBox(height: context.rs(20)),

                  _SectionLabel(label: 'SERVER'),
                  SizedBox(height: context.rs(10)),
                  SoftCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    'Server URL',
                                    style: TextStyle(
                                      fontSize: context.rf(14),
                                      fontWeight: FontWeight.w700,
                                      color: isDark
                                          ? Colors.white
                                          : AppTheme.neutral900,
                                    ),
                                  ),
                                  SizedBox(height: context.rs(4)),
                                  Text(
                                    sl<ISettingsService>().apiBaseUrl,
                                    style: TextStyle(
                                      fontSize: context.rf(12),
                                      color: AppTheme.neutral500,
                                      fontWeight: FontWeight.w500,
                                    ),
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ],
                              ),
                            ),
                            SizedBox(width: context.rs(12)),
                            IconButton(
                              onPressed: () => ServerConfigDialog.show(context),
                              icon: Icon(
                                Icons.settings_ethernet_rounded,
                                color: primary,
                              ),
                              style: IconButton.styleFrom(
                                backgroundColor: primary.withValues(alpha: 0.1),
                                shape: RoundedRectangleBorder(
                                  borderRadius: BorderRadius.circular(10),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),

                  SizedBox(height: context.rs(20)),

                  // Logout
                  if (user != null)
                    GradientButton(
                      text: 'Sign Out',
                      color: AppTheme.rose,
                      onPressed: () async {
                        await sl<IAuthService>().logout();
                      },
                    ),

                  SizedBox(height: context.rs(16)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SectionLabel extends StatelessWidget {
  final String label;
  const _SectionLabel({required this.label});

  @override
  Widget build(BuildContext context) {
    return Text(
      label,
      style: TextStyle(
        fontSize: context.rf(11),
        fontWeight: FontWeight.w800,
        letterSpacing: 1.2,
        color: AppTheme.neutral400,
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;
  const _InfoRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: context.rf(13),
            fontWeight: FontWeight.w600,
            color: AppTheme.neutral500,
          ),
        ),
        Text(
          value,
          style: TextStyle(
            fontSize: context.rf(13),
            fontWeight: FontWeight.w700,
            color: isDark ? Colors.white : AppTheme.neutral900,
          ),
        ),
      ],
    );
  }
}

class _LangButton extends StatelessWidget {
  final String label;
  final Locale locale;
  const _LangButton({required this.label, required this.locale});

  @override
  Widget build(BuildContext context) {
    final primary = Theme.of(context).primaryColor;

    return Expanded(
      child: GestureDetector(
        onTap: () => context.read<SettingsCubit>().setLocale(locale),
        child: Container(
          padding: EdgeInsets.symmetric(vertical: context.rs(10)),
          decoration: BoxDecoration(
            color: primary.withValues(alpha: 0.08),
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: primary.withValues(alpha: 0.15)),
          ),
          child: Center(
            child: Text(
              label,
              style: TextStyle(
                fontSize: context.rf(13),
                fontWeight: FontWeight.w700,
                color: primary,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
