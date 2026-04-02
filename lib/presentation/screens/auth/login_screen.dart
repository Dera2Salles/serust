import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/screens/auth/signup_screen.dart';
import 'package:arosaina/presentation/screens/home/home_screen.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:arosaina/presentation/widgets/server_config_dialog.dart';
import 'package:flutter/material.dart';

class LoginScreen extends StatefulWidget {
  const LoginScreen({super.key});

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _formKey = GlobalKey<FormState>();
  final _emailCtrl = TextEditingController();
  final _passCtrl = TextEditingController();
  bool _loading = false;
  bool _obscure = true;
  String? _error;

  @override
  void dispose() {
    _emailCtrl.dispose();
    _passCtrl.dispose();
    super.dispose();
  }

  Future<void> _login() async {
    if (!(_formKey.currentState?.validate() ?? false)) return;
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final ok = await sl<IAuthService>().login(
        _emailCtrl.text,
        _passCtrl.text,
      );
      if (!mounted) return;
      if (ok) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (_) => const HomeScreen()),
        );
      } else {
        setState(() {
          _error = 'Invalid email or password';
          _loading = false;
        });
      }
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final primary = Theme.of(context).primaryColor;

    return Scaffold(
      backgroundColor: isDark
          ? AppTheme.backgroundDark
          : AppTheme.backgroundLight,
      body: SafeArea(
        child: SingleChildScrollView(
          padding: EdgeInsets.all(context.rs(24)),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(height: context.rs(40)),

                // Brand mark
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Container(
                      width: context.rs(56),
                      height: context.rs(56),
                      decoration: BoxDecoration(
                        color: primary,
                        borderRadius: BorderRadius.circular(16),
                      ),
                      child: Icon(
                        Icons.swap_horiz_rounded,
                        color: Colors.white,
                        size: context.rs(30),
                      ),
                    ),
                    IconButton(
                      onPressed: () => ServerConfigDialog.show(context),
                      icon: Icon(
                        Icons.dns_rounded,
                        color: isDark ? Colors.white : AppTheme.neutral900,
                      ),
                      style: IconButton.styleFrom(
                        backgroundColor: isDark
                            ? Colors.white.withValues(alpha: 0.05)
                            : AppTheme.neutral100,
                        padding: EdgeInsets.all(context.rs(12)),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(12),
                        ),
                      ),
                    ),
                  ],
                ),

                SizedBox(height: context.rs(28)),

                Text(
                  'Welcome back',
                  style: TextStyle(
                    fontSize: context.rf(28),
                    fontWeight: FontWeight.w800,
                    letterSpacing: -0.8,
                    color: isDark ? Colors.white : AppTheme.neutral900,
                  ),
                ),
                SizedBox(height: context.rs(6)),
                Text(
                  'Sign in to AroSaina',
                  style: TextStyle(
                    fontSize: context.rf(15),
                    color: AppTheme.neutral500,
                    fontWeight: FontWeight.w500,
                  ),
                ),

                SizedBox(height: context.rs(36)),

                ModernTextField(
                  label: 'EMAIL',
                  hintText: 'you@example.com',
                  controller: _emailCtrl,
                  keyboardType: TextInputType.emailAddress,
                  prefixIcon: Icons.mail_outline_rounded,
                  validator: (v) => (v?.isEmpty ?? true) ? 'Required' : null,
                ),

                SizedBox(height: context.rs(16)),

                ModernTextField(
                  label: 'PASSWORD',
                  hintText: '••••••••',
                  controller: _passCtrl,
                  obscureText: _obscure,
                  prefixIcon: Icons.lock_outline_rounded,
                  suffix: GestureDetector(
                    onTap: () => setState(() => _obscure = !_obscure),
                    child: Icon(
                      _obscure
                          ? Icons.visibility_off_outlined
                          : Icons.visibility_outlined,
                      color: AppTheme.neutral400,
                      size: 20,
                    ),
                  ),
                  validator: (v) => (v?.isEmpty ?? true) ? 'Required' : null,
                ),

                if (_error != null) ...[
                  SizedBox(height: context.rs(12)),
                  Container(
                    padding: EdgeInsets.all(context.rs(12)),
                    decoration: BoxDecoration(
                      color: AppTheme.rose.withValues(alpha: 0.08),
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(
                        color: AppTheme.rose.withValues(alpha: 0.2),
                      ),
                    ),
                    child: Row(
                      children: [
                        Icon(
                          Icons.error_outline_rounded,
                          color: AppTheme.rose,
                          size: 16,
                        ),
                        SizedBox(width: context.rs(8)),
                        Expanded(
                          child: Text(
                            _error!,
                            style: TextStyle(
                              color: AppTheme.rose,
                              fontSize: context.rf(12),
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],

                SizedBox(height: context.rs(28)),

                GradientButton(
                  text: 'Sign In',
                  onPressed: _login,
                  isLoading: _loading,
                ),

                SizedBox(height: context.rs(20)),

                Center(
                  child: GestureDetector(
                    onTap: () => Navigator.pushReplacement(
                      context,
                      MaterialPageRoute(builder: (_) => const SignupScreen()),
                    ),
                    child: RichText(
                      text: TextSpan(
                        text: "Don't have an account? ",
                        style: TextStyle(
                          color: AppTheme.neutral500,
                          fontSize: context.rf(14),
                          fontWeight: FontWeight.w500,
                        ),
                        children: [
                          TextSpan(
                            text: 'Sign up',
                            style: TextStyle(
                              color: primary,
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
