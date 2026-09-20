import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../services/api_service.dart';
import '../models/task_card.dart';

/// 1:1 Implementation of Rust render_control_panel_contents (src/main.rs:4645-5100)
class ControlPanelWidget extends StatefulWidget {
  final ApiService apiService;
  final bool isConnected;
  final List<TaskCard> cards;
  final Color universalFontColor;
  final double mainLineGap;
  final double subLineGap;
  final double betweenGap;
  final int rotationInterval;
  final bool isAutoRotationEnabled;

  final Function(String mainText, String subText) onAddCustomText;
  final VoidCallback onRefreshServer;
  final Function(Color color) onUniversalColorChanged;
  final Function(double mainGap, double subGap, double betweenGap) onLineGapsChanged;
  final Function(int interval) onIntervalChanged;
  final VoidCallback onToggleRotation;
  final Function(int oldIndex, int newIndex) onReorderCards;
  final Function(int index) onDeleteCard;
  final Function(int index) onToggleHideCard;
  final VoidCallback onOpenVirtualScroller;

  const ControlPanelWidget({
    super.key,
    required this.apiService,
    required this.isConnected,
    required this.cards,
    required this.universalFontColor,
    required this.mainLineGap,
    required this.subLineGap,
    required this.betweenGap,
    required this.rotationInterval,
    required this.isAutoRotationEnabled,
    required this.onAddCustomText,
    required this.onRefreshServer,
    required this.onUniversalColorChanged,
    required this.onLineGapsChanged,
    required this.onIntervalChanged,
    required this.onToggleRotation,
    required this.onReorderCards,
    required this.onDeleteCard,
    required this.onToggleHideCard,
    required this.onOpenVirtualScroller,
  });

  @override
  State<ControlPanelWidget> createState() => _ControlPanelWidgetState();
}

class _ControlPanelWidgetState extends State<ControlPanelWidget> {
  bool _alwaysOnTop = false;
  bool _omniDrag = true;
  bool _hoverToEdit = false;

  final TextEditingController _mainStreamController = TextEditingController();
  final TextEditingController _auxStreamController = TextEditingController();
  final TextEditingController _serverUrlController = TextEditingController();

  late double _mainGap;
  late double _subGap;
  late double _betweenGap;
  late double _intervalVal;

  @override
  void initState() {
    super.initState();
    _serverUrlController.text = widget.apiService.baseUrl;
    _mainGap = widget.mainLineGap;
    _subGap = widget.subLineGap;
    _betweenGap = widget.betweenGap;
    _intervalVal = widget.rotationInterval.toDouble().clamp(1.0, 120.0);
  }

