import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';

/// 100% faithful port of Rust card_header_widgets.rs
class CardHeaderWidget extends StatelessWidget {
  final TaskCard card;
  final VoidCallback onAddSubCard;
  final VoidCallback onToggleStopwatch;
  final VoidCallback onSelectDeadline;
  final VoidCallback onSelectSubTaskTime;
  final VoidCallback onCycleMode;
  final VoidCallback? onOpenNote;

  const CardHeaderWidget({
    super.key,
    required this.card,
    required this.onAddSubCard,
    required this.onToggleStopwatch,
    required this.onSelectDeadline,
    required this.onSelectSubTaskTime,
    required this.onCycleMode,
    this.onOpenNote,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: Row(
            mainAxisSize: MainAxisSize.min,
          children: [
            // [+] Plus Button: 22x22 px, thin green border, bold crimson red plus
            InkWell(
              onTap: onAddSubCard,
              borderRadius: BorderRadius.circular(5),
              child: Container(
                width: 22,
                height: 22,
                decoration: BoxDecoration(
                  color: Colors.transparent,
                  border: Border.all(color: CyberTheme.greenBorder, width: 1.0),
                  borderRadius: BorderRadius.circular(5),
                ),
                alignment: Alignment.center,
                child: const Text(
                  '+',
                  style: TextStyle(
                    color: CyberTheme.redCross,
                    fontSize: 15,
                    fontWeight: FontWeight.w900,
                    height: 1.0,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 6),

            // Badge 1: Deadline (Red Accent, ~62x18 px)
            _buildBadge(
              text: card.startTime,
              textColor: CyberTheme.clockRed,
              borderColor: CyberTheme.greenBorder,
              onTap: onSelectDeadline,
            ),
            const SizedBox(width: 4),

            // Badge 2: Sub-task Time (Amber Accent, ~62x18 px)
            _buildBadge(
              text: card.endTime,
              textColor: CyberTheme.clockAmber,
              borderColor: CyberTheme.greenBorder,
              onTap: onSelectSubTaskTime,
            ),
            const SizedBox(width: 4),

            // Badge 3: Stopwatch (Steel Blue Accent, ~62x18 px)
            _buildBadge(
              text: card.formattedStopwatch,
              textColor: card.isStopwatchRunning
                  ? CyberTheme.neonCyan
                  : CyberTheme.clockBlue,
              borderColor: card.isStopwatchRunning
                  ? CyberTheme.neonCyan
                  : CyberTheme.greenBorder,
              isPulsing: card.isStopwatchRunning,
              onTap: onToggleStopwatch,
            ),

            if (onOpenNote != null) ...[
              const SizedBox(width: 4),
              _buildBadge(
                text: '📝',
                textColor: (card.liveNote.isNotEmpty || card.totalLines > 0)
                    ? CyberTheme.neonCyan
                    : CyberTheme.textMuted,
                borderColor: CyberTheme.greenBorder,
                onTap: onOpenNote!,
              ),
            ],
          ],
        ),
      ),
      const SizedBox(height: 3),

        // Sub-task time guide hint
        GestureDetector(
          onTap: onCycleMode,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
            decoration: BoxDecoration(
              color: Colors.black45,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Text(
              'Mode: ${card.clockMode.label} • Tap badge to set time/timer',
              style: const TextStyle(
                color: Color(0xFF6E7681),
                fontSize: 10,
                fontFamily: 'monospace',
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildBadge({
    required String text,
    required Color textColor,
    required Color borderColor,
    required VoidCallback onTap,
    bool isPulsing = false,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(5),
      child: Container(
        height: 20,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(
          color: isPulsing ? textColor.withValues(alpha: 0.15) : Colors.transparent,
          border: Border.all(color: borderColor, width: 1.0),
          borderRadius: BorderRadius.circular(5),
        ),
        alignment: Alignment.center,
        child: Text(
          text,
          style: TextStyle(
            color: textColor,
            fontSize: 11,
            fontWeight: FontWeight.w600,
            fontFamily: 'monospace',
            letterSpacing: 0.5,
          ),
        ),
      ),
    );
  }
}
