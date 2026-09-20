import 'package:flutter/material.dart';
import '../../theme/cyber_theme.dart';

class ThemeModal extends StatelessWidget {
  final String currentTheme;
  final Color universalColor;
  final Function(String themeName) onSelectTheme;
  final Function(Color color) onSelectColor;

  const ThemeModal({
    super.key,
    required this.currentTheme,
    required this.universalColor,
    required this.onSelectTheme,
    required this.onSelectColor,
  });

  static const List<Map<String, dynamic>> themePresets = [
    {
      'id': 'cyberpunk',
      'name': 'Cyberpunk Neon',
      'color': CyberTheme.neonCyan,
      'desc': 'Electric cyan glow with deep dark glass',
    },
    {
      'id': 'matrix_lime',
      'name': 'Matrix Terminal',
      'color': CyberTheme.neonLime,
      'desc': 'High-contrast biopunk phosphor lime',
    },
    {
      'id': 'amber_sunset',
      'name': 'Solar Amber',
      'color': CyberTheme.neonSolar,
      'desc': 'Warm cybernetic amber & gold glow',
    },
    {
      'id': 'deep_space',
      'name': 'Deep Space Violet',
      'color': CyberTheme.neonPurple,
      'desc': 'Dark cosmic nebula with purple accents',
    },
  ];

  static const List<Color> colorPalette = [
    CyberTheme.neonCyan,
    CyberTheme.neonLime,
    CyberTheme.neonSolar,
    CyberTheme.neonPink,
    CyberTheme.neonPurple,
    Colors.white,
    Colors.amber,
    Colors.tealAccent,
  ];

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 18, vertical: 30),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xF70D121F),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: CyberTheme.neonCyan, width: 1.5),
          boxShadow: [
            BoxShadow(
              color: CyberTheme.neonCyan.withValues(alpha: 0.3),
              blurRadius: 20,
            ),
          ],
        ),
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Title
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Row(
                  children: [
                    Icon(Icons.palette, color: CyberTheme.neonCyan, size: 18),
                    SizedBox(width: 8),
                    Text(
                      'THEME & APPEARANCE',
                      style: TextStyle(
                        color: CyberTheme.neonCyan,
                        fontSize: 13,
                        fontWeight: FontWeight.bold,
                        fontFamily: 'monospace',
                        letterSpacing: 1.0,
                      ),
                    ),
                  ],
                ),
                IconButton(
                  icon: const Icon(Icons.close, color: Colors.white, size: 18),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
            const Divider(color: CyberTheme.borderSubtle),

            // Theme Presets
            const Text(
              'COLOR PRESETS',
              style: TextStyle(
                color: CyberTheme.textSecondary,
                fontSize: 10,
                fontWeight: FontWeight.bold,
                fontFamily: 'monospace',
              ),
            ),
            const SizedBox(height: 8),

            ...themePresets.map((preset) {
              final isSelected = currentTheme == preset['id'];
              final Color presetCol = preset['color'];

              return InkWell(
                onTap: () {
                  onSelectTheme(preset['id']);
                  onSelectColor(presetCol);
                },
                child: Container(
                  margin: const EdgeInsets.symmetric(vertical: 4),
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: isSelected
                        ? presetCol.withValues(alpha: 0.15)
                        : Colors.black26,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: isSelected ? presetCol : CyberTheme.borderSubtle,
                      width: isSelected ? 1.5 : 1.0,
                    ),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 14,
                        height: 14,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: presetCol,
                          boxShadow: [
                            BoxShadow(
                              color: presetCol.withValues(alpha: 0.6),
                              blurRadius: 6,
                            ),
                          ],
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              preset['name'],
                              style: TextStyle(
                                color: isSelected ? presetCol : Colors.white,
                                fontSize: 12,
                                fontWeight: FontWeight.bold,
                                fontFamily: 'monospace',
                              ),
                            ),
                            Text(
                              preset['desc'],
                              style: const TextStyle(
                                color: CyberTheme.textSecondary,
                                fontSize: 10,
                              ),
                            ),
                          ],
                        ),
                      ),
                      if (isSelected)
                        Icon(Icons.check_circle, color: presetCol, size: 16),
                    ],
                  ),
                ),
              );
            }),

            const SizedBox(height: 14),
            // Universal Font Color
            const Text(
              'UNIVERSAL FONT COLOR',
              style: TextStyle(
                color: CyberTheme.textSecondary,
                fontSize: 10,
                fontWeight: FontWeight.bold,
                fontFamily: 'monospace',
              ),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceAround,
              children: colorPalette.map((col) {
                final isSelected = universalColor.toARGB32() == col.toARGB32();
                return InkWell(
                  onTap: () => onSelectColor(col),
                  child: Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: col,
                      border: Border.all(
                        color: isSelected ? Colors.white : Colors.transparent,
                        width: 2.5,
                      ),
                      boxShadow: isSelected
                          ? [
                              BoxShadow(
                                color: col.withValues(alpha: 0.8),
                                blurRadius: 8,
                              ),
                            ]
                          : null,
                    ),
                  ),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }
}
