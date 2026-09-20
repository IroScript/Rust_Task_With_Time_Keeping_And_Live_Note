import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../theme/cyber_theme.dart';
import '../../models/task_card.dart';

class ScheduleDialog extends StatefulWidget {
  final TaskCard card;
  final Function(String date, String time) onSave;

  const ScheduleDialog({
    super.key,
    required this.card,
    required this.onSave,
  });

  @override
  State<ScheduleDialog> createState() => _ScheduleDialogState();
}

class _ScheduleDialogState extends State<ScheduleDialog> {
  late TextEditingController _dateController;
  late TextEditingController _timeController;

  @override
  void initState() {
    super.initState();
    final now = DateTime.now();
    _dateController = TextEditingController(
      text: widget.card.scheduledDate ?? DateFormat('yyyy-MM-dd').format(now),
    );
    _timeController = TextEditingController(
      text: widget.card.scheduledTime ?? DateFormat('HH:mm').format(now),
    );
  }

  @override
  void dispose() {
    _dateController.dispose();
    _timeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: const Color(0xF20A0F1D),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: CyberTheme.badgeSchedule, width: 1.5),
      ),
      title: const Row(
        children: [
          Icon(Icons.access_time_filled, color: CyberTheme.badgeSchedule, size: 18),
          SizedBox(width: 8),
          Text(
            'SCHEDULE TIME',
            style: TextStyle(
              color: CyberTheme.badgeSchedule,
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
          TextField(
            controller: _dateController,
            style: const TextStyle(color: Colors.white, fontFamily: 'monospace'),
            decoration: const InputDecoration(
              labelText: 'Date (YYYY-MM-DD)',
              labelStyle: TextStyle(color: CyberTheme.textSecondary),
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _timeController,
            style: const TextStyle(color: Colors.white, fontFamily: 'monospace'),
            decoration: const InputDecoration(
              labelText: 'Time (HH:MM)',
              labelStyle: TextStyle(color: CyberTheme.textSecondary),
              border: OutlineInputBorder(),
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
            backgroundColor: CyberTheme.badgeSchedule,
            foregroundColor: Colors.white,
          ),
          onPressed: () {
            widget.onSave(_dateController.text.trim(), _timeController.text.trim());
            Navigator.of(context).pop();
          },
          child: const Text('SET SCHEDULE'),
        ),
      ],
    );
  }
}
