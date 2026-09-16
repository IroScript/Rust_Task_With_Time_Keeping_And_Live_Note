import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';
import 'card_header_widget.dart';

class TaskCardWidget extends StatefulWidget {
  final TaskCard card;
  final bool isSelected;
  final VoidCallback onTap;
  final VoidCallback onAddSubCard;
  final VoidCallback onToggleStopwatch;
  final VoidCallback onSelectDeadline;
  final VoidCallback onSelectSubTaskTime;
  final VoidCallback onCycleMode;
  final VoidCallback onMoveUp;
  final VoidCallback onMoveDown;
  final VoidCallback onDelete;
  final Function(String) onUpdateTitle;
  final Function(String) onUpdateLiveNote;

  const TaskCardWidget({
    super.key,
    required this.card,
    this.isSelected = false,
    required this.onTap,
    required this.onAddSubCard,
    required this.onToggleStopwatch,
    required this.onSelectDeadline,
    required this.onSelectSubTaskTime,
    required this.onCycleMode,
    required this.onMoveUp,
    required this.onMoveDown,
    required this.onDelete,
    required this.onUpdateTitle,
    required this.onUpdateLiveNote,
  });

  @override
  State<TaskCardWidget> createState() => _TaskCardWidgetState();
}

class _TaskCardWidgetState extends State<TaskCardWidget> {
  bool _isEditingTitle = false;
  bool _showLiveNote = false;
  late TextEditingController _titleController;
  late TextEditingController _noteController;

  @override
  void initState() {
    super.initState();
    _titleController = TextEditingController(text: widget.card.mainText);
    _noteController = TextEditingController(text: widget.card.liveNote);
    if (widget.card.liveNote.isNotEmpty) {
      _showLiveNote = true;
    }
  }

