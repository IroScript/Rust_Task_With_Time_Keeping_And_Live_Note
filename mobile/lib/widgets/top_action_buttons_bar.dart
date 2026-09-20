import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';

/// 1:1 Parity for Rust Card Top Overlapping Circular Action Buttons
/// (src/main.rs:3720-3835)
class TopActionButtonsBar extends StatelessWidget {
  final bool isFirst;
  final bool isLast;
  final bool isHidden;
  final VoidCallback onMoveUp;
  final VoidCallback onMoveDown;
  final VoidCallback onSetPosition;
  final VoidCallback onScheduleTime;
  final VoidCallback onRotationInterval;
  final VoidCallback onToggleHide;
  final VoidCallback onDelete;
  final VoidCallback onOpenNote;
  final bool isNoteActive;

  const TopActionButtonsBar({
    super.key,
    required this.isFirst,
    required this.isLast,
    required this.isHidden,
    required this.onMoveUp,
    required this.onMoveDown,
    required this.onSetPosition,
    required this.onScheduleTime,
    required this.onRotationInterval,
    required this.onToggleHide,
    required this.onDelete,
    required this.onOpenNote,
    this.isNoteActive = false,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // 1. Move Up (^)
          _buildCircleBtn(
            symbol: '^',
            color: isFirst ? Colors.grey.shade700 : CyberTheme.badgeMoveUp,
            tooltip: 'Move Up',
            fontSize: 13,
            onTap: isFirst ? null : onMoveUp,
          ),
          const SizedBox(width: 4),

          // 2. Move Down (v)
          _buildCircleBtn(
            symbol: 'v',
            color: isLast ? Colors.grey.shade700 : CyberTheme.badgeMoveDown,
            tooltip: 'Move Down',
            fontSize: 11,
            onTap: isLast ? null : onMoveDown,
          ),
          const SizedBox(width: 4),

          // 3. Set Position (#)
          _buildCircleBtn(
            symbol: '#',
            color: CyberTheme.badgePosition,
            tooltip: 'Set Position',
            fontSize: 12,
            onTap: onSetPosition,
          ),
          const SizedBox(width: 4),

          // 4. Schedule Time (⏰)
          _buildCircleBtn(
            symbol: '⏰',
            color: CyberTheme.badgeSchedule,
            tooltip: 'Schedule Time',
            fontSize: 10,
            onTap: onScheduleTime,
          ),
          const SizedBox(width: 4),

          // 5. Rotation Interval (⏱)
          _buildCircleBtn(
            symbol: '⏱',
            color: CyberTheme.badgeInterval,
            tooltip: 'Rotation Interval',
            fontSize: 10,
            onTap: onRotationInterval,
          ),
          const SizedBox(width: 4),

          // 6. Hide/Unhide (H / O) - src/main.rs:3801-3818
          _buildCircleBtn(
            symbol: isHidden ? 'O' : 'H',
            color: isHidden ? CyberTheme.badgeUnhide : CyberTheme.badgeHide,
            tooltip: isHidden ? 'Unhide Card' : 'Hide Card',
            fontSize: 11,
            onTap: onToggleHide,
          ),
          const SizedBox(width: 4),

          // 7. Delete (X) - src/main.rs:3820-3834
          _buildCircleBtn(
            symbol: 'X',
            color: CyberTheme.badgeDelete,
            tooltip: 'Delete Card',
            fontSize: 11,
            onTap: onDelete,
          ),
        ],
      ),
    );
  }

  Widget _buildCircleBtn({
    required String symbol,
    required Color color,
    required String tooltip,
    required double fontSize,
    required VoidCallback? onTap,
    bool isGlowing = false,
  }) {
    final bool disabled = onTap == null;
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          width: 22,
          height: 22,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: disabled ? Colors.black38 : color.withValues(alpha: 0.85),
            border: Border.all(
              color: isGlowing ? CyberTheme.neonCyan : color,
              width: 1.2,
            ),
            boxShadow: isGlowing
                ? [
                    BoxShadow(
                      color: CyberTheme.neonCyan.withValues(alpha: 0.6),
                      blurRadius: 6,
                    ),
                  ]
                : null,
          ),
          alignment: Alignment.center,
          child: Text(
            symbol,
            textAlign: TextAlign.center,
            style: TextStyle(
              color: disabled ? Colors.grey.shade500 : Colors.white,
              fontSize: fontSize,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
              height: 1.0,
            ),
          ),
        ),
      ),
    );
  }
}
