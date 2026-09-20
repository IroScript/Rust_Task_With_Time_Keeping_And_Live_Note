import 'package:flutter/material.dart';

/// Cyberpunk / Biopunk theme matching 100% of Rust egui desktop styling
class CyberTheme {
  // Backgrounds
  static const Color bgCosmic = Color(0xFF080B10);
  static const Color bgPrimary = Color(0xFF0D1117);
  static const Color bgCard = Color(0xFF0C1322);
  static const Color bgCardActive = Color(0xFF142035);
  static const Color bgPanel = Color(0xFF0E131C);
  static const Color bgOverlay = Color(0xCC080B10);

  // Glass Morphism Colors (from src/main.rs:3635)
  static const Color glassFillIdle = Color(0x19081223);    // alpha 25 (approx 10%)
  static const Color glassFillActive = Color(0x2E081223);  // alpha 46 (approx 18%)
  static const Color holoRimColor = Color(0x55B4F0FF);     // rgba(180, 240, 255, 85)

  // Neon Accents (from src/main.rs)
  static const Color neonCyan = Color(0xFF00F0FF);
  static const Color neonLime = Color(0xFF39FF14);
  static const Color greenBorder = Color(0xFF3CB450);
  static const Color neonPurple = Color(0xFF9D4EDD);
  static const Color neonPink = Color(0xFFFF007F);
  static const Color neonYellow = Color(0xFFFFE600);
  static const Color neonSolar = Color(0xFFFF9900);
  static const Color neonRose = Color(0xFFFF3366);

  // Clock Badges Palette (Exact from src/card_header_widgets.rs)
  static const Color redCross = Color(0xFFB21C1C);    // Crimson plus icon
  static const Color clockRed = Color(0xFFA51616);    // Deadline badge text
  static const Color clockAmber = Color(0xFFB95F0F);  // Sub-task badge text
  static const Color clockBlue = Color(0xFF1C76B9);   // Stopwatch badge text

  // 7 Overlapping Circular Badges Palette (Exact from src/main.rs:3720-3830)
  static const Color badgeMoveUp = Color(0xFF1E78C8);
  static const Color badgeMoveDown = Color(0xFF1E78C8);
  static const Color badgePosition = Color(0xFF28A028);
  static const Color badgeSchedule = Color(0xFF9664C8);
  static const Color badgeInterval = Color(0xFF6496C8);
  static const Color badgeHide = Color(0xFF505050);
  static const Color badgeUnhide = Color(0xFFC88C28);
  static const Color badgeDelete = Color(0xFFC8283C);

  // Text Colors
  static const Color textPrimary = Color(0xFFE6EDF3);
  static const Color textSecondary = Color(0xFF8B949E);
  static const Color textMuted = Color(0xFF484F58);
  static const Color textCyan = Color(0xFF58A6FF);

  // Borders & Dividers
  static const Color borderSubtle = Color(0x3330363D);
  static const Color borderActive = Color(0xFF00F0FF);
  static const Color borderGreen = Color(0xFF3CB450);

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: bgCosmic,
      primaryColor: neonCyan,
      canvasColor: bgPanel,
      cardColor: bgCard,
      dividerColor: borderSubtle,
      fontFamily: 'monospace',
      colorScheme: const ColorScheme.dark(
        primary: neonCyan,
        secondary: neonLime,
        surface: bgCard,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: bgPrimary,
        elevation: 0,
        titleTextStyle: TextStyle(
          color: neonCyan,
          fontSize: 14,
          fontWeight: FontWeight.bold,
          fontFamily: 'monospace',
          letterSpacing: 1.5,
        ),
      ),
    );
  }
}
