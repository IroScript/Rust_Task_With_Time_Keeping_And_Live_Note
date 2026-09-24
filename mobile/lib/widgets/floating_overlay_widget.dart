import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_overlay_window/flutter_overlay_window.dart';
import '../theme/cyber_theme.dart';

/// Cyberpunk Floating Overlay Widget (Chat Head / Floating Bubble on Mobile)
class FloatingOverlayWidget extends StatefulWidget {
  const FloatingOverlayWidget({super.key});

  @override
  State<FloatingOverlayWidget> createState() => _FloatingOverlayWidgetState();
}

class _FloatingOverlayWidgetState extends State<FloatingOverlayWidget> {
  bool _isExpanded = false;
  String _title = 'Daily Motivation Task';
  String _subtitle = 'Keep pushing forward!';
  String _liveNote = 'No live note recorded yet.';
  int _stopwatchSeconds = 0;
  bool _isStopwatchRunning = false;
  String _clockMode = 'Stopwatch';
  Timer? _localTimer;
  StreamSubscription? _overlaySub;

  @override
  void initState() {
    super.initState();
    _startLocalTicker();
    _listenToMainApp();
  }

  @override
  void dispose() {
    _localTimer?.cancel();
    _overlaySub?.cancel();
    super.dispose();
  }

  void _startLocalTicker() {
    _localTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (_isStopwatchRunning && mounted) {
        setState(() {
          _stopwatchSeconds++;
        });
      }
    });
  }

  void _listenToMainApp() {
    try {
      _overlaySub = FlutterOverlayWindow.overlayListener.listen((data) {
        if (data == null) return;
        try {
          final Map<String, dynamic> json =
              data is String ? jsonDecode(data) : Map<String, dynamic>.from(data);

          if (json['type'] == 'CARD_UPDATE' && mounted) {
            setState(() {
              _title = json['title'] ?? _title;
              _subtitle = json['subtitle'] ?? _subtitle;
              _liveNote = json['liveNote'] ?? _liveNote;
              _stopwatchSeconds = json['stopwatchSeconds'] ?? _stopwatchSeconds;
              _isStopwatchRunning = json['isStopwatchRunning'] ?? _isStopwatchRunning;
              _clockMode = json['clockMode'] ?? _clockMode;
            });
          }
        } catch (e) {
          debugPrint('[FloatingOverlayWidget] Error parsing data: $e');
        }
      }, onError: (err) {
        debugPrint('[FloatingOverlayWidget] Stream error: $err');
      });
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] Failed to listen to overlayListener: $e');
    }
  }

  String _formatTime(int totalSecs) {
    final h = totalSecs ~/ 3600;
    final m = (totalSecs % 3600) ~/ 60;
    final s = totalSecs % 60;
    if (h > 0) {
      return '${h.toString().padLeft(2, '0')}:${m.toString().padLeft(2, '0')}:${s.toString().padLeft(2, '0')}';
    }
    return '${m.toString().padLeft(2, '0')}:${s.toString().padLeft(2, '0')}';
  }

  Future<void> _toggleExpand() async {
    final nextState = !_isExpanded;
    setState(() {
      _isExpanded = nextState;
    });

    try {
      if (nextState) {
        // Expand to floating card size (950 x 750 px)
        await FlutterOverlayWindow.resizeOverlay(950, 750, true);
      } else {
        // Collapse back to compact floating pill (540 x 260 px)
        await FlutterOverlayWindow.resizeOverlay(540, 260, true);
      }
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] resizeOverlay error: $e');
    }
  }

  void _toggleStopwatch() {
    setState(() {
      _isStopwatchRunning = !_isStopwatchRunning;
    });
    try {
      // Inform main app
      FlutterOverlayWindow.shareData(jsonEncode({
        'type': 'TOGGLE_STOPWATCH',
        'isRunning': _isStopwatchRunning,
        'seconds': _stopwatchSeconds,
      }));
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] shareData error: $e');
    }
  }

  Future<void> _closeOverlay() async {
    try {
      await FlutterOverlayWindow.closeOverlay();
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] closeOverlay error: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: Center(
        child: _isExpanded ? _buildExpandedCard() : _buildCompactBubble(),
      ),
    );
  }

  /// Compact Floating Bubble / Pill (Chat Head Style)
  Widget _buildCompactBubble() {
    final timeStr = _formatTime(_stopwatchSeconds);

    return GestureDetector(
      onTap: _toggleExpand,
      child: Container(
        width: 170,
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        decoration: BoxDecoration(
          color: CyberTheme.bgCardActive.withValues(alpha: 0.92),
          borderRadius: BorderRadius.circular(24),
          border: Border.all(
            color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
            width: 1.5,
          ),
          boxShadow: [
            BoxShadow(
              color: (_isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan)
                  .withValues(alpha: 0.35),
              blurRadius: 10,
              spreadRadius: 1,
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Glowing Indicator
            Container(
              width: 10,
              height: 10,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                boxShadow: [
                  BoxShadow(
                    color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                    blurRadius: 6,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 8),
            // Timer & Status
            Expanded(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    timeStr,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 13,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                    ),
                  ),
                  Text(
                    _title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      color: CyberTheme.textSecondary,
                      fontSize: 10,
                    ),
                  ),
                ],
              ),
            ),
            // Expand Icon
            const Icon(
              Icons.open_in_full_rounded,
              color: CyberTheme.neonCyan,
              size: 14,
            ),
          ],
        ),
      ),
    );
  }

  /// Expanded Floating Task & Live Note Card
  Widget _buildExpandedCard() {
    final timeStr = _formatTime(_stopwatchSeconds);

    return Container(
      width: 310,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CyberTheme.bgPrimary.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: CyberTheme.neonCyan, width: 1.5),
        boxShadow: [
          BoxShadow(
            color: CyberTheme.neonCyan.withValues(alpha: 0.3),
            blurRadius: 14,
            spreadRadius: 2,
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header: Title & Action Buttons
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                decoration: BoxDecoration(
                  color: CyberTheme.neonCyan.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: CyberTheme.neonCyan, width: 0.8),
                ),
                child: const Text(
                  'FLOATING TASK',
                  style: TextStyle(
                    color: CyberTheme.neonCyan,
                    fontSize: 9,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
              const Spacer(),
              // Minimize button
              InkWell(
                onTap: _toggleExpand,
                child: Container(
                  padding: const EdgeInsets.all(3),
                  decoration: BoxDecoration(
                    color: Colors.white12,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Icon(
                    Icons.close_fullscreen_rounded,
                    color: CyberTheme.neonYellow,
                    size: 14,
                  ),
                ),
              ),
              const SizedBox(width: 6),
              // Close button
              InkWell(
                onTap: _closeOverlay,
                child: Container(
                  padding: const EdgeInsets.all(3),
                  decoration: BoxDecoration(
                    color: Colors.red.withValues(alpha: 0.3),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Icon(
                    Icons.close,
                    color: Colors.redAccent,
                    size: 14,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),

          // Main Task Text
          Text(
            _title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 13,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 2),
          Text(
            _subtitle,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              color: CyberTheme.textSecondary,
              fontSize: 10,
            ),
          ),
          const SizedBox(height: 8),

          // Timer & Controls Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
            decoration: BoxDecoration(
              color: Colors.black45,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: CyberTheme.borderSubtle),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.timer_outlined,
                        size: 14,
                        color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        timeStr,
                        style: TextStyle(
                          color: _isStopwatchRunning ? CyberTheme.neonLime : Colors.white,
                          fontSize: 12,
                          fontWeight: FontWeight.bold,
                          fontFamily: 'monospace',
                        ),
                      ),
                      const SizedBox(width: 4),
                      Flexible(
                        child: Text(
                          '($_clockMode)',
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            color: CyberTheme.textMuted,
                            fontSize: 9,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 6),
                // Play / Pause Toggle Button
                InkWell(
                  onTap: _toggleStopwatch,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: _isStopwatchRunning
                          ? CyberTheme.neonLime.withValues(alpha: 0.2)
                          : CyberTheme.neonCyan.withValues(alpha: 0.2),
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(
                        color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                        width: 0.8,
                      ),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          _isStopwatchRunning ? Icons.pause : Icons.play_arrow,
                          size: 12,
                          color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                        ),
                        const SizedBox(width: 2),
                        Text(
                          _isStopwatchRunning ? 'PAUSE' : 'START',
                          style: TextStyle(
                            fontSize: 9,
                            fontWeight: FontWeight.bold,
                            color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.neonCyan,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 8),

          // Live Note Box
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(7),
            decoration: BoxDecoration(
              color: const Color(0xFF090D14),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: CyberTheme.neonPurple.withValues(alpha: 0.5),
                width: 0.8,
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                const Row(
                  children: [
                    Text(
                      '📝 LIVE NOTE',
                      style: TextStyle(
                        color: CyberTheme.neonPurple,
                        fontSize: 9,
                        fontWeight: FontWeight.bold,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 3),
                Text(
                  _liveNote.isEmpty ? 'No notes added.' : _liveNote,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: CyberTheme.textPrimary,
                    fontSize: 11,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
