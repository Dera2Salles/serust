import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:flutter/material.dart';

// ─── AroSaina Flat Design Component Library ───────────────────────────────────

/// Flat primary button — solid color, sharp corners, no shadow
class GradientButton extends StatefulWidget {
  final String text;
  final VoidCallback? onPressed;
  final LinearGradient? gradient;
  final bool isLoading;
  final IconData? icon;
  final double? width;
  final double height;
  final Color? color;

  const GradientButton({
    super.key,
    required this.text,
    this.onPressed,
    this.gradient,
    this.isLoading = false,
    this.icon,
    this.width,
    this.height = 54,
    this.color,
  });

  @override
  State<GradientButton> createState() => _GradientButtonState();
}

class _GradientButtonState extends State<GradientButton>
    with SingleTickerProviderStateMixin {
  late AnimationController _ctrl;
  late Animation<double> _scale;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(
      duration: const Duration(milliseconds: 90),
      vsync: this,
    );
    _scale = Tween<double>(
      begin: 1.0,
      end: 0.96,
    ).animate(CurvedAnimation(parent: _ctrl, curve: Curves.easeOut));
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bg = widget.color ?? Theme.of(context).primaryColor;

    return GestureDetector(
      onTapDown: (_) => _ctrl.forward(),
      onTapUp: (_) {
        _ctrl.reverse();
        widget.onPressed?.call();
      },
      onTapCancel: () => _ctrl.reverse(),
      child: ScaleTransition(
        scale: _scale,
        child: Container(
          width: widget.width ?? double.infinity,
          height: context.rs(widget.height),
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(context.rs(14)),
          ),
          child: Center(
            child: widget.isLoading
                ? const SizedBox(
                    width: 22,
                    height: 22,
                    child: CircularProgressIndicator(
                      color: Colors.white,
                      strokeWidth: 2.5,
                    ),
                  )
                : Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (widget.icon != null) ...[
                        Icon(
                          widget.icon,
                          color: Colors.white,
                          size: context.rs(18),
                        ),
                        SizedBox(width: context.rs(8)),
                      ],
                      Text(
                        widget.text,
                        style: TextStyle(
                          color: Colors.white,
                          fontSize: context.rf(15),
                          fontWeight: FontWeight.w700,
                          letterSpacing: 0.2,
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

/// Flat card — pure white, subtle border, no shadow
class SoftCard extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final VoidCallback? onTap;
  final Color? backgroundColor;
  final double? borderRadius;

  const SoftCard({
    super.key,
    required this.child,
    this.padding,
    this.onTap,
    this.backgroundColor,
    this.borderRadius,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final bg =
        backgroundColor ?? (isDark ? AppTheme.surfaceDark : Colors.white);

    final radius = borderRadius ?? 16.0;

    Widget card = Container(
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(radius),
        border: Border.all(
          color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
          width: 1.0,
        ),
      ),
      padding: padding ?? const EdgeInsets.all(16),
      child: child,
    );

    if (onTap != null) {
      return GestureDetector(onTap: onTap, child: card);
    }
    return card;
  }
}

/// Illustration card — colored top strip, content below
class IllustrationCard extends StatelessWidget {
  final Widget child;
  final double aspectRatio;
  final LinearGradient? gradient;
  final Color? solidColor;

  const IllustrationCard({
    super.key,
    required this.child,
    this.aspectRatio = 16 / 9,
    this.gradient,
    this.solidColor,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return AspectRatio(
      aspectRatio: aspectRatio,
      child: Container(
        decoration: BoxDecoration(
          gradient: gradient ?? AppGradients.overlayPurple,
          color: solidColor,
          borderRadius: BorderRadius.circular(18),
          border: Border.all(
            color: isDark ? AppTheme.neutral700 : AppTheme.neutral200,
          ),
        ),
        clipBehavior: Clip.antiAlias,
        child: child,
      ),
    );
  }
}

/// Modern text field — flat, no floating label, crisp border
class ModernTextField extends StatelessWidget {
  final String label;
  final String hintText;
  final bool obscureText;
  final TextEditingController? controller;
  final TextInputType keyboardType;
  final String? Function(String?)? validator;
  final void Function(String)? onChanged;
  final IconData? prefixIcon;
  final Widget? suffix;

  const ModernTextField({
    super.key,
    required this.label,
    required this.hintText,
    this.obscureText = false,
    this.controller,
    this.keyboardType = TextInputType.text,
    this.validator,
    this.onChanged,
    this.prefixIcon,
    this.suffix,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: context.rf(12),
            fontWeight: FontWeight.w700,
            letterSpacing: 0.6,
            color: isDark ? AppTheme.neutral400 : AppTheme.neutral500,
          ),
        ),
        SizedBox(height: context.rs(8)),
        TextFormField(
          controller: controller,
          obscureText: obscureText,
          keyboardType: keyboardType,
          onChanged: onChanged,
          validator: validator,
          style: TextStyle(
            fontSize: context.rf(15),
            fontWeight: FontWeight.w500,
            color: isDark ? Colors.white : AppTheme.neutral900,
          ),
          decoration: InputDecoration(
            hintText: hintText,
            hintStyle: TextStyle(
              color: AppTheme.neutral400,
              fontSize: context.rf(15),
            ),
            prefixIcon: prefixIcon != null
                ? Icon(prefixIcon, color: AppTheme.neutral400, size: 20)
                : null,
            suffix: suffix,
            filled: true,
            fillColor: isDark ? AppTheme.neutral800 : AppTheme.neutral100,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(12),
              borderSide: BorderSide.none,
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(12),
              borderSide: BorderSide(
                color: Theme.of(context).primaryColor,
                width: 2,
              ),
            ),
            contentPadding: EdgeInsets.symmetric(
              horizontal: context.rs(16),
              vertical: context.rs(14),
            ),
          ),
        ),
      ],
    );
  }
}

/// Pill chip tag
class AroChip extends StatelessWidget {
  final String label;
  final Color color;
  final bool small;

  const AroChip({
    super.key,
    required this.label,
    required this.color,
    this.small = false,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: context.rs(small ? 8 : 12),
        vertical: context.rs(small ? 3 : 5),
      ),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(100),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: context.rf(small ? 10 : 12),
          fontWeight: FontWeight.w700,
          color: color,
          letterSpacing: 0.3,
        ),
      ),
    );
  }
}

/// Flat icon button with background
class AroIconButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback onTap;
  final Color? bgColor;
  final Color? iconColor;
  final double size;

  const AroIconButton({
    super.key,
    required this.icon,
    required this.onTap,
    this.bgColor,
    this.iconColor,
    this.size = 40,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return GestureDetector(
      onTap: onTap,
      child: Container(
        width: context.rs(size),
        height: context.rs(size),
        decoration: BoxDecoration(
          color:
              bgColor ?? (isDark ? AppTheme.neutral800 : AppTheme.neutral100),
          borderRadius: BorderRadius.circular(context.rs(size * 0.3)),
        ),
        child: Icon(
          icon,
          size: context.rs(size * 0.45),
          color: iconColor ?? (isDark ? Colors.white : AppTheme.neutral700),
        ),
      ),
    );
  }
}
