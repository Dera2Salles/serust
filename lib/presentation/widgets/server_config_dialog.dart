import 'package:arosaina/core/api_client.dart';
import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

class ServerConfigDialog extends StatefulWidget {
  const ServerConfigDialog({super.key});

  static Future<void> show(BuildContext context) {
    return showDialog(
      context: context,
      builder: (context) => const ServerConfigDialog(),
    );
  }

  @override
  State<ServerConfigDialog> createState() => _ServerConfigDialogState();
}

class _ServerConfigDialogState extends State<ServerConfigDialog> {
  late final TextEditingController _controller;
  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(
      text: sl<ISettingsService>().apiBaseUrl,
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    if (_formKey.currentState?.validate() ?? false) {
      final newUrl = _controller.text.trim();
      await sl<ISettingsService>().setApiBaseUrl(newUrl);
      sl<ApiClient>().updateBaseUrl(newUrl);
      if (mounted) Navigator.pop(context);
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final primary = Theme.of(context).primaryColor;

    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: EdgeInsets.all(context.rs(24)),
      child: Container(
        width: double.infinity,
        padding: EdgeInsets.all(context.rs(24)),
        decoration: BoxDecoration(
          color: isDark ? AppTheme.surfaceDark : Colors.white,
          borderRadius: BorderRadius.circular(24),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.2),
              blurRadius: 20,
              offset: const Offset(0, 10),
            ),
          ],
        ),
        child: Form(
          key: _formKey,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    padding: EdgeInsets.all(context.rs(8)),
                    decoration: BoxDecoration(
                      color: primary.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Icon(
                      Icons.dns_rounded,
                      color: primary,
                      size: 20,
                    ),
                  ),
                  SizedBox(width: context.rs(12)),
                  Text(
                    'Server Configuration',
                    style: TextStyle(
                      fontSize: context.rf(18),
                      fontWeight: FontWeight.w800,
                      color: isDark ? Colors.white : AppTheme.neutral900,
                    ),
                  ),
                ],
              ),
              SizedBox(height: context.rs(8)),
              Text(
                'Enter the API base URL for your server. Make sure it includes the protocol (http/https).',
                style: TextStyle(
                  fontSize: context.rf(13),
                  color: AppTheme.neutral500,
                ),
              ),
              SizedBox(height: context.rs(24)),
              ModernTextField(
                label: 'SERVER URL',
                hintText: 'http://192.168.1.100:8080',
                controller: _controller,
                prefixIcon: Icons.link_rounded,
                validator: (v) {
                  if (v == null || v.isEmpty) return 'Required';
                  if (!v.startsWith('http://') && !v.startsWith('https://')) {
                    return 'Must start with http:// or https://';
                  }
                  return null;
                },
              ),
              SizedBox(height: context.rs(24)),
              Row(
                children: [
                  Expanded(
                    child: TextButton(
                      onPressed: () => Navigator.pop(context),
                      style: TextButton.styleFrom(
                        padding: EdgeInsets.symmetric(vertical: context.rs(14)),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(12),
                        ),
                      ),
                      child: Text(
                        'Cancel',
                        style: TextStyle(
                          color: AppTheme.neutral500,
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                    ),
                  ),
                  SizedBox(width: context.rs(12)),
                  Expanded(
                    child: GradientButton(
                      text: 'Save Changes',
                      onPressed: _save,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