  @override
  void dispose() {
    _mainStreamController.dispose();
    _auxStreamController.dispose();
    _serverUrlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: CyberTheme.bgPanel,
      child: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header
              const Row(
                children: [
                  Icon(Icons.tune, color: CyberTheme.neonCyan, size: 18),
                  SizedBox(width: 8),
                  Text(
                    'CONTROL PANEL',
                    style: TextStyle(
                      color: CyberTheme.neonCyan,
                      fontSize: 14,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                      letterSpacing: 1.2,
                    ),
                  ),
                ],
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 20),

              // 1. AXUM CONNECTION
              _buildSectionTitle('AXUM API CONNECTION'),
              Row(
                children: [
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: widget.isConnected ? CyberTheme.neonLime : Colors.redAccent,
                    ),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    widget.isConnected ? 'AXUM BACKEND ONLINE' : 'LOCAL CACHE MODE',
                    style: TextStyle(
                      color: widget.isConnected ? CyberTheme.neonLime : Colors.redAccent,
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                    ),
                  ),
                  const Spacer(),
                  IconButton(
                    icon: const Icon(Icons.refresh, color: CyberTheme.neonCyan, size: 16),
                    onPressed: widget.onRefreshServer,
                  ),
                ],
              ),
              const SizedBox(height: 4),
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _serverUrlController,
                      style: const TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace'),
                      decoration: InputDecoration(
                        isDense: true,
                        filled: true,
                        fillColor: Colors.black45,
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(4)),
                      ),
                    ),
                  ),
                  const SizedBox(width: 6),
                  ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: CyberTheme.bgCardActive,
                      foregroundColor: CyberTheme.neonCyan,
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                    onPressed: () {
                      widget.apiService.setBaseUrl(_serverUrlController.text.trim());
                      widget.onRefreshServer();
                    },
                    child: const Text('Save', style: TextStyle(fontSize: 11)),
                  ),
                ],
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 2. UNIVERSAL FONT COLOR
              _buildSectionTitle('UNIVERSAL FONT COLOR'),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text('Select Theme Color:', style: TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace')),
                  Row(
                    children: [
                      _buildColorDot(CyberTheme.neonCyan),
                      _buildColorDot(CyberTheme.neonLime),
                      _buildColorDot(CyberTheme.neonSolar),
                      _buildColorDot(Colors.white),
                    ],
                  ),
                ],
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 3. WINDOW BEHAVIOR (src/main.rs:4729)
              _buildSectionTitle('WINDOW BEHAVIOR'),
              CheckboxListTile(
                dense: true,
                contentPadding: EdgeInsets.zero,
                activeColor: CyberTheme.neonCyan,
                title: const Text('Always on Top / Prominent', style: TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace')),
                value: _alwaysOnTop,
                onChanged: (val) => setState(() => _alwaysOnTop = val ?? false),
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: EdgeInsets.zero,
                activeColor: CyberTheme.neonCyan,
                title: const Text('Omni-Drag (Hold Anywhere)', style: TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace')),
                value: _omniDrag,
                onChanged: (val) => setState(() => _omniDrag = val ?? true),
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: EdgeInsets.zero,
                activeColor: CyberTheme.neonCyan,
                title: const Text('Hover / Touch to Edit', style: TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace')),
                value: _hoverToEdit,
                onChanged: (val) => setState(() => _hoverToEdit = val ?? false),
              ),
              const SizedBox(height: 6),
              ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF16243A),
                  foregroundColor: CyberTheme.neonCyan,
                  side: const BorderSide(color: CyberTheme.neonCyan),
                  minimumSize: const Size.fromHeight(36),
                ),
                icon: const Icon(Icons.description, size: 16),
                label: const Text('📜 Open Virtual Scroller (Massive Text)', style: TextStyle(fontFamily: 'monospace', fontSize: 11)),
                onPressed: widget.onOpenVirtualScroller,
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 4. ADD CUSTOM TEXT [n] (src/main.rs:4761)
              _buildSectionTitle('ADD CUSTOM TEXT [${widget.cards.length + 1}]'),
              const Text('MAIN DATA STREAM', style: TextStyle(color: CyberTheme.neonCyan, fontSize: 10, fontWeight: FontWeight.bold, fontFamily: 'monospace')),
              const SizedBox(height: 4),
              TextField(
                controller: _mainStreamController,
                maxLines: 2,
                style: const TextStyle(color: Colors.white, fontSize: 12, fontFamily: 'monospace'),
                decoration: InputDecoration(
                  hintText: 'Enter motivational quote...',
                  hintStyle: const TextStyle(color: CyberTheme.textMuted, fontSize: 11),
                  filled: true,
                  fillColor: Colors.black45,
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
                ),
              ),
              const SizedBox(height: 8),
              const Text('AUXILIARY STREAM', style: TextStyle(color: CyberTheme.neonCyan, fontSize: 10, fontWeight: FontWeight.bold, fontFamily: 'monospace')),
              const SizedBox(height: 4),
              TextField(
                controller: _auxStreamController,
                maxLines: 1,
                style: const TextStyle(color: Colors.white, fontSize: 11, fontFamily: 'monospace'),
                decoration: InputDecoration(
                  hintText: 'Enter subtitle / note...',
                  hintStyle: const TextStyle(color: CyberTheme.textMuted, fontSize: 10),
                  filled: true,
                  fillColor: Colors.black45,
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(6)),
                ),
              ),
              const SizedBox(height: 10),
              ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: CyberTheme.bgCardActive,
                  foregroundColor: CyberTheme.neonLime,
                  side: const BorderSide(color: CyberTheme.neonLime),
                  minimumSize: const Size.fromHeight(38),
                ),
                icon: const Icon(Icons.add, size: 16),
                label: const Text('ADD TO NEURAL BUFFER', style: TextStyle(fontFamily: 'monospace', fontWeight: FontWeight.bold, fontSize: 11)),
                onPressed: () {
                  final main = _mainStreamController.text.trim();
                  final aux = _auxStreamController.text.trim();
                  if (main.isNotEmpty) {
                    widget.onAddCustomText(main, aux);
                    _mainStreamController.clear();
                    _auxStreamController.clear();
                  }
                },
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 5. LINE GAPS (src/main.rs:4713)
              _buildSectionTitle('LINE GAPS'),
              _buildSliderRow('Main Gap', _mainGap, 0.0, 30.0, (val) {
                setState(() => _mainGap = val);
                widget.onLineGapsChanged(_mainGap, _subGap, _betweenGap);
              }),
              _buildSliderRow('Sub Gap', _subGap, 0.0, 20.0, (val) {
                setState(() => _subGap = val);
                widget.onLineGapsChanged(_mainGap, _subGap, _betweenGap);
              }),
              _buildSliderRow('Between Gap', _betweenGap, 0.0, 30.0, (val) {
                setState(() => _betweenGap = val);
                widget.onLineGapsChanged(_mainGap, _subGap, _betweenGap);
              }),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 6. INTERVAL (SECONDS) (src/main.rs:4854)
              _buildSectionTitle('INTERVAL (SECONDS)'),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('${_intervalVal.toInt()} Seconds', style: const TextStyle(color: CyberTheme.neonCyan, fontSize: 14, fontWeight: FontWeight.bold, fontFamily: 'monospace')),
                  ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: widget.isAutoRotationEnabled ? Colors.amber.shade900 : CyberTheme.neonLime.withValues(alpha: 0.2),
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                    ),
                    onPressed: widget.onToggleRotation,
                    child: Text(widget.isAutoRotationEnabled ? 'PAUSE' : 'RESUME', style: const TextStyle(fontSize: 10, fontFamily: 'monospace')),
                  ),
                ],
              ),
              Slider(
                value: _intervalVal,
                min: 1.0,
                max: 120.0,
                activeColor: CyberTheme.neonCyan,
                onChanged: (val) {
                  setState(() => _intervalVal = val);
                  widget.onIntervalChanged(val.toInt());
                },
              ),
              const Divider(color: CyberTheme.borderSubtle, height: 24),

              // 7. TEXT LIST (n) (src/main.rs:4961)
              _buildSectionTitle('TEXT LIST (${widget.cards.length})'),
              ReorderableListView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: widget.cards.length,
                // ignore: deprecated_member_use
                onReorder: widget.onReorderCards,
                itemBuilder: (context, idx) {
                  final card = widget.cards[idx];
                  return Container(
                    key: ValueKey('drawer_card_${card.id}_$idx'),
                    margin: const EdgeInsets.symmetric(vertical: 2),
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: Colors.black38,
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: CyberTheme.borderSubtle),
                    ),
                    child: Row(
                      children: [
                        Text('${idx + 1}.', style: const TextStyle(color: CyberTheme.textMuted, fontSize: 10, fontFamily: 'monospace')),
                        const SizedBox(width: 6),
                        Expanded(
                          child: Text(
                            card.mainText,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(
                              color: card.isHidden ? CyberTheme.textMuted : Colors.white,
                              fontSize: 11,
                              fontFamily: 'monospace',
                            ),
                          ),
                        ),
                        IconButton(
                          icon: Icon(card.isHidden ? Icons.visibility_off : Icons.visibility, color: Colors.grey, size: 14),
                          onPressed: () => widget.onToggleHideCard(idx),
                        ),
                        IconButton(
                          icon: const Icon(Icons.delete, color: Colors.redAccent, size: 14),
                          onPressed: () => widget.onDeleteCard(idx),
                        ),
                      ],
                    ),
                  );
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title,
        style: const TextStyle(
          color: CyberTheme.textSecondary,
          fontSize: 11,
          fontWeight: FontWeight.bold,
          fontFamily: 'monospace',
          letterSpacing: 0.8,
        ),
      ),
    );
  }

  Widget _buildColorDot(Color color) {
    final isSelected = widget.universalFontColor.toARGB32() == color.toARGB32();
    return InkWell(
      onTap: () => widget.onUniversalColorChanged(color),
      child: Container(
        width: 20,
        height: 20,
        margin: const EdgeInsets.only(left: 6),
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: color,
          border: Border.all(color: isSelected ? Colors.white : Colors.transparent, width: 2),
        ),
      ),
    );
  }

  Widget _buildSliderRow(String label, double value, double min, double max, Function(double) onChanged) {
    return Row(
      children: [
        SizedBox(width: 80, child: Text(label, style: const TextStyle(color: Colors.white, fontSize: 10, fontFamily: 'monospace'))),
        Expanded(
          child: Slider(
            value: value.clamp(min, max),
            min: min,
            max: max,
            activeColor: CyberTheme.neonCyan,
            onChanged: onChanged,
          ),
        ),
        Text('${value.toInt()}px', style: const TextStyle(color: CyberTheme.neonCyan, fontSize: 10, fontFamily: 'monospace')),
      ],
    );
  }
}
