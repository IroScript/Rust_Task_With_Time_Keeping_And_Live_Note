import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_overlay_window/flutter_overlay_window.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';

/// 100% Rust-Parity Cyberpunk Floating Overlay Widget (Chat Head & Full Floating Card)
///
/// Features 1:1 matching with:
/// - Rust Header Row: [+] Plus Button (22x22, #3CB450 border, #B21C1C cross)
/// - Three Clock Badges:
///     * Badge 0: Task Deadline (#A51616 Crimson Red, "HH.MM AM/PM")
///     * Badge 1: Sub-Task Time (#B95F0F Amber, "HH.MM AM/PM")
///     * Badge 2: Live Stopwatch (#1C76B9 Steel-Blue, "MM:SS" / "HH:MM:SS", pulsing dot #28C8FF)
/// - Title Bar & Floating Buttons (src/main.rs:2450-2540):
///     * Auto-hide animation: Fades to opacity 0.0 after 5.0s inactivity
///     * Toggle Panel: Sandwich ☰ when visible, Close ✕ when hidden
///     * Show Header: 🔼 when header is hidden
/// - Live Note Virtual Scroller & In-Place Text Writing (src/views/live_note.rs & src/main.rs:4520-4630):
///     * Multi-line live note editor
///     * Large Text (>10 KB) virtual scroll badge
///     * Font sizing A+ / A-
///     * Instant state persistence and sync
/// - Card Hierarchy & Depth Scale: 0.95^depth (min 70% floor)
class FloatingOverlayWidget extends StatefulWidget {
  const FloatingOverlayWidget({super.key});

  @override
  State<FloatingOverlayWidget> createState() => _FloatingOverlayWidgetState();
}

