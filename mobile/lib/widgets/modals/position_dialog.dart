import 'package:flutter/material.dart';
import '../../theme/cyber_theme.dart';

class PositionDialog extends StatefulWidget {
  final int currentIndex;
  final int totalCount;
  final Function(int newIndex) onSetPosition;

  const PositionDialog({
    super.key,
    required this.currentIndex,
    required this.totalCount,
    required this.onSetPosition,
  });

  @override
  State<PositionDialog> createState() => _PositionDialogState();
}

class _PositionDialogState extends State<PositionDialog> {
  late TextEditingController _posController;

  @override
  void initState() {
    super.initState();
    _posController = TextEditingController(text: (widget.currentIndex + 1).toString());
  }

  @override
  void dispose() {
    _posController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: const Color(0xF20A0F1D),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: CyberTheme.badgePosition, width: 1.5),
      ),
      title: const Row(
        children: [
          Icon(Icons.format_list_numbered, color: CyberTheme.badgePosition, size: 18),
          SizedBox(width: 8),
          Text(
            'SET CARD POSITION (#)',
            style: TextStyle(
              color: CyberTheme.badgePosition,
              fontSize: 13,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Move this card to a specific position (1 to ${widget.totalCount}):',
            style: const TextStyle(color: CyberTheme.textSecondary, fontSize: 11, fontFamily: 'monospace'),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _posController,
            keyboardType: TextInputType.number,
            style: const TextStyle(color: Colors.white, fontFamily: 'monospace', fontSize: 16),
            decoration: const InputDecoration(
              isDense: true,
              border: OutlineInputBorder(),
              focusedBorder: OutlineInputBorder(
                borderSide: BorderSide(color: CyberTheme.badgePosition),
              ),
            ),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('CANCEL', style: TextStyle(color: CyberTheme.textSecondary)),
        ),
        ElevatedButton(
          style: ElevatedButton.styleFrom(
            backgroundColor: CyberTheme.badgePosition,
            foregroundColor: Colors.white,
          ),
          onPressed: () {
            final val = int.tryParse(_posController.text.trim());
            if (val != null && val >= 1 && val <= widget.totalCount) {
              widget.onSetPosition(val - 1);
            }
            Navigator.of(context).pop();
          },
          child: const Text('MOVE'),
        ),
      ],
    );
  }
}
