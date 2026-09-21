import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';
import '../services/api_service.dart';
import '../widgets/task_card_widget.dart';
import '../widgets/control_panel_widget.dart';
import '../widgets/card_size_popup_widget.dart';
import '../widgets/virtual_note_scroller_dialog.dart';
import '../widgets/modals/schedule_dialog.dart';
import '../widgets/modals/position_dialog.dart';
import '../widgets/modals/interval_dialog.dart';
import '../widgets/modals/theme_modal.dart';
import '../widgets/modals/profile_modal.dart';
import '../services/overlay_service.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen>
    with SingleTickerProviderStateMixin {
  final ApiService _apiService = ApiService();
  final List<TaskCard> _cards = [];
  int _selectedCardIndex = 0;
  bool _isAxumOnline = false;

  // Title Bar & HUD State (1:1 with TitleBarState in src/main.rs:799-850)
  double _cardScale = 1.0;
  bool _isCardSizePopupOpen = false;
  double _zoomLevel = 1.0;
  bool _singleQuoteMode = false;
  bool _is3dBgActive = false;
  String _activeAnimation = 'none'; // none, bounce, shake, dance, rotate, dissolve, fly
  String _currentTheme = 'cyberpunk';
  Color _universalFontColor = CyberTheme.textPrimary;

  // Spacing & Gaps (from TextStyleConfig in src/main.rs:567)
  double _mainLineGap = 4.0;
  double _subLineGap = 2.0;
  double _betweenGap = 8.0;

  // Rotation & Timer State
  bool _autoRotationEnabled = true;
  int _rotationIntervalSeconds = 8;
  Timer? _stopwatchTicker;
  Timer? _quoteRotationTicker;

  // Animation controller for Multi-Animation system
  late AnimationController _animController;

  // User Profile
  Map<String, String> _userProfile = {
    'name': 'Irak',
    'email': 'irak@iroscript.org',
    'country': 'Bangladesh',
    'company': 'IroScript',
  };

  @override
  void initState() {
    super.initState();
    _animController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 4),
    )..repeat();

    _initializeApp();
    _startStopwatchTicker();
    _startQuoteRotation();
  }

  @override
  void dispose() {
    _animController.dispose();
    _stopwatchTicker?.cancel();
    _quoteRotationTicker?.cancel();
    super.dispose();
  }

  Future<void> _initializeApp() async {
    await _apiService.loadSavedBaseUrl();
    await _loadSettings();
    await _checkBackendStatus();
    await _loadInitialCards();
  }

  Future<void> _loadSettings() async {
    final settings = await _apiService.loadAppSettings();
    if (settings.isNotEmpty && mounted) {
      setState(() {
        _cardScale = (settings['cardScale'] as num?)?.toDouble() ?? 1.0;
        _zoomLevel = (settings['zoomLevel'] as num?)?.toDouble() ?? 1.0;
        _singleQuoteMode = settings['singleQuoteMode'] ?? false;
        _rotationIntervalSeconds = settings['rotationInterval'] ?? 8;
        _autoRotationEnabled = settings['autoRotation'] ?? true;
        _currentTheme = settings['theme'] ?? 'cyberpunk';
        if (settings['userProfile'] != null) {
          _userProfile = Map<String, String>.from(settings['userProfile']);
        }
      });
    }
  }

  Future<void> _saveSettings() async {
    await _apiService.saveAppSettings({
      'cardScale': _cardScale,
      'zoomLevel': _zoomLevel,
      'singleQuoteMode': _singleQuoteMode,
      'rotationInterval': _rotationIntervalSeconds,
      'autoRotation': _autoRotationEnabled,
      'theme': _currentTheme,
      'userProfile': _userProfile,
    });
  }

  Future<void> _checkBackendStatus() async {
    final online = await _apiService.checkHealth();
    if (mounted) {
      setState(() => _isAxumOnline = online);
    }
  }

  Future<void> _loadInitialCards() async {
    final local = await _apiService.loadLocalCards();
    if (local.isNotEmpty && mounted) {
      setState(() {
        _cards.clear();
        _cards.addAll(local);
      });
    }

    if (_isAxumOnline) {
      final remote = await _apiService.fetchCards();
      if (remote.isNotEmpty && mounted) {
        setState(() {
          _cards.clear();
          _cards.addAll(remote);
        });
        _apiService.saveLocalCards(_cards);
        return;
      }
    }

    if (_cards.isEmpty && mounted) {
      setState(() {
        _cards.addAll([
          TaskCard(
            id: '1',
            mainText: 'Focus on the work - Success is near',
            subText: "Keep pushing - You're doing great! ✨",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
          TaskCard(
            id: '2',
            mainText: 'Stay disciplined - Great things take time',
            subText: "Consistency beats talent when talent sleeps! 🌟",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
          TaskCard(
            id: '3',
            mainText: 'Consistency is the key to mastery',
            subText: "Every small step today compounds tomorrow! 🚀",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
          TaskCard(
            id: '4',
            mainText: 'Small daily improvements lead to stunning results',
            subText: "Deep focus unlocks limitless potential! ⚡",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
        ]);
      });
      _apiService.saveLocalCards(_cards);
    }
  }

  void _startStopwatchTicker() {
    _stopwatchTicker = Timer.periodic(const Duration(seconds: 1), (_) {
      bool changed = false;
      for (var card in _cards) {
        if (card.isStopwatchRunning) {
          card.stopwatchSeconds++;
          changed = true;
        }
      }
      if (changed && mounted) {
        if (OverlayService.instance.isOverlayActive && _cards.isNotEmpty) {
          OverlayService.instance.syncCardData(_cards[_selectedCardIndex]);
        }
        setState(() {});
      }
    });
  }

  void _startQuoteRotation() {
    _quoteRotationTicker?.cancel();
    if (!_autoRotationEnabled || _cards.isEmpty) return;

    _quoteRotationTicker = Timer.periodic(
      Duration(seconds: _rotationIntervalSeconds),
      (_) {
        if (_cards.isNotEmpty && mounted) {
          _nextQuote();
        }
      },
    );
  }

  void _prevQuote() {
    if (_cards.isEmpty) return;
    setState(() {
      _selectedCardIndex =
          (_selectedCardIndex - 1 + _cards.length) % _cards.length;
    });
  }

  void _nextQuote() {
    if (_cards.isEmpty) return;
    setState(() {
      _selectedCardIndex = (_selectedCardIndex + 1) % _cards.length;
    });
  }

  void _addNewCard({String? title, String? subtitle, int depth = 0, String? parentId}) {
    final nowTime = DateFormat('hh:mm a').format(DateTime.now());
    final newCard = TaskCard(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      mainText: title ?? 'New Task / Motivation Quote',
      subText: subtitle ?? "Keep pushing - You're doing great! ✨",
      startTime: nowTime,
      endTime: nowTime,
      depth: depth,
      parentId: parentId,
      orderIndex: _cards.length,
    );

    setState(() {
      _cards.add(newCard);
      _selectedCardIndex = _cards.length - 1;
    });

    _apiService.saveLocalCards(_cards);
    if (_isAxumOnline) {
      _apiService.createCard(newCard.mainText);
    }
  }

  void _addSubCard(int parentIndex) {
    if (parentIndex >= 0 && parentIndex < _cards.length) {
      final parent = _cards[parentIndex];
      _addNewCard(
        title: 'Sub-task for: ${parent.mainText}',
        depth: parent.depth + 1,
        parentId: parent.id,
      );
    }
  }

  Future<void> _selectTime(int cardIndex, bool isDeadline) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay.now(),
      builder: (context, child) {
        return Theme(
          data: ThemeData.dark().copyWith(
            colorScheme: const ColorScheme.dark(
              primary: CyberTheme.neonCyan,
              surface: CyberTheme.bgCard,
            ),
          ),
          child: child!,
        );
      },
    );

    if (picked != null && mounted) {
      final now = DateTime.now();
      final dt = DateTime(now.year, now.month, now.day, picked.hour, picked.minute);
      final formatted = DateFormat('hh:mm a').format(dt);

      setState(() {
        if (isDeadline) {
          _cards[cardIndex].startTime = formatted;
        } else {
          _cards[cardIndex].endTime = formatted;
        }
      });
      _apiService.saveLocalCards(_cards);
    }
  }

  void _exportQuotesJson() {
    final jsonStr = const JsonEncoder.withIndent('  ').convert(
      _cards.map((c) => c.toLocalJson()).toList(),
    );

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xF20A0F1D),
        title: const Text('EXPORT QUOTES (JSON)', style: TextStyle(color: CyberTheme.neonCyan, fontFamily: 'monospace', fontSize: 13)),
        content: SizedBox(
          width: double.maxFinite,
          child: SingleChildScrollView(
            child: SelectableText(
              jsonStr,
              style: const TextStyle(color: Colors.white, fontSize: 10, fontFamily: 'monospace'),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('CLOSE', style: TextStyle(color: CyberTheme.neonCyan)),
          ),
        ],
      ),
    );
  }

  Future<void> _toggleFloatingOverlay() async {
    final overlay = OverlayService.instance;
    if (overlay.isOverlayActive) {
      await overlay.closeFloatingOverlay();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Floating Widget Closed'),
            duration: Duration(seconds: 2),
          ),
        );
        setState(() {});
      }
    } else {
      final currentCard = _cards.isNotEmpty ? _cards[_selectedCardIndex] : null;
      final success = await overlay.showFloatingOverlay(activeCard: currentCard);
      if (mounted) {
        if (success) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Floating Widget Active! Drag anywhere on screen'),
              duration: Duration(seconds: 3),
              backgroundColor: CyberTheme.bgCardActive,
            ),
          );
        } else {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: const Text(
                'Please toggle ON "Display over other apps" and tap here again',
              ),
              duration: const Duration(seconds: 4),
              backgroundColor: Colors.amber.shade900,
              action: SnackBarAction(
                label: 'SETTINGS',
                textColor: Colors.white,
                onPressed: () => overlay.openAppDetailsSettings(),
              ),
            ),
          );
        }
        setState(() {});
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final isWideScreen = MediaQuery.of(context).size.width >= 820;

    return Scaffold(
      backgroundColor: _is3dBgActive ? const Color(0xFF04060A) : CyberTheme.bgCosmic,
      appBar: _buildCyberTopBar(),
      endDrawer: !isWideScreen
          ? Drawer(
              child: ControlPanelWidget(
                apiService: _apiService,
                isConnected: _isAxumOnline,
                cards: _cards,
                universalFontColor: _universalFontColor,
                mainLineGap: _mainLineGap,
                subLineGap: _subLineGap,
                betweenGap: _betweenGap,
                rotationInterval: _rotationIntervalSeconds,
                isAutoRotationEnabled: _autoRotationEnabled,
                onAddCustomText: (main, aux) => _addNewCard(title: main, subtitle: aux),
                onRefreshServer: _checkBackendStatus,
                onUniversalColorChanged: (col) => setState(() => _universalFontColor = col),
                onLineGapsChanged: (m, s, b) => setState(() {
                  _mainLineGap = m;
                  _subLineGap = s;
                  _betweenGap = b;
                }),
                onIntervalChanged: (val) {
                  setState(() => _rotationIntervalSeconds = val);
                  _startQuoteRotation();
                  _saveSettings();
                },
                onToggleRotation: () {
                  setState(() => _autoRotationEnabled = !_autoRotationEnabled);
                  _startQuoteRotation();
                  _saveSettings();
                },
                onReorderCards: (oldIdx, newIdx) {
                  setState(() {
                    if (newIdx > oldIdx) newIdx -= 1;
                    final item = _cards.removeAt(oldIdx);
                    _cards.insert(newIdx, item);
                    _selectedCardIndex = newIdx;
                  });
                  _apiService.saveLocalCards(_cards);
                },
                onDeleteCard: (idx) {
                  setState(() => _cards.removeAt(idx));
                  _apiService.saveLocalCards(_cards);
                },
                onToggleHideCard: (idx) {
                  setState(() => _cards[idx].isHidden = !_cards[idx].isHidden);
                  _apiService.saveLocalCards(_cards);
                },
                onOpenVirtualScroller: () {
                  if (_cards.isNotEmpty) {
                    _openVirtualNoteDialog(_cards[_selectedCardIndex]);
                  }
                },
              ),
            )
          : null,
      body: Stack(
        children: [
          Column(
            children: [
              // Main Scroll Canvas
              Expanded(
                child: Row(
                  children: [
                    // Main Viewport (Cards Carousel or Single Quote)
                    Expanded(
                      child: AnimatedBuilder(
                        animation: _animController,
                        builder: (context, child) {
                          return _applyMultiAnimation(child!);
                        },
                        child: _cards.isEmpty
                            ? const Center(
                                child: Text(
                                  'No Tasks. Tap [+] to add one.',
                                  style: TextStyle(color: CyberTheme.textSecondary, fontFamily: 'monospace'),
                                ),
                              )
                            : _singleQuoteMode
                                ? _buildSingleQuoteView()
                                : _buildCardsListView(),
                      ),
                    ),

                    // Side Control Panel if Tablet / Desktop width
                    if (isWideScreen)
                      SizedBox(
                        width: 320,
                        child: ControlPanelWidget(
                          apiService: _apiService,
                          isConnected: _isAxumOnline,
                          cards: _cards,
                          universalFontColor: _universalFontColor,
                          mainLineGap: _mainLineGap,
                          subLineGap: _subLineGap,
                          betweenGap: _betweenGap,
                          rotationInterval: _rotationIntervalSeconds,
                          isAutoRotationEnabled: _autoRotationEnabled,
                          onAddCustomText: (main, aux) => _addNewCard(title: main, subtitle: aux),
                          onRefreshServer: _checkBackendStatus,
                          onUniversalColorChanged: (col) => setState(() => _universalFontColor = col),
                          onLineGapsChanged: (m, s, b) => setState(() {
                            _mainLineGap = m;
                            _subLineGap = s;
                            _betweenGap = b;
                          }),
                          onIntervalChanged: (val) {
                            setState(() => _rotationIntervalSeconds = val);
                            _startQuoteRotation();
                            _saveSettings();
                          },
                          onToggleRotation: () {
                            setState(() => _autoRotationEnabled = !_autoRotationEnabled);
                            _startQuoteRotation();
                            _saveSettings();
                          },
                          onReorderCards: (oldIdx, newIdx) {
                            setState(() {
                              if (newIdx > oldIdx) newIdx -= 1;
                              final item = _cards.removeAt(oldIdx);
                              _cards.insert(newIdx, item);
                              _selectedCardIndex = newIdx;
                            });
                            _apiService.saveLocalCards(_cards);
                          },
                          onDeleteCard: (idx) {
                            setState(() => _cards.removeAt(idx));
                            _apiService.saveLocalCards(_cards);
                          },
                          onToggleHideCard: (idx) {
                            setState(() => _cards[idx].isHidden = !_cards[idx].isHidden);
                            _apiService.saveLocalCards(_cards);
                          },
                          onOpenVirtualScroller: () {
                            if (_cards.isNotEmpty) {
                              _openVirtualNoteDialog(_cards[_selectedCardIndex]);
                            }
                          },
                        ),
                      ),
                  ],
                ),
              ),

              // Bottom Status Bar / Neural Feed
              _buildCyberBottomBar(),
            ],
          ),

          // Card Size Adjustment Popup (1:1 with render_card_size_popup)
          if (_isCardSizePopupOpen)
            Positioned(
              top: 4,
              right: 12,
              child: CardSizePopupWidget(
                currentScale: _cardScale,
                onSelectScale: (scale) {
                  setState(() => _cardScale = scale);
                  _saveSettings();
                },
                onClose: () => setState(() => _isCardSizePopupOpen = false),
              ),
            ),
        ],
      ),
    );
  }

  /// Single Quote Mode (1:1 with render_single_quote_mode in src/main.rs:3842)
  Widget _buildSingleQuoteView() {
    if (_cards.isEmpty) return const SizedBox();
    final card = _cards[_selectedCardIndex];

    return Center(
      child: SingleChildScrollView(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 30),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                '— SINGLE QUOTE FOCUS —',
                style: TextStyle(
                  color: CyberTheme.neonLime.withValues(alpha: 0.7),
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                  fontFamily: 'monospace',
                  letterSpacing: 2.0,
                ),
              ),
              const SizedBox(height: 24),
              TaskCardWidget(
                card: card,
                index: _selectedCardIndex,
                totalCards: _cards.length,
                isSelected: true,
                cardScale: _cardScale,
                zoomLevel: _zoomLevel,
                universalFontColor: _universalFontColor,
                mainLineGap: _mainLineGap,
                subLineGap: _subLineGap,
                betweenGap: _betweenGap,
                onTap: () {},
                onAddSubCard: () => _addSubCard(_selectedCardIndex),
                onToggleStopwatch: () {
                  setState(() => card.isStopwatchRunning = !card.isStopwatchRunning);
                  _apiService.saveLocalCards(_cards);
                },
                onSelectDeadline: () => _selectTime(_selectedCardIndex, true),
                onSelectSubTaskTime: () => _selectTime(_selectedCardIndex, false),
                onCycleMode: () {
                  setState(() => card.clockMode = card.clockMode.next);
                  _apiService.saveLocalCards(_cards);
                },
                onMoveUp: () {
                  if (_selectedCardIndex > 0) {
                    setState(() {
                      final item = _cards.removeAt(_selectedCardIndex);
                      _cards.insert(_selectedCardIndex - 1, item);
                      _selectedCardIndex--;
                    });
                    _apiService.saveLocalCards(_cards);
                  }
                },
                onMoveDown: () {
                  if (_selectedCardIndex < _cards.length - 1) {
                    setState(() {
                      final item = _cards.removeAt(_selectedCardIndex);
                      _cards.insert(_selectedCardIndex + 1, item);
                      _selectedCardIndex++;
                    });
                    _apiService.saveLocalCards(_cards);
                  }
                },
                onSetPosition: () => _openPositionDialog(_selectedCardIndex),
                onScheduleTime: () => _openScheduleDialog(_selectedCardIndex),
                onRotationInterval: () => _openIntervalDialog(),
                onToggleHide: () {
                  setState(() => card.isHidden = !card.isHidden);
                  _apiService.saveLocalCards(_cards);
                },
                onDelete: () {
                  setState(() {
                    _cards.removeAt(_selectedCardIndex);
                    if (_selectedCardIndex >= _cards.length) {
                      _selectedCardIndex = (_cards.length - 1).clamp(0, _cards.length);
                    }
                  });
                  _apiService.saveLocalCards(_cards);
                },
                onOpenNote: () => _openVirtualNoteDialog(card),
                onUpdateTitle: (title) {
                  setState(() => card.mainText = title);
                  _apiService.saveLocalCards(_cards);
                },
                onUpdateSubText: (sub) {
                  setState(() => card.subText = sub);
                  _apiService.saveLocalCards(_cards);
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  /// Multi-card list view
  Widget _buildCardsListView() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: 10),
      itemCount: _cards.length,
      itemBuilder: (context, index) {
        final card = _cards[index];
        return TaskCardWidget(
          card: card,
          index: index,
          totalCards: _cards.length,
          isSelected: index == _selectedCardIndex,
          cardScale: _cardScale,
          zoomLevel: _zoomLevel,
          universalFontColor: _universalFontColor,
          mainLineGap: _mainLineGap,
          subLineGap: _subLineGap,
          betweenGap: _betweenGap,
          onTap: () => setState(() => _selectedCardIndex = index),
          onAddSubCard: () => _addSubCard(index),
          onToggleStopwatch: () {
            setState(() => card.isStopwatchRunning = !card.isStopwatchRunning);
            _apiService.saveLocalCards(_cards);
          },
          onSelectDeadline: () => _selectTime(index, true),
          onSelectSubTaskTime: () => _selectTime(index, false),
          onCycleMode: () {
            setState(() => card.clockMode = card.clockMode.next);
            _apiService.saveLocalCards(_cards);
          },
          onMoveUp: () {
            if (index > 0) {
              setState(() {
                final item = _cards.removeAt(index);
                _cards.insert(index - 1, item);
                _selectedCardIndex = index - 1;
              });
              _apiService.saveLocalCards(_cards);
            }
          },
          onMoveDown: () {
            if (index < _cards.length - 1) {
              setState(() {
                final item = _cards.removeAt(index);
                _cards.insert(index + 1, item);
                _selectedCardIndex = index + 1;
              });
              _apiService.saveLocalCards(_cards);
            }
          },
          onSetPosition: () => _openPositionDialog(index),
          onScheduleTime: () => _openScheduleDialog(index),
          onRotationInterval: () => _openIntervalDialog(),
          onToggleHide: () {
            setState(() => card.isHidden = !card.isHidden);
            _apiService.saveLocalCards(_cards);
          },
          onDelete: () {
            setState(() {
              _cards.removeAt(index);
              if (_selectedCardIndex >= _cards.length) {
                _selectedCardIndex = (_cards.length - 1).clamp(0, _cards.length);
              }
            });
            _apiService.saveLocalCards(_cards);
          },
          onOpenNote: () => _openVirtualNoteDialog(card),
          onUpdateTitle: (title) {
            setState(() => card.mainText = title);
            _apiService.saveLocalCards(_cards);
          },
          onUpdateSubText: (sub) {
            setState(() => card.subText = sub);
            _apiService.saveLocalCards(_cards);
          },
        );
      },
    );
  }

  /// Dialog Handlers
  void _openScheduleDialog(int cardIndex) {
    showDialog(
      context: context,
      builder: (ctx) => ScheduleDialog(
        card: _cards[cardIndex],
        onSave: (date, time) {
          setState(() {
            _cards[cardIndex].scheduledDate = date;
            _cards[cardIndex].scheduledTime = time;
          });
          _apiService.saveLocalCards(_cards);
        },
      ),
    );
  }

  void _openPositionDialog(int cardIndex) {
    showDialog(
      context: context,
      builder: (ctx) => PositionDialog(
        currentIndex: cardIndex,
        totalCount: _cards.length,
        onSetPosition: (newIndex) {
          setState(() {
            final item = _cards.removeAt(cardIndex);
            _cards.insert(newIndex, item);
            _selectedCardIndex = newIndex;
          });
          _apiService.saveLocalCards(_cards);
        },
      ),
    );
  }

  void _openIntervalDialog() {
    showDialog(
      context: context,
      builder: (ctx) => IntervalDialog(
        currentInterval: _rotationIntervalSeconds,
        onSave: (newInterval) {
          setState(() => _rotationIntervalSeconds = newInterval);
          _startQuoteRotation();
          _saveSettings();
        },
      ),
    );
  }

  void _openVirtualNoteDialog(TaskCard card) {
    showDialog(
      context: context,
      builder: (ctx) => VirtualNoteScrollerDialog(
        card: card,
        apiService: _apiService,
        onSaveLocalNote: (updatedNote) {
          setState(() => card.liveNote = updatedNote);
          _apiService.saveLocalCards(_cards);
        },
      ),
    );
  }

  void _openThemeModal() {
    showDialog(
      context: context,
      builder: (ctx) => ThemeModal(
        currentTheme: _currentTheme,
        universalColor: _universalFontColor,
        onSelectTheme: (themeId) {
          setState(() => _currentTheme = themeId);
          _saveSettings();
        },
        onSelectColor: (color) {
          setState(() => _universalFontColor = color);
          _saveSettings();
        },
      ),
    );
  }

  void _openProfileModal() {
    showDialog(
      context: context,
      builder: (ctx) => ProfileModal(
        userProfile: _userProfile,
        isBackendOnline: _isAxumOnline,
        onSave: (updated) {
          setState(() => _userProfile = updated);
          _saveSettings();
        },
      ),
    );
  }

  /// Multi-Animation Processor (1:1 with src/main.rs:7964-8050)
  Widget _applyMultiAnimation(Widget child) {
    if (_activeAnimation == 'none') return child;

    final progress = _animController.value;

    switch (_activeAnimation) {
      case 'bounce':
        final bounceY = math.sin(progress * 2 * math.pi) * 16.0;
        return Transform.translate(
          offset: Offset(0, bounceY),
          child: child,
        );

      case 'shake':
        final shakeX = math.sin(progress * 130.0) * 8.0;
        final shakeY = math.cos(progress * 115.0) * 8.0;
        return Transform.translate(
          offset: Offset(shakeX, shakeY),
          child: child,
        );

      case 'dance':
        final danceX = math.sin(progress * 4.0 * math.pi) * 20.0;
        final danceY = math.cos(progress * 2.5 * math.pi) * 15.0;
        return Transform.translate(
          offset: Offset(danceX, danceY),
          child: child,
        );

      case 'rotate':
        return Transform.rotate(
          angle: progress * 2 * math.pi,
          child: child,
        );

      case 'dissolve':
        final opacity = (0.35 + 0.65 * math.cos(progress * 2.5 * math.pi).abs()).clamp(0.2, 1.0);
        return Opacity(
          opacity: opacity,
          child: child,
        );

      case 'fly':
        final flyX = (progress * 2.0 - 1.0) * 100.0;
        final flyY = math.sin(progress * 2.0 * math.pi) * 30.0;
        return Transform.translate(
          offset: Offset(flyX, flyY),
          child: child,
        );

      default:
        return child;
    }
  }

  /// 1:1 Parity Title Bar (src/main.rs:2179-2445)
  PreferredSizeWidget _buildCyberTopBar() {
    return PreferredSize(
      preferredSize: const Size.fromHeight(44),
      child: Container(
        color: CyberTheme.bgPrimary,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        alignment: Alignment.center,
        child: SafeArea(
          bottom: false,
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                // 1. App Icon
                const Text('🚀', style: TextStyle(fontSize: 14)),
                const SizedBox(width: 4),

                // 2. [+] Add Card Button
                InkWell(
                  onTap: () => _addNewCard(),
                  child: Container(
                    padding: const EdgeInsets.all(3),
                    decoration: BoxDecoration(
                      border: Border.all(color: CyberTheme.neonLime, width: 1.0),
                      borderRadius: BorderRadius.circular(3),
                    ),
                    child: const Icon(Icons.add, color: CyberTheme.neonLime, size: 14),
                  ),
                ),
                const SizedBox(width: 6),

                // 3. Title
                const Text(
                  'DAILY MOTIVATION',
                  style: TextStyle(
                    color: CyberTheme.neonCyan,
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 1.0,
                    fontFamily: 'monospace',
                  ),
                ),
                const SizedBox(width: 6),

                // 4. Version Badge (v∞.0)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.08),
                    borderRadius: BorderRadius.circular(3),
                    border: Border.all(color: CyberTheme.neonCyan.withValues(alpha: 0.3), width: 0.5),
                  ),
                  child: const Text(
                    'v∞.0',
                    style: TextStyle(color: CyberTheme.neonCyan, fontSize: 9, fontFamily: 'monospace'),
                  ),
                ),
                const SizedBox(width: 6),

                // 5. [ Index / Total ] Counter
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.black45,
                    borderRadius: BorderRadius.circular(3),
                    border: Border.all(color: CyberTheme.borderSubtle),
                  ),
                  child: Text(
                    '[ ${_cards.isEmpty ? 0 : _selectedCardIndex + 1} / ${_cards.length} ]',
                    style: const TextStyle(
                      color: CyberTheme.neonLime,
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                    ),
                  ),
                ),
                const SizedBox(width: 8),

                // 6. Zoom In (A+) & Zoom Out (A-)
                _buildTopIconBtn(
                  label: 'A+',
                  tooltip: 'Zoom In',
                  color: CyberTheme.neonCyan,
                  onTap: () {
                    setState(() => _zoomLevel = (_zoomLevel + 0.1).clamp(0.7, 2.5));
                    _saveSettings();
                  },
                ),
                const SizedBox(width: 2),
                _buildTopIconBtn(
                  label: 'A-',
                  tooltip: 'Zoom Out',
                  color: CyberTheme.neonCyan,
                  onTap: () {
                    setState(() => _zoomLevel = (_zoomLevel - 0.1).clamp(0.7, 2.5));
                    _saveSettings();
                  },
                ),
                const SizedBox(width: 4),

                // 7. CARD SIZE Button (10% to 300% Popup)
                _buildTopIconBtn(
                  label: '📏',
                  tooltip: 'Card Size (10% to 300%)',
                  color: _isCardSizePopupOpen ? CyberTheme.neonLime : Colors.white,
                  onTap: () => setState(() => _isCardSizePopupOpen = !_isCardSizePopupOpen),
                ),
                const SizedBox(width: 4),

                // 8. Single Quote Mode Toggle (SQ)
                _buildTopIconBtn(
                  label: 'SQ',
                  tooltip: 'Single Quote Mode',
                  color: _singleQuoteMode ? CyberTheme.neonLime : Colors.white,
                  onTap: () {
                    setState(() => _singleQuoteMode = !_singleQuoteMode);
                    _saveSettings();
                  },
                ),
                const SizedBox(width: 4),

                // 8.5. Floating Overlay Widget Button (🪟)
                _buildTopIconBtn(
                  label: '🪟',
                  tooltip: 'Floating Widget / Chat Head',
                  color: OverlayService.instance.isOverlayActive ? CyberTheme.neonLime : CyberTheme.neonCyan,
                  onTap: _toggleFloatingOverlay,
                ),
                const SizedBox(width: 4),

                // 9. Animations Popup Menu
                PopupMenuButton<String>(
                  tooltip: 'Multi-Animation',
                  icon: Icon(
                    Icons.play_circle_fill,
                    color: _activeAnimation != 'none' ? CyberTheme.neonLime : Colors.white,
                    size: 16,
                  ),
                  color: const Color(0xF20A0F1D),
                  onSelected: (val) => setState(() => _activeAnimation = val),
                  itemBuilder: (ctx) => [
                    _buildAnimMenuItem('none', 'None (Stationary)'),
                    _buildAnimMenuItem('bounce', 'Bounce (Playful Bouncing)'),
                    _buildAnimMenuItem('shake', 'Shake (Gentle Shaking)'),
                    _buildAnimMenuItem('dance', 'Dance (Rhythmic Orbital)'),
                    _buildAnimMenuItem('rotate', 'Rotate (Smooth 360 Spin)'),
                    _buildAnimMenuItem('dissolve', 'Dissolve (Fading Pulse)'),
                    _buildAnimMenuItem('fly', 'Fly (Horizontal Flight)'),
                  ],
                ),

                // 10. Theme Settings Button
                _buildTopIconBtn(
                  label: '🎨',
                  tooltip: 'Theme Settings',
                  color: Colors.white,
                  onTap: _openThemeModal,
                ),
                const SizedBox(width: 2),

                // 11. Profile Modal Button
                _buildTopIconBtn(
                  label: '👤',
                  tooltip: 'User Profile',
                  color: Colors.white,
                  onTap: _openProfileModal,
                ),
                const SizedBox(width: 2),

                // 12. Export Quotes Button
                _buildTopIconBtn(
                  label: '💾',
                  tooltip: 'Export Quotes (JSON)',
                  color: Colors.white,
                  onTap: _exportQuotesJson,
                ),
                const SizedBox(width: 2),

                // 13. 3D Background Toggle
                _buildTopIconBtn(
                  label: '🌌',
                  tooltip: 'Toggle 3D Background',
                  color: _is3dBgActive ? CyberTheme.neonCyan : Colors.white,
                  onTap: () => setState(() => _is3dBgActive = !_is3dBgActive),
                ),
                const SizedBox(width: 4),

                // 14. Axum Status Badge
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                  decoration: BoxDecoration(
                    color: _isAxumOnline ? CyberTheme.neonLime.withValues(alpha: 0.15) : Colors.black38,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(
                      color: _isAxumOnline ? CyberTheme.neonLime : Colors.redAccent,
                      width: 0.8,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Container(
                        width: 5,
                        height: 5,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: _isAxumOnline ? CyberTheme.neonLime : Colors.redAccent,
                        ),
                      ),
                      const SizedBox(width: 3),
                      Text(
                        _isAxumOnline ? 'AXUM' : 'OFFLINE',
                        style: TextStyle(
                          color: _isAxumOnline ? CyberTheme.neonLime : Colors.redAccent,
                          fontSize: 9,
                          fontWeight: FontWeight.bold,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ],
                  ),
                ),

                // 15. Drawer open button
                Builder(
                  builder: (context) => IconButton(
                    icon: const Icon(Icons.tune, color: CyberTheme.neonCyan, size: 16),
                    tooltip: 'Control Panel',
                    onPressed: () => Scaffold.of(context).openEndDrawer(),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  PopupMenuItem<String> _buildAnimMenuItem(String key, String title) {
    final active = _activeAnimation == key;
    return PopupMenuItem(
      value: key,
      child: Text(
        title,
        style: TextStyle(
          color: active ? CyberTheme.neonLime : Colors.white,
          fontSize: 11,
          fontFamily: 'monospace',
          fontWeight: active ? FontWeight.bold : FontWeight.normal,
        ),
      ),
    );
  }

  Widget _buildTopIconBtn({
    required String label,
    required String tooltip,
    required Color color,
    required VoidCallback onTap,
  }) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(3),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.05),
            borderRadius: BorderRadius.circular(3),
            border: Border.all(color: color.withValues(alpha: 0.4), width: 0.7),
          ),
          child: Text(
            label,
            style: TextStyle(
              color: color,
              fontSize: 10,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
        ),
      ),
    );
  }

  /// 1:1 Parity Footer (src/main.rs:4124-4208)
  Widget _buildCyberBottomBar() {
    return Container(
      height: 28,
      color: const Color(0xFF06080C),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      alignment: Alignment.center,
      child: Row(
        children: [
          // 1. Navigation Buttons (◀ & ▶)
          InkWell(
            onTap: _prevQuote,
            child: const Padding(
              padding: EdgeInsets.symmetric(horizontal: 4, vertical: 2),
              child: Text(
                '◀',
                style: TextStyle(
                  color: CyberTheme.neonCyan,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ),
          InkWell(
            onTap: _nextQuote,
            child: const Padding(
              padding: EdgeInsets.symmetric(horizontal: 4, vertical: 2),
              child: Text(
                '▶',
                style: TextStyle(
                  color: CyberTheme.neonCyan,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),

          // 2. Technical Readout
          Expanded(
            child: Text(
              '◈ NEURAL FEED ◈ SYN:${_cards.length.toString().padLeft(3, '0')} · FREQ:${_rotationIntervalSeconds * 1000}ms · CORE:∞',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                color: CyberTheme.neonSolar.withValues(alpha: 0.8),
                fontSize: 9,
                fontFamily: 'monospace',
              ),
            ),
          ),

          // 3. Status Indicator Dot
          Container(
            width: 6,
            height: 6,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: _autoRotationEnabled ? CyberTheme.neonLime : Colors.redAccent,
            ),
          ),
          const SizedBox(width: 4),

          // 4. Rotation Status Text
          Text(
            'Δt ${_rotationIntervalSeconds}s · ${_autoRotationEnabled ? 'STREAMING' : 'PAUSED'}',
            style: TextStyle(
              color: _autoRotationEnabled ? CyberTheme.neonLime : CyberTheme.textMuted,
              fontSize: 9,
              fontWeight: FontWeight.bold,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }
}
