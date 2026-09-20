import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';

/// 1:1 Implementation of Rust render_card_size_popup from src/main.rs:6295-6370
class CardSizePopupWidget extends StatelessWidget {
  final double currentScale;
  final Function(double) onSelectScale;
  final VoidCallback onClose;

  const CardSizePopupWidget({
    super.key,
    required this.currentScale,
    required this.onSelectScale,
    required this.onClose,
  });

  static const List<MapEntry<double, String>> sizes = [
    MapEntry(0.0, "0%"),
    MapEntry(0.1, "10%"),
    MapEntry(0.2, "20%"),
    MapEntry(0.3, "30%"),
    MapEntry(0.4, "40%"),
    MapEntry(0.5, "50%"),
    MapEntry(0.6, "60%"),
    MapEntry(0.7, "70%"),
    MapEntry(0.8, "80%"),
    MapEntry(0.9, "90%"),
    MapEntry(1.0, "100%"),
    MapEntry(1.2, "120%"),
    MapEntry(1.4, "140%"),
    MapEntry(1.6, "160%"),
    MapEntry(1.8, "180%"),
    MapEntry(2.0, "200%"),
    MapEntry(2.2, "220%"),
    MapEntry(2.4, "240%"),
    MapEntry(2.6, "260%"),
    MapEntry(2.8, "280%"),
    MapEntry(3.0, "300%"),
  ];

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: Container(
        width: 170,
        constraints: const BoxConstraints(maxHeight: 380),
        decoration: BoxDecoration(
          color: const Color(0xF00A0F1E),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: CyberTheme.neonCyan.withValues(alpha: 0.7),
            width: 1.5,
          ),
          boxShadow: [
            BoxShadow(
              color: CyberTheme.neonCyan.withValues(alpha: 0.25),
              blurRadius: 16,
              spreadRadius: 2,
            ),
          ],
        ),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Header with Close
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Row(
                  children: [
                    Icon(Icons.height, color: CyberTheme.neonCyan, size: 14),
                    SizedBox(width: 4),
                    Text(
                      'CARD SIZE',
                      style: TextStyle(
                        color: CyberTheme.neonCyan,
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        fontFamily: 'monospace',
                        letterSpacing: 0.8,
                      ),
                    ),
                  ],
                ),
                InkWell(
                  onTap: onClose,
                  child: const Padding(
                    padding: EdgeInsets.all(2),
                    child: Text(
                      '×',
                      style: TextStyle(
                        color: CyberTheme.textSecondary,
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const Divider(color: CyberTheme.borderSubtle, height: 12),

            // Scrollable size items
            Flexible(
              child: ListView.builder(
                shrinkWrap: true,
                itemCount: sizes.length,
                itemBuilder: (context, index) {
                  final entry = sizes[index];
                  final bool isCurrent = (currentScale - entry.key).abs() < 0.05;

                  return InkWell(
                    onTap: () {
                      onSelectScale(entry.key);
                      onClose();
                    },
                    borderRadius: BorderRadius.circular(4),
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 6,
                      ),
                      margin: const EdgeInsets.symmetric(vertical: 1),
                      decoration: BoxDecoration(
                        color: isCurrent
                            ? CyberTheme.neonLime.withValues(alpha: 0.15)
                            : Colors.transparent,
                        borderRadius: BorderRadius.circular(4),
                        border: isCurrent
                            ? Border.all(
                                color: CyberTheme.neonLime.withValues(alpha: 0.6),
                                width: 0.8,
                              )
                            : null,
                      ),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Expanded(
                            child: Text(
                              entry.value,
                              style: TextStyle(
                                color: isCurrent ? CyberTheme.neonLime : Colors.white,
                                fontSize: 11,
                                fontWeight: isCurrent
                                    ? FontWeight.bold
                                    : FontWeight.normal,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ),
                          if (isCurrent)
                            const Icon(
                              Icons.check,
                              color: CyberTheme.neonLime,
                              size: 12,
                            ),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
