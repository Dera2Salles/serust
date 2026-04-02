import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// ─── AroSaina Design System — Notion Edition ─────────────────────────────────
/// Black-and-white, minimal, flat. Inspired by Notion's monochrome aesthetic.
class AppTheme {
  // ── Legacy color aliases — remapped to monochrome (Notion B&W) ────────────
  /// Accent used for "danger/sent" states → dark grey
  static const Color rose    = Color(0xFF555555);
  /// Accent used for "success/received" states → white
  static const Color emerald = Colors.white;
  /// Accent used for "in-progress" states → mid grey
  static const Color amber   = Color(0xFF888888);
  /// Accent for misc highlights → secondary grey
  static const Color violet  = Color(0xFF9B9B9B);
  static const Color cyan    = Color(0xFF9B9B9B);
  static const Color indigo  = Color(0xFF9B9B9B);
  static const Color blue    = indigo;
  static const Color green   = emerald;
  static const Color coral   = rose;
  static const Color teal    = cyan;
  static const Color purple  = violet;
  static const Color accentGreen = emerald;
  static const Color accentRed   = rose;
  /// Single-entry themeColors — Notion uses no color picker
  static const List<Color> themeColors = [notionBlack];

  // ── Notion Palette ─────────────────────────────────────────────────────────
  /// Deep black — Notion dark background
  static const Color notionBlack = Color(0xFF191919);
  /// Raised card surface in dark mode
  static const Color notionSurface = Color(0xFF252525);
  /// Elevated hover/selected state in dark mode
  static const Color notionElevated = Color(0xFF2F2F2F);
  /// Divider lines in dark mode
  static const Color notionDividerDark = Color(0xFF2F2F2F);
  /// Secondary text in dark mode
  static const Color notionTextSecondaryDark = Color(0xFF9B9B9B);

  /// Light surface
  static const Color notionLightBg = Color(0xFFFFFFFF);
  /// Light card
  static const Color notionLightSurface = Color(0xFFF7F7F5);
  /// Light divider
  static const Color notionDividerLight = Color(0xFFE9E9E7);
  /// Secondary text in light mode
  static const Color notionTextSecondaryLight = Color(0xFF737373);

  // ── Legacy aliases kept for backward compat ────────────────────────────────
  static const Color backgroundDark = notionBlack;
  static const Color backgroundLight = notionLightBg;
  static const Color surfaceDark = notionSurface;
  static const Color surfaceLight = notionLightSurface;
  static const Color neutral50 = Color(0xFFFAFAFA);
  static const Color neutral100 = Color(0xFFF7F7F5);
  static const Color neutral200 = notionDividerLight;
  static const Color neutral300 = Color(0xFFD1D5DB);
  static const Color neutral400 = notionTextSecondaryLight;
  static const Color neutral500 = Color(0xFF737373);
  static const Color neutral600 = Color(0xFF4B5563);
  static const Color neutral700 = notionDividerDark;
  static const Color neutral800 = notionSurface;
  static const Color neutral900 = notionBlack;

  // ── Typography ────────────────────────────────────────────────────────────
  static TextTheme _textTheme(bool isDark) {
    final base = isDark ? Colors.white : notionBlack;
    final secondary = isDark ? notionTextSecondaryDark : notionTextSecondaryLight;
    return GoogleFonts.spaceGroteskTextTheme().copyWith(
      displayLarge: GoogleFonts.spaceGrotesk(
        fontSize: 34, fontWeight: FontWeight.w800, color: base, letterSpacing: -1.0,
      ),
      displayMedium: GoogleFonts.spaceGrotesk(
        fontSize: 26, fontWeight: FontWeight.w700, color: base, letterSpacing: -0.5,
      ),
      titleLarge: GoogleFonts.spaceGrotesk(
        fontSize: 20, fontWeight: FontWeight.w700, color: base, letterSpacing: -0.3,
      ),
      bodyLarge: GoogleFonts.spaceGrotesk(
        fontSize: 16, fontWeight: FontWeight.w500, color: base,
      ),
      bodyMedium: GoogleFonts.spaceGrotesk(
        fontSize: 14, fontWeight: FontWeight.w400, color: secondary,
      ),
      labelSmall: GoogleFonts.spaceGrotesk(
        fontSize: 11, fontWeight: FontWeight.w600, letterSpacing: 0.8, color: secondary,
      ),
    );
  }

