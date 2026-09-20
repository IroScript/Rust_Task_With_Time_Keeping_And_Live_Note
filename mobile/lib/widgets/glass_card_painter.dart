import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';

/// 1:1 Implementation of Rust egui card painting from src/main.rs:3610-3690
class GlassCardPainter extends CustomPainter {
  final bool isSelected;
  final bool isHovered;
  final Color accentColor;

  GlassCardPainter({
    this.isSelected = false,
    this.isHovered = false,
    this.accentColor = CyberTheme.neonCyan,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Rect.fromLTWH(0, 0, size.width, size.height);
    const r = Radius.circular(20.0);
    final rrect = RRect.fromRectAndRadius(rect, r);

    final bool active = isSelected || isHovered;
    final double glowIntensity = active ? 0.75 : 0.15;

    // 1. Two-Layer Glow Halos (src/main.rs:3624: expand 18.0 and 8.0)
    // Rounding formula: Rounding::same(22.0 + expand * 0.5)
    final outerRRect = RRect.fromRectAndRadius(
      rect.inflate(18.0),
      const Radius.circular(31.0),
    );
    final outerPaint = Paint()
      ..color = accentColor.withValues(alpha: (0.12 * glowIntensity).clamp(0.0, 1.0))
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 12);
    canvas.drawRRect(outerRRect, outerPaint);

    final innerGlowRRect = RRect.fromRectAndRadius(
      rect.inflate(8.0),
      const Radius.circular(26.0),
    );
    final innerGlowPaint = Paint()
      ..color = accentColor.withValues(alpha: (0.20 * glowIntensity).clamp(0.0, 1.0))
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6);
    canvas.drawRRect(innerGlowRRect, innerGlowPaint);

    // 2. Transparent Glass Card Fill (src/main.rs:3634-3640)
    // glass_opacity = if is_hovered { 25 } else { 12 };
    // fill_color = Color32::from_rgba_unmultiplied(8, 18, 35, glass_opacity);
    final fillPaint = Paint()
      ..color = Color.fromARGB(active ? 25 : 12, 8, 18, 35)
      ..style = PaintingStyle.fill;
    canvas.drawRRect(rrect, fillPaint);

    // 3. Card Border (src/main.rs:3654-3661)
    // border_intensity = if is_hovered { 0.85 } else { 0.25 };
    // border_width = if is_hovered { 2.0 } else { 1.0 };
    final borderPaint = Paint()
      ..color = accentColor.withValues(alpha: active ? 0.85 : 0.25)
      ..strokeWidth = active ? 2.0 : 1.0
      ..style = PaintingStyle.stroke;
    canvas.drawRRect(rrect, borderPaint);

    // 4. Holographic Rim (top edge highlight, src/main.rs:3642-3651)
    // rim_alpha = if is_hovered { 85 } else { 35 };
    // Color32::from_rgba_unmultiplied(180, 240, 255, rim_alpha);
    // Stroke::new(1.5, rim_color);
    if (size.width > 60) {
      final rimPaint = Paint()
        ..color = Color.fromARGB(active ? 85 : 35, 180, 240, 255)
        ..strokeWidth = 1.5
        ..strokeCap = StrokeCap.round;
      canvas.drawLine(
        const Offset(24.0, 2.0),
        Offset(size.width - 24.0, 2.0),
        rimPaint,
      );
    }

    // 5. 4 Cyberpunk Corner L-Markers (src/main.rs:3664-3687)
    // marker_length = 16.0; marker_stroke = Stroke::new(2.5, NEON_CYAN.gamma_multiply(0.95));
    // Offset is 10.0px from corner vertex
    if (active) {
      final markerPaint = Paint()
        ..color = accentColor.withValues(alpha: 0.95)
        ..strokeWidth = 2.5
        ..strokeCap = StrokeCap.square;

      const double mLen = 16.0;
      const double mOffset = 10.0;

      // Top-Left L
      canvas.drawLine(
        const Offset(mOffset, 0),
        const Offset(mOffset + mLen, 0),
        markerPaint,
      );
      canvas.drawLine(
        const Offset(0, mOffset),
        const Offset(0, mOffset + mLen),
        markerPaint,
      );

      // Top-Right L
      canvas.drawLine(
        Offset(size.width - mOffset, 0),
        Offset(size.width - mOffset - mLen, 0),
        markerPaint,
      );
      canvas.drawLine(
        Offset(size.width, mOffset),
        Offset(size.width, mOffset + mLen),
        markerPaint,
      );

      // Bottom-Left L
      canvas.drawLine(
        Offset(mOffset, size.height),
        Offset(mOffset + mLen, size.height),
        markerPaint,
      );
      canvas.drawLine(
        Offset(0, size.height - mOffset),
        Offset(0, size.height - mOffset - mLen),
        markerPaint,
      );

      // Bottom-Right L
      canvas.drawLine(
        Offset(size.width - mOffset, size.height),
        Offset(size.width - mOffset - mLen, size.height),
        markerPaint,
      );
      canvas.drawLine(
        Offset(size.width, size.height - mOffset),
        Offset(size.width, size.height - mOffset - mLen),
        markerPaint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant GlassCardPainter oldDelegate) {
    return oldDelegate.isSelected != isSelected ||
        oldDelegate.isHovered != isHovered ||
        oldDelegate.accentColor != accentColor;
  }
}
