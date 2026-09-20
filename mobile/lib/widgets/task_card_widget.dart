import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';
import 'card_header_widget.dart';
import 'glass_card_painter.dart';
import 'top_action_buttons_bar.dart';

/// 1:1 Parity implementation of Rust render_quote_card (src/main.rs:3054-3840)
class TaskCardWidget extends StatefulWidget {
  final TaskCard card;
  final int index;
  final int totalCards;
  final bool isSelected;
  final double cardScale;
  final double zoomLevel;
  final Color universalFontColor;
  final double mainLineGap;
  final double subLineGap;
  final double betweenGap;

  final VoidCallback onTap;
  final VoidCallback onAddSubCard;
  final VoidCallback onToggleStopwatch;
  final VoidCallback onSelectDeadline;
  final VoidCallback onSelectSubTaskTime;
  final VoidCallback onCycleMode;
  final VoidCallback onMoveUp;
  final VoidCallback onMoveDown;
  final VoidCallback onSetPosition;
  final VoidCallback onScheduleTime;
  final VoidCallback onRotationInterval;
  final VoidCallback onToggleHide;
  final VoidCallback onDelete;
  final VoidCallback onOpenNote;
  final Function(String title) onUpdateTitle;
  final Function(String subText) onUpdateSubText;

  const TaskCardWidget({
    super.key,
    required this.card,
    required this.index,
    required this.totalCards,
    this.isSelected = false,
    this.cardScale = 1.0,
    this.zoomLevel = 1.0,
    this.universalFontColor = CyberTheme.textPrimary,
    this.mainLineGap = 4.0,
    this.subLineGap = 2.0,
    this.betweenGap = 8.0,
    required this.onTap,
    required this.onAddSubCard,
    required this.onToggleStopwatch,
    required this.onSelectDeadline,
    required this.onSelectSubTaskTime,
    required this.onCycleMode,
    required this.onMoveUp,
    required this.onMoveDown,
    required this.onSetPosition,
    required this.onScheduleTime,
    required this.onRotationInterval,
    required this.onToggleHide,
    required this.onDelete,
    required this.onOpenNote,
    required this.onUpdateTitle,
    required this.onUpdateSubText,
  });

  @override
  State<TaskCardWidget> createState() => _TaskCardWidgetState();
}

class _TaskCardWidgetState extends State<TaskCardWidget> {
  bool _isEditingTitle = false;
  bool _isEditingSub = false;
  late TextEditingController _titleController;
  late TextEditingController _subController;

  @override
  void initState() {
    super.initState();
    _titleController = TextEditingController(text: widget.card.mainText);
    _subController = TextEditingController(text: widget.card.subText);
  }

