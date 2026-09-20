import 'package:flutter/material.dart';
import '../theme/cyber_theme.dart';
import '../services/api_service.dart';
import '../models/task_card.dart';

/// 1:1 Implementation of Rust LiveNoteViewer (src/virtual_scroller.rs)
/// Virtual Scrolling Component supporting chunked lazy-loading of lines
class VirtualNoteScrollerDialog extends StatefulWidget {
  final TaskCard card;
  final ApiService apiService;
  final Function(String) onSaveLocalNote;

  const VirtualNoteScrollerDialog({
    super.key,
    required this.card,
    required this.apiService,
    required this.onSaveLocalNote,
  });

  @override
  State<VirtualNoteScrollerDialog> createState() =>
      _VirtualNoteScrollerDialogState();
}

class _VirtualNoteScrollerDialogState extends State<VirtualNoteScrollerDialog> {
  final Map<int, String> _loadedLines = {};
  int _totalLines = 0;
  bool _isLoadingMeta = true;
  final Set<int> _fetchingChunks = {};
  final TextEditingController _quickEditController = TextEditingController();
  int? _editingLineNumber;
  bool _isAxumConnected = false;

  @override
  void initState() {
    super.initState();
    _initVirtualScroller();
  }

  @override
  void dispose() {
    _quickEditController.dispose();
    super.dispose();
  }

  Future<void> _initVirtualScroller() async {
    _isAxumConnected = await widget.apiService.checkHealth();

    if (_isAxumConnected) {
      final meta = await widget.apiService.fetchCardMetadata(widget.card.id);
      if (meta != null && mounted) {
        setState(() {
          _totalLines = (meta['total_lines'] as num?)?.toInt() ?? 0;
          _isLoadingMeta = false;
        });
        if (_totalLines > 0) {
          _fetchChunk(1, 50);
          return;
        }
      }
    }

    // Fallback: Populate from local card liveNote lines
    if (mounted) {
      final localLines = widget.card.liveNote.split('\n');
      for (int i = 0; i < localLines.length; i++) {
        _loadedLines[i + 1] = localLines[i];
      }
      setState(() {
        _totalLines = localLines.isNotEmpty && localLines.first.isNotEmpty
            ? localLines.length
            : 0;
        _isLoadingMeta = false;
      });
    }
  }

  Future<void> _fetchChunk(int startLine, int limit) async {
    final chunkKey = startLine ~/ 50;
    if (_fetchingChunks.contains(chunkKey)) return;
    _fetchingChunks.add(chunkKey);

    final entries = await widget.apiService.fetchCardLineEntries(
      widget.card.id,
      startLine: startLine,
      limit: limit,
    );

    if (mounted) {
      setState(() {
        for (var item in entries) {
          final lineNum = item['line_number'] as int;
          final lineText = item['line_text'] as String;
          _loadedLines[lineNum] = lineText;
        }
      });
    }
    _fetchingChunks.remove(chunkKey);
  }

  void _saveLine(int lineNum, String text) {
    setState(() {
      _loadedLines[lineNum] = text;
      _editingLineNumber = null;
    });

    if (_isAxumConnected) {
      widget.apiService.updateLine(widget.card.id, lineNum, text);
    }

    // Sync back to local card liveNote
    final sortedKeys = _loadedLines.keys.toList()..sort();
    final fullText = sortedKeys.map((k) => _loadedLines[k] ?? '').join('\n');
    widget.onSaveLocalNote(fullText);
  }

