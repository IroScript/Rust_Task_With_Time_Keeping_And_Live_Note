import 'package:flutter/material.dart';
import '../../theme/cyber_theme.dart';

class IntervalDialog extends StatefulWidget {
  final int currentInterval;
  final Function(int newInterval) onSave;

  const IntervalDialog({
    super.key,
    required this.currentInterval,
    required this.onSave,
  });

  @override
  State<IntervalDialog> createState() => _IntervalDialogState();
}

class _IntervalDialogState extends State<IntervalDialog> {
  late double _intervalVal;

  @override
  void initState() {
    super.initState();
    _intervalVal = widget.currentInterval.toDouble().clamp(1.0, 120.0);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: const Color(0xF20A0F1D),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: CyberTheme.badgeInterval, width: 1.5),
      ),
      title: const Row(
        children: [
          Icon(Icons.timer, color: CyberTheme.badgeInterval, size: 18),
          SizedBox(width: 8),
          Text(
            'ROTATION INTERVAL (⏱)',
            style: TextStyle(
              color: CyberTheme.badgeInterval,
              fontSize: 13,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            '${_intervalVal.toInt()} Seconds',
            style: const TextStyle(
              color: CyberTheme.neonCyan,
              fontSize: 22,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
          Slider(
            value: _intervalVal,
            min: 1.0,
            max: 120.0,
            divisions: 119,
            activeColor: CyberTheme.badgeInterval,
            onChanged: (val) => setState(() => _intervalVal = val),
          ),
          const Text(
            'Adjust how often motivational quotes rotate automatically.',
            textAlign: TextAlign.center,
            style: TextStyle(color: CyberTheme.textSecondary, fontSize: 11, fontFamily: 'monospace'),
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
            backgroundColor: CyberTheme.badgeInterval,
            foregroundColor: Colors.white,
          ),
          onPressed: () {
            widget.onSave(_intervalVal.toInt());
            Navigator.of(context).pop();
          },
          child: const Text('SAVE INTERVAL'),
        ),
      ],
    );
  }
}