  @override
  void didUpdateWidget(covariant TaskCardWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.card.mainText != widget.card.mainText && !_isEditingTitle) {
      _titleController.text = widget.card.mainText;
    }
    if (oldWidget.card.subText != widget.card.subText && !_isEditingSub) {
      _subController.text = widget.card.subText;
    }
  }

  @override
  void dispose() {
    _titleController.dispose();
    _subController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Completely collapsed state (0% scale in Rust)
    if (widget.cardScale <= 0.001) {
      return const SizedBox(height: 2);
    }

    // Nesting scale and indent (from Rust card_scale_at_depth)
    final double indent = widget.card.depth * 16.0;
    final double depthScale = (1.0 - widget.card.depth * 0.05).clamp(0.70, 1.0);

    // Apply effective card height scaling
    final double effectiveScale = (widget.cardScale * depthScale).clamp(0.1, 3.0);

    // Font size calculations matching Rust
    final double mainFontSize =
        ((widget.card.mainTextSize ?? 18.0) * widget.zoomLevel).clamp(10.0, 48.0);
    final double subFontSize =
        ((widget.card.subTextSize ?? 13.0) * widget.zoomLevel).clamp(9.0, 32.0);

    final Color mainTextColor = widget.card.mainTextColor != null
        ? Color(widget.card.mainTextColor!)
        : widget.universalFontColor;

    final Color subTextColor = widget.card.subTextColor != null
        ? Color(widget.card.subTextColor!)
        : CyberTheme.textSecondary;

    final double effectiveBetweenGap = (widget.card.betweenGap ?? widget.betweenGap) * effectiveScale;

    return Opacity(
      opacity: widget.card.isHidden ? 0.45 : 1.0,
      child: Transform.scale(
        scale: depthScale,
        alignment: Alignment.topLeft,
        child: Container(
          margin: EdgeInsets.only(
            left: indent + 12.0,
            right: 12.0,
            top: 14.0, // Room for overlapping circular buttons
            bottom: (10.0 * effectiveScale).clamp(4.0, 24.0),
          ),
          child: Stack(
            clipBehavior: Clip.none,
            children: [
              // 1. Holographic Glass Morphism Card Container (GlassCardPainter)
              CustomPaint(
                painter: GlassCardPainter(
                  isSelected: widget.isSelected,
                  isHovered: widget.isSelected,
                  accentColor: widget.isSelected ? CyberTheme.neonCyan : CyberTheme.borderSubtle,
                ),
                child: InkWell(
                  onTap: widget.onTap,
                  borderRadius: BorderRadius.circular(20.0),
                  child: Padding(
                    padding: EdgeInsets.symmetric(
                      horizontal: 16.0,
                      vertical: (16.0 * effectiveScale).clamp(8.0, 32.0),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.center,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // Card Header Row: [+] Button & 3 Clock Badges
                        Align(
                          alignment: Alignment.centerLeft,
                          child: CardHeaderWidget(
                            card: widget.card,
                            onAddSubCard: widget.onAddSubCard,
                            onToggleStopwatch: widget.onToggleStopwatch,
                            onSelectDeadline: widget.onSelectDeadline,
                            onSelectSubTaskTime: widget.onSelectSubTaskTime,
                            onCycleMode: widget.onCycleMode,
                            onOpenNote: widget.onOpenNote,
                          ),
                        ),

                        if (widget.card.scheduledDate != null &&
                            widget.card.scheduledDate!.isNotEmpty) ...[
                          const SizedBox(height: 6),
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Container(
                              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                              decoration: BoxDecoration(
                                color: CyberTheme.badgeSchedule.withValues(alpha: 0.15),
                                borderRadius: BorderRadius.circular(4),
                                border: Border.all(
                                  color: CyberTheme.badgeSchedule.withValues(alpha: 0.5),
                                  width: 0.8,
                                ),
                              ),
                              child: Text(
                                '⏰ Scheduled: ${widget.card.scheduledDate} ${widget.card.scheduledTime ?? ''}',
                                style: const TextStyle(
                                  color: CyberTheme.badgeSchedule,
                                  fontSize: 10,
                                  fontFamily: 'monospace',
                                ),
                              ),
                            ),
                          ),
                        ],

                        SizedBox(height: (12.0 * effectiveScale).clamp(4.0, 24.0)),

                        // ── Main Text (Center-aligned motivational quote) ──
                        if (_isEditingTitle)
                          TextField(
                            controller: _titleController,
                            autofocus: true,
                            textAlign: TextAlign.center,
                            maxLines: null,
                            style: TextStyle(
                              color: mainTextColor,
                              fontSize: mainFontSize,
                              fontWeight: FontWeight.bold,
                              fontFamily: 'monospace',
                            ),
                            decoration: const InputDecoration(
                              isDense: true,
                              border: UnderlineInputBorder(
                                borderSide: BorderSide(color: CyberTheme.neonCyan),
                              ),
                            ),
                            onSubmitted: (val) {
                              setState(() => _isEditingTitle = false);
                              widget.onUpdateTitle(val);
                            },
                          )
                        else
                          GestureDetector(
                            onTap: () {
                              widget.onTap();
                              setState(() => _isEditingTitle = true);
                            },
                            child: Text(
                              widget.card.mainText.isEmpty ? 'Untitled Card' : widget.card.mainText,
                              textAlign: TextAlign.center,
                              style: TextStyle(
                                color: mainTextColor,
                                fontSize: mainFontSize,
                                fontWeight: FontWeight.bold,
                                fontFamily: 'monospace',
                                letterSpacing: 0.5,
                                height: 1.25,
                              ),
                            ),
                          ),

                        // ── Thin Cyan Separator Line (src/main.rs:3419-3425) ──
                        if (widget.card.subText.isNotEmpty && effectiveBetweenGap > 1.0) ...[
                          SizedBox(height: effectiveBetweenGap),
                          LayoutBuilder(
                            builder: (context, constraints) {
                              final double sepWidth = constraints.maxWidth * 0.35;
                              return Container(
                                width: sepWidth,
                                height: 1.2,
                                decoration: BoxDecoration(
                                  gradient: LinearGradient(
                                    colors: [
                                      Colors.transparent,
                                      CyberTheme.neonCyan.withValues(alpha: 0.6),
                                      Colors.transparent,
                                    ],
                                  ),
                                ),
                              );
                            },
                          ),
                          SizedBox(height: effectiveBetweenGap),

                          // ── Sub Text ──
                          if (_isEditingSub)
                            TextField(
                              controller: _subController,
                              autofocus: true,
                              textAlign: TextAlign.center,
                              maxLines: null,
                              style: TextStyle(
                                color: subTextColor,
                                fontSize: subFontSize,
                                fontFamily: 'monospace',
                              ),
                              decoration: const InputDecoration(
                                isDense: true,
                                border: UnderlineInputBorder(
                                  borderSide: BorderSide(color: CyberTheme.neonCyan),
                                ),
                              ),
                              onSubmitted: (val) {
                                setState(() => _isEditingSub = false);
                                widget.onUpdateSubText(val);
                              },
                            )
                          else
                            GestureDetector(
                              onTap: () {
                                widget.onTap();
                                setState(() => _isEditingSub = true);
                              },
                              child: Text(
                                widget.card.subText,
                                textAlign: TextAlign.center,
                                style: TextStyle(
                                  color: subTextColor,
                                  fontSize: subFontSize,
                                  fontFamily: 'monospace',
                                  height: 1.2,
                                ),
                              ),
                            ),
                        ],
                      ],
                    ),
                  ),
                ),
              ),

              // 2. 7 Overlapping Circular Action Buttons (Right-aligned, overlapping top border)
              // (src/main.rs:3692-3835)
              Positioned(
                top: -11.0, // Perfectly overlaps the top border stroke
                right: 18.0,
                child: TopActionButtonsBar(
                  isFirst: widget.index == 0,
                  isLast: widget.index == widget.totalCards - 1,
                  isHidden: widget.card.isHidden,
                  isNoteActive: widget.card.liveNote.isNotEmpty || widget.card.totalLines > 0,
                  onMoveUp: widget.onMoveUp,
                  onMoveDown: widget.onMoveDown,
                  onSetPosition: widget.onSetPosition,
                  onScheduleTime: widget.onScheduleTime,
                  onRotationInterval: widget.onRotationInterval,
                  onToggleHide: widget.onToggleHide,
                  onDelete: widget.onDelete,
                  onOpenNote: widget.onOpenNote,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
