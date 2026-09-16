import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../services/api_service.dart';

class ControlPanelWidget extends StatefulWidget {
  final ApiService apiService;
  final bool isConnected;
  final Function(String mainText, String subText) onAddCustomText;
  final VoidCallback onRefreshServer;

  const ControlPanelWidget({
    super.key,
    required this.apiService,
    required this.isConnected,
    required this.onAddCustomText,
    required this.onRefreshServer,
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

  double _mainFontSize = 13.0;
  double _auxFontSize = 11.0;

  @override
  void initState() {
    super.initState();
    _serverUrlController.text = widget.apiService.baseUrl;
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
      padding: const EdgeInsets.all(12),
      color: CyberTheme.bgPanel,
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // API Server Connection
            _buildSectionHeader('AXUM API CONNECTION'),
            const SizedBox(height: 6),
            Row(
              children: [
                Container(
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: widget.isConnected ? CyberTheme.neonLime : Colors.redAccent,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  widget.isConnected ? 'CONNECTED TO AXUM' : 'OFFLINE / LOCAL ONLY',
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
                  tooltip: 'Reconnect to Axum',
                ),
              ],
            ),
            const SizedBox(height: 4),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _serverUrlController,
                    style: const TextStyle(
                      color: CyberTheme.textPrimary,
                      fontSize: 11,
                      fontFamily: 'monospace',
                    ),
                    decoration: InputDecoration(
                      isDense: true,
                      hintText: 'http://localhost:3000',
                      hintStyle: const TextStyle(color: CyberTheme.textMuted, fontSize: 11),
                      filled: true,
                      fillColor: Colors.black45,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(4),
                        borderSide: const BorderSide(color: CyberTheme.borderSubtle),
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 6),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: CyberTheme.bgCardActive,
                    foregroundColor: CyberTheme.neonCyan,
                    side: const BorderSide(color: CyberTheme.neonCyan, width: 0.8),
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
                  ),
                  onPressed: () {
                    widget.apiService.setBaseUrl(_serverUrlController.text.trim());
                    widget.onRefreshServer();
                  },
                  child: const Text('Save', style: TextStyle(fontSize: 10)),
                ),
              ],
            ),
            const Divider(height: 24, color: CyberTheme.borderSubtle),

            // Universal Font Color
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'Universal Font Color:',
                  style: TextStyle(
                    color: CyberTheme.textPrimary,
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                ),
                Container(
                  width: 18,
                  height: 18,
                  decoration: BoxDecoration(
                    color: Colors.white,
                    border: Border.all(color: CyberTheme.borderSubtle),
                    borderRadius: BorderRadius.circular(3),
                  ),
                ),
              ],
            ),
            const Divider(height: 20, color: CyberTheme.borderSubtle),

            // Window Behavior
            _buildSectionHeader('WINDOW BEHAVIOR'),
            _buildCheckbox('Always on Top (Overlap Taskbar)', _alwaysOnTop, (v) {
              setState(() => _alwaysOnTop = v ?? false);
            }),
            _buildCheckbox('Omni-Drag (Hold Anywhere)', _omniDrag, (v) {
              setState(() => _omniDrag = v ?? false);
            }),
            _buildCheckbox('Hover-to-Edit (Hover Text to Edit)', _hoverToEdit, (v) {
              setState(() => _hoverToEdit = v ?? false);
            }),
            const SizedBox(height: 8),

            // Open Virtual Scroller Button
            OutlinedButton(
              style: OutlinedButton.styleFrom(
                foregroundColor: CyberTheme.textPrimary,
                side: const BorderSide(color: CyberTheme.borderSubtle),
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
              ),
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Virtual Scroller active (streaming lines from Axum)'),
                    backgroundColor: CyberTheme.bgCardActive,
                  ),
                );
              },
              child: const Row(
                children: [
                  Text('? ', style: TextStyle(color: CyberTheme.neonCyan, fontSize: 11)),
                  Expanded(
                    child: Text(
                      'Open Virtual Scroller (Massive Text)',
                      style: TextStyle(fontSize: 10, fontFamily: 'monospace'),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
            const Divider(height: 24, color: CyberTheme.borderSubtle),

            // ADD CUSTOM TEXT
            _buildSectionHeader('ADD CUSTOM TEXT'),
            const SizedBox(height: 6),

            // MAIN DATA STREAM
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'MAIN DATA STREAM',
                  style: TextStyle(
                    color: CyberTheme.neonCyan,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
                Row(
                  children: [
                    _buildTextSizeBtn('A+', () => setState(() => _mainFontSize += 1)),
                    _buildTextSizeBtn('A-', () => setState(() => _mainFontSize = (_mainFontSize - 1).clamp(9.0, 20.0))),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 4),
            Container(
              height: 70,
              decoration: BoxDecoration(
                color: Colors.black45,
                border: Border.all(color: CyberTheme.borderSubtle),
                borderRadius: BorderRadius.circular(4),
              ),
              child: TextField(
                controller: _mainStreamController,
                maxLines: null,
                style: TextStyle(
                  color: CyberTheme.textPrimary,
                  fontSize: _mainFontSize,
                  fontFamily: 'monospace',
                ),
                decoration: const InputDecoration(
                  hintText: '// INPUT MAIN TEXT DATA...',
                  hintStyle: TextStyle(color: CyberTheme.textMuted, fontSize: 10),
                  contentPadding: EdgeInsets.all(8),
                  border: InputBorder.none,
                ),
              ),
            ),
            const SizedBox(height: 8),

            // AUXILIARY DATA STREAM
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'AUXILIARY DATA STREAM',
                  style: TextStyle(
                    color: CyberTheme.neonLime,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
                Row(
                  children: [
                    _buildTextSizeBtn('A+', () => setState(() => _auxFontSize += 1)),
                    _buildTextSizeBtn('A-', () => setState(() => _auxFontSize = (_auxFontSize - 1).clamp(8.0, 18.0))),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 4),
            Container(
              height: 60,
              decoration: BoxDecoration(
                color: Colors.black45,
                border: Border.all(color: CyberTheme.borderSubtle),
                borderRadius: BorderRadius.circular(4),
              ),
              child: TextField(
                controller: _auxStreamController,
                maxLines: null,
                style: TextStyle(
                  color: CyberTheme.textPrimary,
                  fontSize: _auxFontSize,
                  fontFamily: 'monospace',
                ),
                decoration: const InputDecoration(
                  hintText: '// INPUT AUXILIARY TEXT DATA...',
                  hintStyle: TextStyle(color: CyberTheme.textMuted, fontSize: 10),
                  contentPadding: EdgeInsets.all(8),
                  border: InputBorder.none,
                ),
              ),
            ),
            const SizedBox(height: 10),

            // + Add/Apply Text Button
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: CyberTheme.bgCardActive,
                  foregroundColor: CyberTheme.neonLime,
                  side: const BorderSide(color: CyberTheme.greenBorder, width: 1.0),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
                  padding: const EdgeInsets.symmetric(vertical: 8),
                ),
                onPressed: () {
                  final main = _mainStreamController.text.trim();
                  final aux = _auxStreamController.text.trim();
                  if (main.isNotEmpty) {
                    widget.onAddCustomText(
                      main,
                      aux.isNotEmpty ? aux : "Keep pushing - You're doing great! ✨",
                    );
                    _mainStreamController.clear();
                    _auxStreamController.clear();
                  }
                },
                child: const Text(
                  '+ Add/Apply Text',
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Row(
      children: [
        Container(
          width: 3,
          height: 11,
          color: CyberTheme.neonCyan,
        ),
        const SizedBox(width: 5),
        Text(
          title,
          style: const TextStyle(
            color: CyberTheme.neonCyan,
            fontSize: 10,
            fontWeight: FontWeight.bold,
            letterSpacing: 1.0,
            fontFamily: 'monospace',
          ),
        ),
      ],
    );
  }

  Widget _buildCheckbox(String label, bool value, ValueChanged<bool?> onChanged) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 20,
            height: 20,
            child: Checkbox(
              value: value,
              onChanged: onChanged,
              activeColor: CyberTheme.neonCyan,
              checkColor: Colors.black,
              side: const BorderSide(color: CyberTheme.borderSubtle),
            ),
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              label,
              style: const TextStyle(
                color: CyberTheme.textPrimary,
                fontSize: 10,
                fontFamily: 'monospace',
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTextSizeBtn(String text, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
        child: Text(
          text,
          style: const TextStyle(
            color: CyberTheme.textSecondary,
            fontSize: 10,
            fontFamily: 'monospace',
          ),
        ),
      ),
    );
  }
}