  @override
  void didUpdateWidget(covariant TaskCardWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.card.mainText != widget.card.mainText && !_isEditingTitle) {
      _titleController.text = widget.card.mainText;
    }
    if (oldWidget.card.liveNote != widget.card.liveNote) {
      _noteController.text = widget.card.liveNote;
    }
  }

  @override
  void dispose() {
    _titleController.dispose();
    _noteController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Nested depth scale and indent (from Rust card_scale_at_depth)
    final double indent = widget.card.depth * 20.0;
    final double scale = (1.0 - widget.card.depth * 0.05).clamp(0.70, 1.0);

    return Transform.scale(
      scale: scale,
      alignment: Alignment.topLeft,
      child: Container(
        margin: EdgeInsets.only(left: indent, right: 8, top: 4, bottom: 6),
        decoration: BoxDecoration(
          color: widget.isSelected ? CyberTheme.bgCardActive : CyberTheme.bgCard,
          border: Border.all(
            color: widget.isSelected
                ? CyberTheme.neonCyan
                : CyberTheme.borderSubtle,
            width: widget.isSelected ? 1.5 : 1.0,
          ),
          borderRadius: BorderRadius.circular(8),
          boxShadow: widget.isSelected
              ? [
                  BoxShadow(
                    color: CyberTheme.neonCyan.withValues(alpha: 0.2),
                    blurRadius: 8,
                    spreadRadius: 1,
                  ),
                ]
              : null,
        ),
        child: InkWell(
          onTap: widget.onTap,
          borderRadius: BorderRadius.circular(8),
          child: Padding(
            padding: const EdgeInsets.all(10),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Header row: Header Widgets on left + Action Badges on right
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Expanded(
                      child: CardHeaderWidget(
                        card: widget.card,
                        onAddSubCard: widget.onAddSubCard,
                        onToggleStopwatch: widget.onToggleStopwatch,
                        onSelectDeadline: widget.onSelectDeadline,
                        onSelectSubTaskTime: widget.onSelectSubTaskTime,
                        onCycleMode: widget.onCycleMode,
                      ),
                    ),

                    // Card Action Icons: ^ v # > ? H X
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        _buildActionIcon('^', widget.onMoveUp, tooltip: 'Move Up'),
                        _buildActionIcon('v', widget.onMoveDown, tooltip: 'Move Down'),
                        _buildActionIcon(
                          '📝',
                          () => setState(() => _showLiveNote = !_showLiveNote),
                          tooltip: 'Toggle Live Note',
                          active: _showLiveNote,
                        ),
                        _buildActionIcon(
                          'X',
                          widget.onDelete,
                          tooltip: 'Delete Card',
                          color: CyberTheme.redCross,
                        ),
                      ],
                    ),
                  ],
                ),
                const SizedBox(height: 8),

                // Main Title / Quote text
                if (_isEditingTitle)
                  Row(
                    children: [
                      Expanded(
                        child: TextField(
                          controller: _titleController,
                          autofocus: true,
                          style: const TextStyle(
                            color: CyberTheme.textPrimary,
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                            fontFamily: 'monospace',
                          ),
                          decoration: const InputDecoration(
                            isDense: true,
                            border: UnderlineInputBorder(
                              borderSide: BorderSide(color: CyberTheme.neonCyan),
                            ),
                          ),
                          onSubmitted: (val) {
                            widget.onUpdateTitle(val);
                            setState(() => _isEditingTitle = false);
                          },
                        ),
                      ),
                      IconButton(
                        icon: const Icon(Icons.check, color: CyberTheme.neonLime, size: 18),
                        onPressed: () {
                          widget.onUpdateTitle(_titleController.text);
                          setState(() => _isEditingTitle = false);
                        },
                      ),
                    ],
                  )
                else
                  GestureDetector(
                    onDoubleTap: () => setState(() => _isEditingTitle = true),
                    child: Text(
                      widget.card.mainText,
                      style: const TextStyle(
                        color: CyberTheme.textPrimary,
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ),
                const SizedBox(height: 4),

                // Subtitle / Motivation text
                Text(
                  widget.card.subText,
                  style: const TextStyle(
                    color: CyberTheme.textSecondary,
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                ),

                // Expandable Live Note section
                if (_showLiveNote) ...[
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Colors.black38,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: CyberTheme.borderSubtle),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Row(
                          children: [
                            Text(
                              '// LIVE NOTE STREAM',
                              style: TextStyle(
                                color: CyberTheme.neonCyan,
                                fontSize: 10,
                                fontFamily: 'monospace',
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            Spacer(),
                            Text(
                              'Auto-syncs to SQLite',
                              style: TextStyle(
                                color: CyberTheme.textMuted,
                                fontSize: 9,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 4),
                        TextField(
                          controller: _noteController,
                          maxLines: 3,
                          style: const TextStyle(
                            color: CyberTheme.textPrimary,
                            fontSize: 11,
                            fontFamily: 'monospace',
                          ),
                          decoration: const InputDecoration(
                            hintText: 'Enter massive notes / logs here...',
                            hintStyle: TextStyle(color: CyberTheme.textMuted, fontSize: 11),
                            border: InputBorder.none,
                            isDense: true,
                          ),
                          onChanged: widget.onUpdateLiveNote,
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildActionIcon(
    String label,
    VoidCallback onTap, {
    String? tooltip,
    Color? color,
    bool active = false,
  }) {
    return Tooltip(
      message: tooltip ?? '',
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          width: 22,
          height: 22,
          margin: const EdgeInsets.only(left: 3),
          decoration: BoxDecoration(
            color: active ? CyberTheme.neonCyan.withValues(alpha: 0.2) : Colors.black26,
            border: Border.all(
              color: active ? CyberTheme.neonCyan : CyberTheme.borderSubtle,
              width: 0.8,
            ),
            borderRadius: BorderRadius.circular(4),
          ),
          alignment: Alignment.center,
          child: Text(
            label,
            style: TextStyle(
              color: color ?? (active ? CyberTheme.neonCyan : CyberTheme.textSecondary),
              fontSize: 10,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
        ),
      ),
    );
  }
}