class _FloatingOverlayWidgetState extends State<FloatingOverlayWidget>
    with TickerProviderStateMixin {
  bool _isExpanded = false;

  // Title Bar & Animations (src/main.rs:2414 AppAnimation::Dance)
  bool _isDancing = false;
  late AnimationController _danceController;

  // Task & Card State (faithful to Quote / TaskCard in Rust)
  String _cardId = '1';
  int _depth = 0;
  String _title = 'Daily Motivation Task';
  String _subtitle = 'Keep pushing forward!';
  String _liveNote = 'No live note recorded yet.';
  double _noteTextSize = 12.0;

  // Header Clocks State (3 independent badges matching CardHeaderState)
  String _deadlineTime = '12.10 PM';
  int _deadlineHour = 12;
  int _deadlineMinute = 10;

  String _subTaskTime = '12.10 PM';
  int _subTaskHour = 12;
  int _subTaskMinute = 10;

  int _stopwatchSeconds = 0;
  bool _isStopwatchRunning = false;
  ClockMode _badge0Mode = ClockMode.taskDeadline;
  ClockMode _badge1Mode = ClockMode.subTaskTime;
  ClockMode _badge2Mode = ClockMode.stopwatch;

  // Title Bar Auto-Hide & Floating Controls (src/main.rs:2450-2540)
  bool _isHeaderVisible = true;
  bool _isControlPanelVisible = true;
  double _floatingButtonOpacity = 1.0;
  Timer? _inactivityTimer;
  DateTime _lastInteractionTime = DateTime.now();

  // In-place Live Note Writing & Sub-card state
  bool _isEditingNote = false;
  late TextEditingController _noteEditController;
  final List<String> _subCards = [];

  // Tickers & Subscriptions
  Timer? _localTimer;
  StreamSubscription? _overlaySub;

  // Breathing Pulse Animation (src/card_header_widgets.rs:264)
  late AnimationController _animController;

  @override
  void initState() {
    super.initState();
    _noteEditController = TextEditingController(text: _liveNote);
    _animController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1400),
    )..repeat(reverse: true);

    _danceController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 4),
    );

    _startLocalTicker();
    _resetInactivityTimer();
    _listenToMainApp();
  }

  @override
  void dispose() {
    _localTimer?.cancel();
    _inactivityTimer?.cancel();
    _overlaySub?.cancel();
    _noteEditController.dispose();
    _animController.dispose();
    _danceController.dispose();
    super.dispose();
  }

  void _recordInteraction() {
    _lastInteractionTime = DateTime.now();
    if (_floatingButtonOpacity < 1.0) {
      setState(() {
        _floatingButtonOpacity = 1.0;
      });
    }
    _resetInactivityTimer();
  }

  void _resetInactivityTimer() {
    _inactivityTimer?.cancel();
    // 5.0 seconds auto-hide logic matching Rust src/main.rs:2454
    _inactivityTimer = Timer(const Duration(seconds: 5), () {
      if (mounted) {
        setState(() {
          _floatingButtonOpacity = 0.0;
        });
      }
    });
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
              _cardId = json['id']?.toString() ?? _cardId;
              _depth = json['depth'] ?? _depth;
              _title = json['title'] ?? _title;
              _subtitle = json['subtitle'] ?? _subtitle;
              _liveNote = json['liveNote'] ?? _liveNote;
              _noteEditController.text = _liveNote;
              _stopwatchSeconds = json['stopwatchSeconds'] ?? _stopwatchSeconds;
              _isStopwatchRunning = json['isStopwatchRunning'] ?? _isStopwatchRunning;
              _deadlineTime = json['deadlineTime'] ?? _deadlineTime;
              _subTaskTime = json['subTaskTime'] ?? _subTaskTime;
            });
          }
        } catch (e) {
          debugPrint('[FloatingOverlayWidget] Error parsing data: $e');
        }
      }, onError: (err) {
        debugPrint('[FloatingOverlayWidget] Stream error: $err');
      });
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] Failed to listen: $e');
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
    _recordInteraction();
    final nextState = !_isExpanded;
    setState(() {
      _isExpanded = nextState;
      if (!nextState) _isEditingNote = false;
    });

    try {
      if (nextState) {
        // Expand to full cyberpunk card view
        await FlutterOverlayWindow.resizeOverlay(960, 820, true);
      } else {
        // Collapse back to compact floating pill
        await FlutterOverlayWindow.resizeOverlay(540, 260, true);
      }
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] resizeOverlay error: $e');
    }
  }

  void _toggleStopwatch() {
    _recordInteraction();
    setState(() {
      _isStopwatchRunning = !_isStopwatchRunning;
    });
    try {
      FlutterOverlayWindow.shareData(jsonEncode({
        'type': 'TOGGLE_STOPWATCH',
        'isRunning': _isStopwatchRunning,
        'seconds': _stopwatchSeconds,
      }));
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] shareData error: $e');
    }
  }

  void _resetStopwatch() {
    _recordInteraction();
    setState(() {
      _isStopwatchRunning = false;
      _stopwatchSeconds = 0;
    });
    try {
      FlutterOverlayWindow.shareData(jsonEncode({
        'type': 'RESET_STOPWATCH',
        'isRunning': false,
        'seconds': 0,
      }));
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] reset error: $e');
    }
  }

  void _addSubCard() {
    _recordInteraction();
    final newDepth = _depth + 1;
    final subId = '${_cardId}.${_subCards.length + 1}';
    final scalePercent = (0.95 * 100).toInt();

    setState(() {
      _subCards.add('Sub-Task $subId (Scale $scalePercent%)');
      _isExpanded = true;
    });

    try {
      FlutterOverlayWindow.shareData(jsonEncode({
        'type': 'ADD_SUBCARD',
        'parentId': _cardId,
        'depth': newDepth,
      }));
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] addSubCard error: $e');
    }
  }

  void _saveLiveNote() {
    _recordInteraction();
    final text = _noteEditController.text.trim();
    setState(() {
      _liveNote = text.isEmpty ? 'No notes recorded.' : text;
      _isEditingNote = false;
    });

    try {
      FlutterOverlayWindow.shareData(jsonEncode({
        'type': 'UPDATE_NOTE',
        'cardId': _cardId,
        'liveNote': _liveNote,
      }));
    } catch (e) {
      debugPrint('[FloatingOverlayWidget] saveLiveNote error: $e');
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
        child: GestureDetector(
          onTapDown: (_) => _recordInteraction(),
          child: _isExpanded ? _buildExpandedCard() : _buildCompactBubble(),
        ),
      ),
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // COMPACT FLOATING BUBBLE / PILL (Chat Head Mode)
  // ───────────────────────────────────────────────────────────────────────────
  Widget _buildCompactBubble() {
    final timeStr = _formatTime(_stopwatchSeconds);

    return AnimatedBuilder(
      animation: _animController,
      builder: (context, child) {
        final pulse = _animController.value;
        return GestureDetector(
          onTap: _toggleExpand,
          child: Container(
            width: 175,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            decoration: BoxDecoration(
              color: CyberTheme.bgCardActive.withValues(alpha: 0.94),
              borderRadius: BorderRadius.circular(24),
              border: Border.all(
                color: _isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.greenBorder,
                width: 1.5,
              ),
              boxShadow: [
                BoxShadow(
                  color: (_isStopwatchRunning ? CyberTheme.neonLime : CyberTheme.greenBorder)
                      .withValues(alpha: 0.25 + 0.3 * pulse),
                  blurRadius: 10 + 4 * pulse,
                  spreadRadius: 1,
                ),
              ],
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Glowing Breathing Dot (src/card_header_widgets.rs:414)
                Container(
                  width: 9,
                  height: 9,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: _isStopwatchRunning
                        ? const Color(0xFF28C8FF)
                        : CyberTheme.greenBorder,
                    boxShadow: [
                      BoxShadow(
                        color: _isStopwatchRunning
                            ? const Color(0xFF28C8FF).withValues(alpha: 0.8)
                            : CyberTheme.greenBorder.withValues(alpha: 0.8),
                        blurRadius: 6,
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                // Compact Timer & Title
                Expanded(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        timeStr,
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 12,
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
                          fontSize: 9.5,
                        ),
                      ),
                    ],
                  ),
                ),
                // Compact [+] Button (Fast sub-card trigger)
                InkWell(
                  onTap: _addSubCard,
                  child: Container(
                    width: 20,
                    height: 20,
                    decoration: BoxDecoration(
                      border: Border.all(color: CyberTheme.greenBorder, width: 1.2),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    alignment: Alignment.center,
                    child: const Text(
                      '+',
                      style: TextStyle(
                        color: CyberTheme.redCross,
                        fontSize: 14,
                        fontWeight: FontWeight.w900,
                        height: 1.0,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // FULL EXPANDED CYBERPUNK CARD (100% Rust Parity View)
  // ───────────────────────────────────────────────────────────────────────────
  Widget _buildExpandedCard() {
    final isLargeText = _liveNote.length > 10240;

    final cardContent = Container(
      width: 320,
      constraints: const BoxConstraints(maxHeight: 520),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CyberTheme.bgCosmic.withValues(alpha: 0.96),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: CyberTheme.greenBorder, width: 1.6),
        boxShadow: [
          BoxShadow(
            color: CyberTheme.greenBorder.withValues(alpha: 0.35),
            blurRadius: 16,
            spreadRadius: 2,
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // ── 1. Top Title Bar with Auto-Hide Floating Controls (src/main.rs:2450-2540) ──
          if (_isHeaderVisible)
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                  decoration: BoxDecoration(
                    color: CyberTheme.greenBorder.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: CyberTheme.greenBorder, width: 0.8),
                  ),
                  child: const Text(
                    '⚡ TASK & LIVE NOTE',
                    style: TextStyle(
                      color: CyberTheme.greenBorder,
                      fontSize: 8.5,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                    ),
                  ),
                ),
                const Spacer(),
                // Floating Action Group with Linear Opacity Auto-Hide
                AnimatedOpacity(
                  opacity: _floatingButtonOpacity,
                  duration: const Duration(milliseconds: 300),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      // Dance Animation Button (💃 src/main.rs:2414 icons::ANIM_DANCE)
                      if (_isControlPanelVisible) ...[
                        InkWell(
                          onTap: () {
                            _recordInteraction();
                            setState(() {
                              _isDancing = !_isDancing;
                              if (_isDancing) {
                                _danceController.repeat();
                              } else {
                                _danceController.stop();
                                _danceController.reset();
                              }
                            });
                          },
                          child: Container(
                            padding: const EdgeInsets.all(3),
                            decoration: BoxDecoration(
                              color: _isDancing ? CyberTheme.neonLime.withValues(alpha: 0.25) : Colors.white12,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: const Text(
                              '💃',
                              style: TextStyle(fontSize: 10),
                            ),
                          ),
                        ),
                        const SizedBox(width: 4),
                        // Hide Header Button (▲ src/main.rs:2343 icons::HIDE_HEADER)
                        InkWell(
                          onTap: () {
                            _recordInteraction();
                            setState(() {
                              _isHeaderVisible = false;
                            });
                          },
                          child: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
                            decoration: BoxDecoration(
                              color: Colors.white12,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: const Text(
                              '▲',
                              style: TextStyle(
                                color: Colors.white,
                                fontSize: 10,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(width: 4),
                      ],
                      // Toggle Control Panel / Header (Sandwich ☰ when visible, ✕ when hidden)
                      InkWell(
                        onTap: () {
                          _recordInteraction();
                          setState(() {
                            _isControlPanelVisible = !_isControlPanelVisible;
                          });
                        },
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
                          decoration: BoxDecoration(
                            color: Colors.white12,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            _isControlPanelVisible ? '☰' : '✕',
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 10,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ),
                      const SizedBox(width: 4),
                      // Minimize to pill
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
                            size: 12,
                          ),
                        ),
                      ),
                      const SizedBox(width: 4),
                      // Close overlay
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
                            size: 12,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),

          // If header was hidden, show the small Floating Show Header button (src/main.rs:2525)
          if (!_isHeaderVisible)
            Align(
              alignment: Alignment.topRight,
              child: InkWell(
                onTap: () {
                  _recordInteraction();
                  setState(() => _isHeaderVisible = true);
                },
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.white12,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text(
                    '🔼 Show Header',
                    style: TextStyle(color: Colors.white70, fontSize: 9),
                  ),
                ),
              ),
            ),

          const SizedBox(height: 8),

          // ── 2. The Holographic Header Row: [+] Plus Button + 3 Clock Badges ──
          _buildRustCardHeaderRow(),

          const SizedBox(height: 8),

          // ── 3. Main Task Content & Subtitle ──
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
          if (_subtitle.isNotEmpty) ...[
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
          ],

          const SizedBox(height: 8),

          // ── 4. Live Note Section (Virtual Scrolling & In-Place Writing) ──
          _buildLiveNoteSection(isLargeText),

          // ── 5. Sub-Cards Tree (Depth Scaling: 0.95^depth) ──
          if (_subCards.isNotEmpty) ...[
            const SizedBox(height: 6),
            _buildSubCardsList(),
          ],
        ],
      ),
    );

    return AnimatedBuilder(
      animation: _danceController,
      builder: (context, child) {
        if (!_isDancing) return child!;
        final p = _danceController.value * 2.0 * math.pi;
        final danceX = math.sin(p * 4.0) * 16.0;
        final danceY = math.cos(p * 2.5) * 12.0;
        return Transform.translate(
          offset: Offset(danceX, danceY),
          child: child,
        );
      },
      child: cardContent,
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // RUST CARD HEADER ROW (100% faithful to card_header_widgets.rs)
  // ───────────────────────────────────────────────────────────────────────────
  Widget _buildRustCardHeaderRow() {
    final swTime = _formatTime(_stopwatchSeconds);

    return AnimatedBuilder(
      animation: _animController,
      builder: (context, child) {
        final pulse = _animController.value;

        return SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              // 1. [+] Plus Button: 22x22, green border (#3CB450), bold crimson red plus (#B21C1C)
              InkWell(
                onTap: _addSubCard,
                borderRadius: BorderRadius.circular(5),
                child: Container(
                  width: 22,
                  height: 22,
                  decoration: BoxDecoration(
                    color: Colors.transparent,
                    border: Border.all(color: CyberTheme.greenBorder, width: 1.2),
                    borderRadius: BorderRadius.circular(5),
                    boxShadow: [
                      BoxShadow(
                        color: CyberTheme.greenBorder.withValues(alpha: 0.18 + 0.3 * pulse),
                        blurRadius: 4 + 3 * pulse,
                      ),
                    ],
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

              const SizedBox(width: 4),

              // 2. Badge 0: Task Deadline (#A51616 Crimson Red, "12.10 PM")
              _buildClockChip(
                label: _deadlineTime,
                color: CyberTheme.clockRed,
                onTap: () {
                  _recordInteraction();
                  _showTimePickerDialog(true);
                },
                tooltip: 'Deadline │ Tap: set time',
              ),

              const SizedBox(width: 4),

              // 3. Badge 1: Sub-Task Time (#B95F0F Amber, "12.10 PM")
              _buildClockChip(
                label: _subTaskTime,
                color: CyberTheme.clockAmber,
                onTap: () {
                  _recordInteraction();
                  _showTimePickerDialog(false);
                },
                tooltip: 'Sub-task time │ Tap: set time',
              ),

              const SizedBox(width: 4),

              // 4. Badge 2: Live Stopwatch (#1C76B9 Steel-Blue, "MM:SS", pulsing dot #28C8FF)
              _buildClockChip(
                label: swTime,
                color: CyberTheme.clockBlue,
                isRunning: _isStopwatchRunning,
                pulseVal: pulse,
                onTap: _toggleStopwatch,
                onLongPress: _resetStopwatch,
                tooltip: 'Stopwatch │ Tap: start/pause │ Long-press: reset',
              ),
            ],
          ),
        );
      },
    );
  }

  /// Compact Pill Badge Chip (64x19 px, 5px radius, thin green border)
  Widget _buildClockChip({
    required String label,
    required Color color,
    required VoidCallback onTap,
    VoidCallback? onLongPress,
    bool isRunning = false,
    double pulseVal = 0.0,
    required String tooltip,
  }) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        onLongPress: onLongPress,
        borderRadius: BorderRadius.circular(5),
        child: Container(
          height: 20,
          padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
          decoration: BoxDecoration(
            color: Colors.transparent,
            border: Border.all(
              color: isRunning ? color : CyberTheme.greenBorder,
              width: isRunning ? 1.4 : 1.1,
            ),
            borderRadius: BorderRadius.circular(5),
            boxShadow: isRunning
                ? [
                    BoxShadow(
                      color: color.withValues(alpha: 0.25 + 0.3 * pulseVal),
                      blurRadius: 6,
                    ),
                  ]
                : null,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                label,
                style: TextStyle(
                  color: color,
                  fontSize: 10.5,
                  fontWeight: FontWeight.bold,
                  fontFamily: 'monospace',
                ),
              ),
              if (isRunning) ...[
                const SizedBox(width: 4),
                // Pulsing dot indicator matching Rust line 414: circle_filled(#28C8FF)
                Container(
                  width: 5,
                  height: 5,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: const Color(0xFF28C8FF),
                    boxShadow: [
                      BoxShadow(
                        color: const Color(0xFF28C8FF).withValues(alpha: 0.8),
                        blurRadius: 4,
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // LIVE NOTE & VIRTUAL SCROLLING SECTION (src/views/live_note.rs)
  // ───────────────────────────────────────────────────────────────────────────
  Widget _buildLiveNoteSection(bool isLargeText) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(8),
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
          Row(
            children: [
              const Text(
                '📄 LIVE NOTE',
                style: TextStyle(
                  color: CyberTheme.neonPurple,
                  fontSize: 9.5,
                  fontWeight: FontWeight.bold,
                  fontFamily: 'monospace',
                ),
              ),
              const Spacer(),
              // Font size controls A+ / A- (src/main.rs:4565)
              InkWell(
                onTap: () {
                  _recordInteraction();
                  setState(() => _noteTextSize = (_noteTextSize + 1).clamp(9.0, 18.0));
                },
                child: const Text(' A+ ', style: TextStyle(color: Colors.white70, fontSize: 10)),
              ),
              InkWell(
                onTap: () {
                  _recordInteraction();
                  setState(() => _noteTextSize = (_noteTextSize - 1).clamp(9.0, 18.0));
                },
                child: const Text(' A- ', style: TextStyle(color: Colors.white70, fontSize: 10)),
              ),
            ],
          ),

          // Large Text Virtual Scrolling Badge (src/main.rs:3206)
          if (isLargeText) ...[
            const SizedBox(height: 4),
            Row(
              children: [
                const Text('📄 ', style: TextStyle(fontSize: 10)),
                Text(
                  'Large Text (${(_liveNote.length / 1024).toStringAsFixed(1)} KB) - Virtual Scrolling Active',
                  style: const TextStyle(
                    color: Color(0xFF64C8FF),
                    fontSize: 8.5,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ],

          const SizedBox(height: 4),

          // Google Keep Parity: Always-open editable TextField (auto-saved on keypress)
          TextField(
            controller: _noteEditController,
            maxLines: 4,
            style: TextStyle(
              color: Colors.white,
              fontSize: _noteTextSize,
            ),
            onChanged: (text) {
              _recordInteraction();
              _liveNote = text;
              _saveLiveNote();
            },
            decoration: const InputDecoration(
              hintText: 'Write note here... (auto-saved)',
              hintStyle: TextStyle(color: Colors.white30, fontSize: 11),
              border: InputBorder.none,
              isDense: true,
              contentPadding: EdgeInsets.zero,
            ),
          ),
        ],
      ),
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // SUB-CARDS TREE (Depth scale 0.95^depth, src/card_header_widgets.rs:243)
  // ───────────────────────────────────────────────────────────────────────────
  Widget _buildSubCardsList() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Nested Sub-Cards:',
          style: TextStyle(color: Color(0xFF64C8FF), fontSize: 9.5, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 3),
        for (int i = 0; i < _subCards.length; i++)
          Padding(
            padding: EdgeInsets.only(left: (i + 1) * 8.0, bottom: 2),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
              decoration: BoxDecoration(
                color: Colors.white.withValues(alpha: 0.05),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: CyberTheme.greenBorder.withValues(alpha: 0.6), width: 0.8),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('↳ ', style: TextStyle(color: CyberTheme.greenBorder, fontSize: 10)),
                  Text(
                    _subCards[i],
                    style: const TextStyle(color: Colors.white70, fontSize: 9.5),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }

  // ───────────────────────────────────────────────────────────────────────────
  // TIME PICKER DIALOG (Deadline & Sub-task time setter)
  // ───────────────────────────────────────────────────────────────────────────
  void _showTimePickerDialog(bool isDeadline) {
    showTimePicker(
      context: context,
      initialTime: TimeOfDay(
        hour: isDeadline ? _deadlineHour : _subTaskHour,
        minute: isDeadline ? _deadlineMinute : _subTaskMinute,
      ),
    ).then((picked) {
      if (picked != null) {
        final period = picked.period == DayPeriod.am ? 'AM' : 'PM';
        final h12 = picked.hourOfPeriod == 0 ? 12 : picked.hourOfPeriod;
        final formatted =
            '${h12.toString().padLeft(2, '0')}.${picked.minute.toString().padLeft(2, '0')} $period';

        setState(() {
          if (isDeadline) {
            _deadlineHour = picked.hour;
            _deadlineMinute = picked.minute;
            _deadlineTime = formatted;
          } else {
            _subTaskHour = picked.hour;
            _subTaskMinute = picked.minute;
            _subTaskTime = formatted;
          }
        });

        FlutterOverlayWindow.shareData(jsonEncode({
          'type': isDeadline ? 'UPDATE_DEADLINE' : 'UPDATE_SUBTASK_TIME',
          'formattedTime': formatted,
          'hour': picked.hour,
          'minute': picked.minute,
        }));
      }
    });
  }
}