  static ThemeData lightTheme([Color? _]) => ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    primaryColor: notionBlack,
    scaffoldBackgroundColor: notionLightBg,
    colorScheme: const ColorScheme.light(
      primary: notionBlack,
      secondary: notionTextSecondaryLight,
      surface: notionLightSurface,
      surfaceContainerHighest: notionLightSurface,
      error: Color(0xFFD32F2F),
      outline: notionDividerLight,
      onPrimary: Colors.white,
      onSurface: notionBlack,
    ),
    textTheme: _textTheme(false),
    appBarTheme: const AppBarTheme(
      backgroundColor: notionLightBg,
      foregroundColor: notionBlack,
      elevation: 0,
      titleTextStyle: TextStyle(
        color: notionBlack, fontSize: 16, fontWeight: FontWeight.w700,
      ),
    ),
    cardTheme: const CardThemeData(
      color: notionLightSurface, elevation: 0,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(10)),
        side: BorderSide(color: notionDividerLight),
      ),
    ),
    dividerColor: notionDividerLight,
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: notionLightSurface,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: notionDividerLight),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: notionDividerLight),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: notionBlack, width: 1.5),
      ),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: notionBlack, foregroundColor: Colors.white,
        elevation: 0, shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(foregroundColor: notionBlack),
    ),
    snackBarTheme: SnackBarThemeData(
      backgroundColor: notionBlack,
      contentTextStyle: const TextStyle(color: Colors.white, fontSize: 13),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      behavior: SnackBarBehavior.floating,
    ),
    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? notionBlack : neutral300,
      ),
      trackColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? neutral500 : neutral200,
      ),
    ),
  );

  static ThemeData darkTheme([Color? _]) => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    primaryColor: Colors.white,
    scaffoldBackgroundColor: notionBlack,
    colorScheme: const ColorScheme.dark(
      primary: Colors.white,
      secondary: notionTextSecondaryDark,
      surface: notionSurface,
      surfaceContainerHighest: notionElevated,
      error: Color(0xFFEF5350),
      outline: notionDividerDark,
      onPrimary: notionBlack,
      onSurface: Colors.white,
    ),
    textTheme: _textTheme(true),
    appBarTheme: const AppBarTheme(
      backgroundColor: notionBlack,
      foregroundColor: Colors.white,
      elevation: 0,
      titleTextStyle: TextStyle(
        color: Colors.white, fontSize: 16, fontWeight: FontWeight.w700,
      ),
    ),
    cardTheme: CardThemeData(
      color: notionSurface, elevation: 0,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: const BorderRadius.all(Radius.circular(10)),
        side: BorderSide(color: notionDividerDark),
      ),
    ),
    dialogTheme: const DialogThemeData(backgroundColor: notionSurface),
    bottomSheetTheme: const BottomSheetThemeData(backgroundColor: notionSurface),
    dividerColor: notionDividerDark,
    iconTheme: const IconThemeData(color: Colors.white),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: notionElevated,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: notionDividerDark),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: notionDividerDark),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Colors.white, width: 1.5),
      ),
      labelStyle: const TextStyle(color: notionTextSecondaryDark),
      hintStyle: const TextStyle(color: notionTextSecondaryDark),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: Colors.white, foregroundColor: notionBlack,
        elevation: 0, shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(foregroundColor: Colors.white),
    ),
    snackBarTheme: SnackBarThemeData(
      backgroundColor: notionSurface,
      contentTextStyle: const TextStyle(color: Colors.white, fontSize: 13),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      behavior: SnackBarBehavior.floating,
    ),
    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? Colors.white : neutral500,
      ),
      trackColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? neutral500 : notionDividerDark,
      ),
    ),
  );
}

// ── Gradient System (kept minimal — monochrome only) ─────────────────────────
class AppGradients {
  static const LinearGradient backgroundDark = LinearGradient(
    colors: [AppTheme.notionBlack, AppTheme.notionSurface],
    begin: Alignment.topCenter, end: Alignment.bottomCenter,
  );
  static const LinearGradient backgroundLight = LinearGradient(
    colors: [AppTheme.notionLightBg, AppTheme.notionLightSurface],
    begin: Alignment.topCenter, end: Alignment.bottomCenter,
  );
  // Kept for API compat — all go to monochrome
  static const LinearGradient primaryPurple = backgroundDark;
  static const LinearGradient primaryBlue = backgroundDark;
  static const LinearGradient primaryGreen = backgroundDark;
  static const LinearGradient primaryCoral = backgroundDark;
  static const LinearGradient primaryRose = backgroundDark;
  static const LinearGradient primaryTeal = backgroundDark;
  static const LinearGradient overlayPurple = backgroundDark;
  static const LinearGradient overlayBlue = backgroundDark;
  static const LinearGradient overlayGreen = backgroundDark;
  static const LinearGradient shimmer = LinearGradient(
    colors: [Color(0x00FFFFFF), Color(0x40FFFFFF), Color(0x00FFFFFF)],
    stops: [0.0, 0.5, 1.0],
    begin: Alignment(-1.0, 0.0), end: Alignment(1.0, 0.0),
  );
  static LinearGradient getGradientForColor(Color _) => backgroundDark;
  static LinearGradient getOverlayForColor(Color _) => overlayPurple;
}
