import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';
import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/screens/auth/login_screen.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:arosaina/presentation/widgets/premium_components.dart';
import 'package:flutter/material.dart';

/// Premium Onboarding Screen with 4 Illustration-Driven Pages
class OnboardingScreen extends StatefulWidget {
  const OnboardingScreen({super.key});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final PageController _pageController = PageController();
  int _currentPage = 0;

  final List<_OnboardingPage> _pages = [
    _OnboardingPage(
      icon: Icons.swap_horiz_rounded,
      gradient: AppGradients.primaryPurple,
      title: "Welcome to AroSaina",
      subtitle: "Fast, secure file sharing",
      description:
          "Share files instantly with end-to-end encryption. No servers, no tracking.",
    ),
    _OnboardingPage(
      icon: Icons.upload_rounded,
      gradient: AppGradients.primaryBlue,
      title: "Share Anything",
      subtitle: "Send files instantly",
      description:
          "Transfer photos, videos, documents, and more at lightning speed.",
    ),
    _OnboardingPage(
      icon: Icons.qr_code_scanner_rounded,
      gradient: AppGradients.primaryGreen,
      title: "Simple & Secure",
      subtitle: "Scan and receive",
      description:
          "Just scan a QR code to start receiving files. It's that easy.",
    ),
    _OnboardingPage(
      icon: Icons.lock_outline_rounded,
      gradient: AppGradients.primaryTeal,
      title: "End-to-End Encrypted",
      subtitle: "Your privacy matters",
      description:
          "All transfers are protected with AES-256 encryption. Your data stays yours.",
    ),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Container(
        decoration: BoxDecoration(gradient: AppGradients.backgroundLight),
        child: SafeArea(
          child: Column(
            children: [
              Expanded(
                child: PageView.builder(
                  controller: _pageController,
                  onPageChanged: (value) {
                    setState(() {
                      _currentPage = value;
                    });
                  },
                  itemCount: _pages.length,
                  itemBuilder: (context, index) {
                    return _buildPage(_pages[index]);
                  },
                ),
              ),

              // Page Indicators
              Padding(
                padding: EdgeInsets.symmetric(vertical: context.rs(24)),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: List.generate(
                    _pages.length,
                    (index) => AnimatedContainer(
                      duration: const Duration(milliseconds: 300),
                      margin: EdgeInsets.only(right: context.rs(8)),
                      height: context.rs(8),
                      width: _currentPage == index
                          ? context.rs(32)
                          : context.rs(8),
                      decoration: BoxDecoration(
                        gradient: _currentPage == index
                            ? _pages[_currentPage].gradient
                            : null,
                        color: _currentPage != index
                            ? AppTheme.neutral300
                            : null,
                        borderRadius: BorderRadius.circular(context.rs(4)),
                      ),
                    ),
                  ),
                ),
              ),

              // Action Buttons
              Padding(
                padding: EdgeInsets.fromLTRB(
                  context.rs(24),
                  0,
                  context.rs(24),
                  context.rs(24),
                ),
                child: Column(
                  children: [
                    GradientButton(
                      text: _currentPage == _pages.length - 1
                          ? "Get Started"
                          : "Next",
                      gradient: _pages[_currentPage].gradient,
                      onPressed: () async {
                        if (_currentPage < _pages.length - 1) {
                          _pageController.nextPage(
                            duration: const Duration(milliseconds: 300),
                            curve: Curves.easeInOut,
                          );
                        } else {
                          await sl<ISettingsService>().setHasSeenOnboarding(
                            true,
                          );
                          if (!context.mounted) return;
                          Navigator.pushReplacement(
                            context,
                            MaterialPageRoute(
                              builder: (_) => const LoginScreen(),
                            ),
                          );
                        }
                      },
                    ),
                    if (_currentPage < _pages.length - 1) ...[
                      SizedBox(height: context.rs(12)),
                      TextButton(
                        onPressed: () async {
                          await sl<ISettingsService>().setHasSeenOnboarding(
                            true,
                          );
                          if (!context.mounted) return;
                          Navigator.pushReplacement(
                            context,
                            MaterialPageRoute(
                              builder: (_) => const LoginScreen(),
                            ),
                          );
                        },
                        child: Text(
                          "Skip",
                          style: TextStyle(
                            color: AppTheme.neutral500,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildPage(_OnboardingPage page) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: context.rs(24)),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Illustration Card
          IllustrationCard(
            aspectRatio: 1,
            gradient: LinearGradient(
              colors: [
                page.gradient.colors.first.withValues(alpha: 0.15),
                page.gradient.colors.last.withValues(alpha: 0.05),
              ],
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
            child: Icon(
              page.icon,
              size: context.rs(100),
              color: page.gradient.colors.first,
            ),
          ),

          SizedBox(height: context.rs(48)),

          // Title
          Text(
            page.title,
            style: TextStyle(
              fontSize: context.rf(28),
              fontWeight: FontWeight.bold,
            ),
            textAlign: TextAlign.center,
          ),

          SizedBox(height: context.rs(12)),

          // Subtitle
          ShaderMask(
            shaderCallback: (bounds) => page.gradient.createShader(bounds),
            child: Text(
              page.subtitle,
              style: TextStyle(
                fontSize: context.rf(18),
                fontWeight: FontWeight.w600,
                color: Colors.white,
              ),
              textAlign: TextAlign.center,
            ),
          ),

          SizedBox(height: context.rs(24)),

          // Description
          Text(
            page.description,
            style: TextStyle(
              fontSize: context.rf(16),
              color: Theme.of(
                context,
              ).colorScheme.onSurface.withValues(alpha: 0.6),
              height: 1.5,
            ),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}

class _OnboardingPage {
  final IconData icon;
  final LinearGradient gradient;
  final String title;
  final String subtitle;
  final String description;

  _OnboardingPage({
    required this.icon,
    required this.gradient,
    required this.title,
    required this.subtitle,
    required this.description,
  });
}
