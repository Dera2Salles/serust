import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

import 'package:arosaina/presentation/screens/home/home_screen.dart';

class SignupScreen extends StatefulWidget {
  const SignupScreen({super.key});

  @override
  State<SignupScreen> createState() => _SignupScreenState();
}

class _SignupScreenState extends State<SignupScreen> {
  final _nameController = TextEditingController();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  bool _isLoading = false;

  Future<void> _signup() async {
    if (_nameController.text.isEmpty ||
        _emailController.text.isEmpty ||
        _passwordController.text.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text("Please fill in all fields")),
      );
      return;
    }

    setState(() => _isLoading = true);

    try {
      final authService = sl<IAuthService>();
      await authService.register(
        _nameController.text,
        _emailController.text,
        _passwordController.text,
      );

      if (mounted) {
        Navigator.pushAndRemoveUntil(
          context,
          MaterialPageRoute(builder: (context) => const HomeScreen()),
          (route) => false,
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text("Signup failed: $e")));
      }
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Container(
        decoration: BoxDecoration(gradient: AppGradients.backgroundLight),
        child: SafeArea(
          child: SingleChildScrollView(
            padding: EdgeInsets.all(context.rs(24)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Premium-styled back button
                GestureDetector(
                  onTap: () => Navigator.pop(context),
                  child: Container(
                    width: context.rs(40),
                    height: context.rs(40),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      shape: BoxShape.circle,
                      border: Border.all(color: AppTheme.neutral200),
                    ),
                    child: Icon(Icons.arrow_back, size: context.rs(20)),
                  ),
                ),

                SizedBox(height: context.rs(32)),

                Text(
                  "Create Account",
                  style: Theme.of(context).textTheme.displaySmall?.copyWith(
                    fontWeight: FontWeight.bold,
                    color: AppTheme.neutral900,
                  ),
                ),
                SizedBox(height: context.rs(8)),
                Text(
                  "Join AroSaina today and share securely.",
                  style: TextStyle(
                    fontSize: context.rf(16),
                    color: AppTheme.neutral500,
                  ),
                ),

                SizedBox(height: context.rs(48)),

                SoftCard(
                  padding: EdgeInsets.all(context.rs(24)),
                  child: Column(
                    children: [
                      ModernTextField(
                        label: "Full Name",
                        hintText: "John Doe",
                        controller: _nameController,
                        prefixIcon: Icons.person_outline,
                      ),
                      SizedBox(height: context.rs(24)),
                      ModernTextField(
                        label: "Email",
                        hintText: "example@email.com",
                        controller: _emailController,
                        keyboardType: TextInputType.emailAddress,
                        prefixIcon: Icons.email_outlined,
                      ),
                      SizedBox(height: context.rs(24)),
                      ModernTextField(
                        label: "Password",
                        hintText: "Create a password",
                        obscureText: true,
                        controller: _passwordController,
                        prefixIcon: Icons.lock_outline,
                      ),
                    ],
                  ),
                ),

                SizedBox(height: context.rs(48)),

                _isLoading
                    ? const Center(child: CircularProgressIndicator())
                    : GradientButton(text: "Sign Up", onPressed: _signup),

                SizedBox(height: context.rs(24)),

                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      "Already have an account? ",
                      style: TextStyle(
                        color: AppTheme.neutral600,
                        fontSize: context.rf(14),
                      ),
                    ),
                    TextButton(
                      onPressed: () => Navigator.pop(context),
                      child: Text(
                        "Login",
                        style: TextStyle(
                          color: AppTheme.blue,
                          fontWeight: FontWeight.bold,
                          fontSize: context.rf(14),
                        ),
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
}
