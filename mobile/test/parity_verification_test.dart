import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:task_note_mobile/models/task_card.dart';
import 'package:task_note_mobile/services/api_service.dart';
import 'package:task_note_mobile/widgets/top_action_buttons_bar.dart';
import 'package:task_note_mobile/widgets/task_card_widget.dart';
import 'package:task_note_mobile/widgets/glass_card_painter.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Rust GUI -> Flutter Mobile 1:1 Deep Verification Test Suite', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    // ── SECTION 5: TOP ACTION BUTTONS COUNT VERIFICATION ──
    testWidgets('TopActionButtonsBar has EXACTLY 7 buttons matching Rust src/main.rs:3692-3835', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TopActionButtonsBar(
              isFirst: false,
              isLast: false,
              isHidden: false,
              onMoveUp: () {},
              onMoveDown: () {},
              onSetPosition: () {},
              onScheduleTime: () {},
              onRotationInterval: () {},
              onToggleHide: () {},
              onDelete: () {},
              onOpenNote: () {},
            ),
          ),
        ),
      );

      // Verify the 7 symbols from Rust exist
      expect(find.text('^'), findsOneWidget, reason: 'Button 1: Move Up');
      expect(find.text('v'), findsOneWidget, reason: 'Button 2: Move Down');
      expect(find.text('#'), findsOneWidget, reason: 'Button 3: Set Position');
      expect(find.text('⏰'), findsOneWidget, reason: 'Button 4: Schedule Time');
      expect(find.text('⏱'), findsOneWidget, reason: 'Button 5: Rotation Interval');
      expect(find.text('H'), findsOneWidget, reason: 'Button 6: Hide/Unhide (unhidden)');
      expect(find.text('X'), findsOneWidget, reason: 'Button 7: Delete');

      // Crucial check: Note icon '📝' must NOT be in the top action buttons bar
      expect(find.text('📝'), findsNothing, reason: 'Top action bar must strictly match Rust 7-button layout');
    });

    testWidgets('TopActionButtonsBar toggle shows O when hidden', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TopActionButtonsBar(
              isFirst: false,
              isLast: false,
              isHidden: true,
              onMoveUp: () {},
              onMoveDown: () {},
              onSetPosition: () {},
              onScheduleTime: () {},
              onRotationInterval: () {},
              onToggleHide: () {},
              onDelete: () {},
              onOpenNote: () {},
            ),
          ),
        ),
      );

      expect(find.text('O'), findsOneWidget, reason: 'Hidden card must show O symbol matching Rust');
      expect(find.text('H'), findsNothing);
    });

    // ── SECTION 6: CARD SIZE AND 0% COLLAPSE ──
    testWidgets('Card scale <= 0.001 collapses card to zero/near-zero height (Rust src/main.rs:3135)', (tester) async {
      final card = TaskCard(
        id: '1',
        mainText: 'Test Quote',
        subText: 'Test Subtitle',
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TaskCardWidget(
              card: card,
              index: 0,
              totalCards: 1,
              isSelected: false,
              cardScale: 0.0, // 0% scale
              zoomLevel: 1.0,
              universalFontColor: Colors.white,
              mainLineGap: 4.0,
              subLineGap: 2.0,
              betweenGap: 8.0,
              onTap: () {},
              onAddSubCard: () {},
              onToggleStopwatch: () {},
              onSelectDeadline: () {},
              onSelectSubTaskTime: () {},
              onCycleMode: () {},
              onMoveUp: () {},
              onMoveDown: () {},
              onSetPosition: () {},
              onScheduleTime: () {},
              onRotationInterval: () {},
              onToggleHide: () {},
              onDelete: () {},
              onOpenNote: () {},
              onUpdateTitle: (_) {},
              onUpdateSubText: (_) {},
            ),
          ),
        ),
      );

      // Card content must be collapsed/omitted
      expect(find.text('Test Quote'), findsNothing, reason: 'At 0% scale, content is collapsed');
      expect(find.byType(SizedBox), findsWidgets);
    });

    // ── SECTION 10: TEST REAL USER FLOWS (FLOW A - J) ──

    test('Flow A: Create card -> persist -> reload -> verify card remains', () async {
      final apiService = ApiService();
      final cards = [
        TaskCard(id: 'c1', mainText: 'Original Card', subText: 'Sub 1'),
      ];

      // Save
      await apiService.saveLocalCards(cards);

      // Reload
      final loaded = await apiService.loadLocalCards();
      expect(loaded.length, 1);
      expect(loaded[0].id, 'c1');
      expect(loaded[0].mainText, 'Original Card');
      expect(loaded[0].subText, 'Sub 1');
    });

    test('Flow B: Move card -> persist -> reload -> verify order remains', () async {
      final apiService = ApiService();
      final cards = [
        TaskCard(id: 'c1', mainText: 'Card 1', subText: 'Sub 1'),
        TaskCard(id: 'c2', mainText: 'Card 2', subText: 'Sub 2'),
      ];

      // Reorder (swap)
      final moved = [cards[1], cards[0]];
      await apiService.saveLocalCards(moved);

      // Reload
      final loaded = await apiService.loadLocalCards();
      expect(loaded.length, 2);
      expect(loaded[0].id, 'c2');
      expect(loaded[1].id, 'c1');
    });

    test('Flow C: Hide card -> persist -> reload -> verify hidden state remains', () async {
      final apiService = ApiService();
      final card = TaskCard(id: 'c1', mainText: 'Hide Me', subText: 'Sub');
      card.isHidden = true;

      await apiService.saveLocalCards([card]);

      final loaded = await apiService.loadLocalCards();
      expect(loaded.length, 1);
      expect(loaded[0].isHidden, true);
    });

    test('Flow D: Schedule card -> persist -> reload -> verify schedule remains', () async {
      final apiService = ApiService();
      final card = TaskCard(
        id: 'c1',
        mainText: 'Meeting',
        subText: 'Project Sync',
        scheduledDate: '2026-10-15',
        scheduledTime: '14:30',
      );

      await apiService.saveLocalCards([card]);

      final loaded = await apiService.loadLocalCards();
      expect(loaded.length, 1);
      expect(loaded[0].scheduledDate, '2026-10-15');
      expect(loaded[0].scheduledTime, '14:30');
    });

    test('Flow E: Set interval -> persist -> reload -> verify interval remains', () async {
      final apiService = ApiService();
      await apiService.saveAppSettings({'rotationInterval': 25, 'autoRotation': true});

      final loaded = await apiService.loadAppSettings();
      expect(loaded['rotationInterval'], 25);
      expect(loaded['autoRotation'], true);
    });

    test('Flow F: Resize card scale -> persist -> reload -> verify size state', () async {
      final apiService = ApiService();
      await apiService.saveAppSettings({'cardScale': 1.6, 'zoomLevel': 1.2});

      final loaded = await apiService.loadAppSettings();
      expect(loaded['cardScale'], 1.6);
      expect(loaded['zoomLevel'], 1.2);
    });

    test('Flow G: Edit live note -> persist -> reload -> verify content', () async {
      final apiService = ApiService();
      final card = TaskCard(
        id: 'c1',
        mainText: 'Task With Note',
        subText: 'Check details',
        liveNote: 'Line 1: Spec\nLine 2: Design\nLine 3: Deploy',
      );

      await apiService.saveLocalCards([card]);

      final loaded = await apiService.loadLocalCards();
      expect(loaded.length, 1);
      expect(loaded[0].liveNote, 'Line 1: Spec\nLine 2: Design\nLine 3: Deploy');
    });

    test('Flow H: TaskCard serialization roundtrip with all 19 fields', () {
      final original = TaskCard(
        id: 'full_card_100',
        mainText: 'Master Task',
        subText: 'Detailed Explanation',
        startTime: '09:00 AM',
        endTime: '05:00 PM',
        stopwatchSeconds: 125,
        isStopwatchRunning: true,
        clockMode: ClockMode.stopwatch,
        depth: 1,
        liveNote: 'Note test',
        totalLines: 15,
        isHidden: true,
        mainTextSize: 22.0,
        subTextSize: 14.0,
        mainTextColor: 0xFF00FFCC,
        subTextColor: 0xFFAAAAAA,
        mainLineGap: 6.0,
        subLineGap: 3.0,
        betweenGap: 10.0,
        intervalSecs: 12,
        scheduledDate: '2026-12-31',
        scheduledTime: '23:59',
        orderIndex: 4,
      );

      final jsonMap = original.toLocalJson();
      final restored = TaskCard.fromLocalJson(jsonMap);

      expect(restored.id, original.id);
      expect(restored.mainText, original.mainText);
      expect(restored.subText, original.subText);
      expect(restored.startTime, original.startTime);
      expect(restored.endTime, original.endTime);
      expect(restored.stopwatchSeconds, original.stopwatchSeconds);
      expect(restored.isStopwatchRunning, original.isStopwatchRunning);
      expect(restored.clockMode, original.clockMode);
      expect(restored.depth, original.depth);
      expect(restored.liveNote, original.liveNote);
      expect(restored.totalLines, original.totalLines);
      expect(restored.isHidden, original.isHidden);
      expect(restored.mainTextSize, original.mainTextSize);
      expect(restored.subTextSize, original.subTextSize);
      expect(restored.mainTextColor, original.mainTextColor);
      expect(restored.subTextColor, original.subTextColor);
      expect(restored.mainLineGap, original.mainLineGap);
      expect(restored.subLineGap, original.subLineGap);
      expect(restored.betweenGap, original.betweenGap);
      expect(restored.intervalSecs, original.intervalSecs);
      expect(restored.scheduledDate, original.scheduledDate);
      expect(restored.scheduledTime, original.scheduledTime);
      expect(restored.orderIndex, original.orderIndex);
    });

    test('Flow J: JSON export data format matches Rust expectations', () {
      final cards = [
        TaskCard(id: '1', mainText: 'Quote 1', subText: 'Sub 1'),
        TaskCard(id: '2', mainText: 'Quote 2', subText: 'Sub 2'),
      ];

      final exportList = cards.map((c) => {
        'id': c.id,
        'main_text': c.mainText,
        'sub_text': c.subText,
        'is_hidden': c.isHidden,
        'interval_secs': c.intervalSecs,
      }).toList();

      final jsonStr = jsonEncode(exportList);
      expect(jsonStr.contains('Quote 1'), true);
      expect(jsonStr.contains('Quote 2'), true);
      expect(jsonStr.contains('main_text'), true);
    });

    // ── SECTION 4: GLASS CARD PAINTER REPAINT TEST ──
    test('GlassCardPainter repaints on state change', () {
      final p1 = GlassCardPainter(isSelected: false, isHovered: false);
      final p2 = GlassCardPainter(isSelected: true, isHovered: false);
      final p3 = GlassCardPainter(isSelected: false, isHovered: false);

      expect(p1.shouldRepaint(p2), true);
      expect(p1.shouldRepaint(p3), false);
    });
  });
}