  void _addNewLine() {
    final nextLineNum = _totalLines + 1;
    setState(() {
      _totalLines = nextLineNum;
      _loadedLines[nextLineNum] = '';
      _editingLineNumber = nextLineNum;
      _quickEditController.text = '';
    });
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 24),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xF70A0F1D),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: CyberTheme.neonCyan, width: 1.5),
          boxShadow: [
            BoxShadow(
              color: CyberTheme.neonCyan.withValues(alpha: 0.25),
              blurRadius: 20,
              spreadRadius: 2,
            ),
          ],
        ),
        child: Column(
          children: [
            // Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: const BoxDecoration(
                color: Color(0xFF0F172A),
                borderRadius: BorderRadius.vertical(top: Radius.circular(15)),
                border: Border(bottom: BorderSide(color: CyberTheme.borderSubtle)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.description, color: CyberTheme.neonCyan, size: 18),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'VIRTUAL LIVE NOTE · CARD #${widget.card.id}',
                          style: const TextStyle(
                            color: CyberTheme.neonCyan,
                            fontSize: 12,
                            fontWeight: FontWeight.bold,
                            fontFamily: 'monospace',
                            letterSpacing: 1.0,
                          ),
                        ),
                        Text(
                          _isLoadingMeta
                              ? 'Connecting to Axum Engine...'
                              : '$_totalLines Lines Total · ${_isAxumConnected ? 'Axum Virtual Stream' : 'Local Memory Mode'}',
                          style: TextStyle(
                            color: _isAxumConnected ? CyberTheme.neonLime : CyberTheme.textSecondary,
                            fontSize: 10,
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.add, color: CyberTheme.neonLime, size: 18),
                    tooltip: 'Append Line',
                    onPressed: _addNewLine,
                  ),
                  IconButton(
                    icon: const Icon(Icons.close, color: Colors.white, size: 18),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
            ),

            // Virtual Scroller Body
            Expanded(
              child: _isLoadingMeta
                  ? const Center(
                      child: CircularProgressIndicator(color: CyberTheme.neonCyan),
                    )
                  : _totalLines == 0
                      ? Center(
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              const Icon(Icons.notes, color: CyberTheme.textMuted, size: 40),
                              const SizedBox(height: 8),
                              const Text(
                                'No note lines found.',
                                style: TextStyle(color: CyberTheme.textSecondary, fontFamily: 'monospace'),
                              ),
                              const SizedBox(height: 12),
                              ElevatedButton.icon(
                                icon: const Icon(Icons.add, size: 14),
                                label: const Text('Add First Line', style: TextStyle(fontFamily: 'monospace')),
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: CyberTheme.bgCardActive,
                                  foregroundColor: CyberTheme.neonCyan,
                                  side: const BorderSide(color: CyberTheme.neonCyan),
                                ),
                                onPressed: _addNewLine,
                              ),
                            ],
                          ),
                        )
                      : ListView.builder(
                          itemCount: _totalLines,
                          itemBuilder: (context, index) {
                            final lineNum = index + 1;
                            if (!_loadedLines.containsKey(lineNum) && _isAxumConnected) {
                              final chunkStart = ((lineNum - 1) ~/ 50) * 50 + 1;
                              _fetchChunk(chunkStart, 50);
                            }

                            final text = _loadedLines[lineNum] ?? '...';
                            final isEditing = _editingLineNumber == lineNum;

                            return Container(
                              decoration: const BoxDecoration(
                                border: Border(
                                  bottom: BorderSide(
                                    color: Color(0x1A30363D),
                                    width: 0.5,
                                  ),
                                ),
                              ),
                              child: isEditing
                                  ? Padding(
                                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                      child: Row(
                                        children: [
                                          Text(
                                            lineNum.toString().padLeft(4, '0'),
                                            style: const TextStyle(
                                              color: CyberTheme.neonLime,
                                              fontSize: 11,
                                              fontFamily: 'monospace',
                                            ),
                                          ),
                                          const SizedBox(width: 8),
                                          Expanded(
                                            child: TextField(
                                              controller: _quickEditController,
                                              autofocus: true,
                                              style: const TextStyle(
                                                color: Colors.white,
                                                fontSize: 12,
                                                fontFamily: 'monospace',
                                              ),
                                              decoration: const InputDecoration(
                                                isDense: true,
                                                border: OutlineInputBorder(
                                                  borderSide: BorderSide(color: CyberTheme.neonCyan),
                                                ),
                                              ),
                                              onSubmitted: (val) => _saveLine(lineNum, val),
                                            ),
                                          ),
                                          IconButton(
                                            icon: const Icon(Icons.check, color: CyberTheme.neonLime, size: 16),
                                            onPressed: () => _saveLine(lineNum, _quickEditController.text),
                                          ),
                                        ],
                                      ),
                                    )
                                  : InkWell(
                                      onTap: () {
                                        setState(() {
                                          _editingLineNumber = lineNum;
                                          _quickEditController.text = text;
                                        });
                                      },
                                      child: Padding(
                                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                                        child: Row(
                                          crossAxisAlignment: CrossAxisAlignment.start,
                                          children: [
                                            // Line number column (monospace)
                                            Container(
                                              width: 38,
                                              alignment: Alignment.centerRight,
                                              child: Text(
                                                lineNum.toString().padLeft(4, '0'),
                                                style: const TextStyle(
                                                  color: Color(0xFF6E7681),
                                                  fontSize: 11,
                                                  fontFamily: 'monospace',
                                                ),
                                              ),
                                            ),
                                            const SizedBox(width: 12),
                                            // Line text
                                            Expanded(
                                              child: Text(
                                                text.isEmpty ? ' ' : text,
                                                style: const TextStyle(
                                                  color: CyberTheme.textPrimary,
                                                  fontSize: 12,
                                                  fontFamily: 'monospace',
                                                  height: 1.3,
                                                ),
                                              ),
                                            ),
                                          ],
                                        ),
                                      ),
                                    ),
                            );
                          },
                        ),
            ),

            // Footer info
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: const BoxDecoration(
                color: Color(0xFF090D16),
                borderRadius: BorderRadius.vertical(bottom: Radius.circular(15)),
                border: Border(top: BorderSide(color: CyberTheme.borderSubtle)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.touch_app, color: CyberTheme.textMuted, size: 12),
                  const SizedBox(width: 4),
                  const Text(
                    'Tap any line to edit inline · Single line updates',
                    style: TextStyle(color: CyberTheme.textMuted, fontSize: 10, fontFamily: 'monospace'),
                  ),
                  const Spacer(),
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(),
                    child: const Text('DONE', style: TextStyle(color: CyberTheme.neonCyan, fontSize: 11)),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
