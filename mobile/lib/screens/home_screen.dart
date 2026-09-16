import 'dart:async';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../theme/cyber_theme.dart';
import '../models/task_card.dart';
import '../services/api_service.dart';
import '../widgets/task_card_widget.dart';
import '../widgets/control_panel_widget.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final ApiService _apiService = ApiService();
  final List<TaskCard> _cards = [];
  int _selectedCardIndex = 0;
  bool _isAxumOnline = false;
  bool _autoRotationEnabled = true;
  final int _rotationIntervalSeconds = 8;
  Timer? _stopwatchTicker;
  Timer? _quoteRotationTicker;

  @override
  void initState() {
    super.initState();
    _initializeApp();
    _startStopwatchTicker();
    _startQuoteRotation();
  }

  @override
  void dispose() {
    _stopwatchTicker?.cancel();
    _quoteRotationTicker?.cancel();
    super.dispose();
  }

  Future<void> _initializeApp() async {
    await _apiService.loadSavedBaseUrl();
    await _checkBackendStatus();
    await _loadInitialCards();
  }

  Future<void> _checkBackendStatus() async {
    final online = await _apiService.checkHealth();
    if (mounted) {
      setState(() => _isAxumOnline = online);
    }
  }

  Future<void> _loadInitialCards() async {
    // 1. Try local cache first
    final local = await _apiService.loadLocalCards();
    if (local.isNotEmpty) {
      if (mounted) {
        setState(() {
          _cards.clear();
          _cards.addAll(local);
        });
      }
    }

    // 2. Try fetching from Axum API
    if (_isAxumOnline) {
      final remote = await _apiService.fetchCards();
      if (remote.isNotEmpty) {
        if (mounted) {
          setState(() {
            _cards.clear();
            _cards.addAll(remote);
          });
          _apiService.saveLocalCards(_cards);
        }
        return;
      }
    }

    // 3. Fallback default cards if completely empty (matches Rust defaults in screenshot)
    if (_cards.isEmpty) {
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
            subText: "Keep pushing - You're doing great! ✨",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
          TaskCard(
            id: '3',
            mainText: 'Consistency is the key to mastery',
            subText: "Keep pushing - You're doing great! ✨",
            startTime: '12:10 PM',
            endTime: '12:10 PM',
            stopwatchSeconds: 0,
          ),
          TaskCard(
            id: '4',
            mainText: 'Small daily improvements lead to stunning results',
            subText: "Keep pushing - You're doing great! ✨",
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
          setState(() {
            _selectedCardIndex = (_selectedCardIndex + 1) % _cards.length;
          });
        }
      },
    );
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

  @override
  Widget build(BuildContext context) {
    final isWideScreen = MediaQuery.of(context).size.width >= 768;

    return Scaffold(
      backgroundColor: CyberTheme.bgCosmic,
      appBar: _buildCyberTopBar(),
      endDrawer: !isWideScreen
          ? Drawer(
              child: ControlPanelWidget(
                apiService: _apiService,
                isConnected: _isAxumOnline,
                onAddCustomText: (main, aux) => _addNewCard(title: main, subtitle: aux),
                onRefreshServer: _checkBackendStatus,
              ),
            )
          : null,
      body: Column(
        children: [
          // Main Body: Split view on Tablet/Desktop, Full width on Phone
          Expanded(
            child: Row(
              children: [
                // Cards List View
                Expanded(
                  child: _cards.isEmpty
                      ? const Center(
                          child: Text(
                            'No Tasks. Tap [+] to add one.',
                            style: TextStyle(color: CyberTheme.textSecondary),
                          ),
                        )
                      : ListView.builder(
                          padding: const EdgeInsets.symmetric(vertical: 6),
                          itemCount: _cards.length,
                          itemBuilder: (context, index) {
                            final card = _cards[index];
                            return TaskCardWidget(
                              card: card,
                              isSelected: index == _selectedCardIndex,
                              onTap: () => setState(() => _selectedCardIndex = index),
                              onAddSubCard: () => _addSubCard(index),
                              onToggleStopwatch: () {
                                setState(() {
                                  card.isStopwatchRunning = !card.isStopwatchRunning;
                                });
                                _apiService.saveLocalCards(_cards);
                              },
                              onSelectDeadline: () => _selectTime(index, true),
                              onSelectSubTaskTime: () => _selectTime(index, false),
                              onCycleMode: () {
                                setState(() {
                                  card.clockMode = card.clockMode.next;
                                });
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
                              onDelete: () {
                                setState(() {
                                  _cards.removeAt(index);
                                  if (_selectedCardIndex >= _cards.length) {
                                    _selectedCardIndex = (_cards.length - 1).clamp(0, _cards.length);
                                  }
                                });
                                _apiService.saveLocalCards(_cards);
                              },
                              onUpdateTitle: (title) {
                                setState(() => card.mainText = title);
                                _apiService.saveLocalCards(_cards);
                              },
                              onUpdateLiveNote: (note) {
                                card.liveNote = note;
                                _apiService.saveLocalCards(_cards);
                              },
                            );
                          },
                        ),
                ),

                // Right Panel if wide screen
                if (isWideScreen)
                  SizedBox(
                    width: 320,
                    child: ControlPanelWidget(
                      apiService: _apiService,
                      isConnected: _isAxumOnline,
                      onAddCustomText: (main, aux) => _addNewCard(title: main, subtitle: aux),
                      onRefreshServer: _checkBackendStatus,
                    ),
                  ),
              ],
            ),
          ),

          // Cyber Bottom Status Bar
          _buildCyberBottomBar(),
        ],
      ),
    );
  }

  PreferredSizeWidget _buildCyberTopBar() {
    return PreferredSize(
      preferredSize: const Size.fromHeight(42),
      child: Container(
        color: CyberTheme.bgPrimary,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        alignment: Alignment.center,
        child: SafeArea(
          bottom: false,
          child: Row(
            children: [
              // Rocket Icon
              const Text('🚀', style: TextStyle(fontSize: 14)),
              const SizedBox(width: 6),

              // [+] Add Button
              InkWell(
                onTap: () => _addNewCard(),
                child: Container(
                  padding: const EdgeInsets.all(3),
                  decoration: BoxDecoration(
                    border: Border.all(color: CyberTheme.neonLime, width: 1),
                    borderRadius: BorderRadius.circular(3),
                  ),
                  child: const Icon(Icons.add, color: CyberTheme.neonLime, size: 14),
                ),
              ),
              const SizedBox(width: 8),

              // Title
              const Text(
                'DAILY MOTIVATION',
                style: TextStyle(
                  color: CyberTheme.neonCyan,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 1.2,
                  fontFamily: 'monospace',
                ),
              ),
              const SizedBox(width: 6),

              // Version badge
              const Text(
                'v0.0',
                style: TextStyle(color: CyberTheme.textMuted, fontSize: 10, fontFamily: 'monospace'),
              ),
              const SizedBox(width: 8),

              // [Index / Total] badge
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.black45,
                  borderRadius: BorderRadius.circular(3),
                  border: Border.all(color: CyberTheme.borderSubtle),
                ),
                child: Text(
                  '[${_cards.isEmpty ? 0 : _selectedCardIndex + 1} / ${_cards.length}]',
                  style: const TextStyle(
                    color: CyberTheme.neonCyan,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
              ),

              const Spacer(),

              // Auto Rotation Toggle
              IconButton(
                icon: Icon(
                  _autoRotationEnabled ? Icons.sync : Icons.sync_disabled,
                  color: _autoRotationEnabled ? CyberTheme.neonLime : CyberTheme.textMuted,
                  size: 16,
                ),
                tooltip: 'Toggle Auto Quote Rotation',
                onPressed: () {
                  setState(() => _autoRotationEnabled = !_autoRotationEnabled);
                  _startQuoteRotation();
                },
              ),

              // Axum Status Chip
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
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
                      width: 6,
                      height: 6,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: _isAxumOnline ? CyberTheme.neonLime : Colors.redAccent,
                      ),
                    ),
                    const SizedBox(width: 4),
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

              // Drawer toggle button (on mobile)
              Builder(
                builder: (context) => IconButton(
                  icon: const Icon(Icons.tune, color: CyberTheme.neonCyan, size: 16),
                  tooltip: 'Open Controls',
                  onPressed: () => Scaffold.of(context).openEndDrawer(),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCyberBottomBar() {
    return Container(
      height: 24,
      color: const Color(0xFF06080C),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      alignment: Alignment.center,
      child: Row(
        children: [
          const Text(
            '/ NEURAL FEED · SYN010 · FREQ:8000ms · CORE: OK',
            style: TextStyle(
              color: CyberTheme.textMuted,
              fontSize: 9,
              fontFamily: 'monospace',
            ),
          ),
          const Spacer(),
          Container(
            width: 6,
            height: 6,
            decoration: const BoxDecoration(
              shape: BoxShape.circle,
              color: CyberTheme.neonLime,
            ),
          ),
          const SizedBox(width: 4),
          Text(
            'Δt ${_rotationIntervalSeconds}s · STREAMING | AUTO: ${_autoRotationEnabled ? 'ON' : 'OFF'}',
            style: const TextStyle(
              color: CyberTheme.neonLime,
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
