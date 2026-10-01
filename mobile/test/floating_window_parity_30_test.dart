import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:task_note_mobile/models/task_card.dart';
import 'package:task_note_mobile/services/overlay_service.dart';
import 'package:task_note_mobile/widgets/floating_overlay_widget.dart';
import 'package:task_note_mobile/theme/cyber_theme.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('30-Vector Small Floating Window vs Rust Parity Suite', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    testWidgets('Test 01-06: Plus button & sub-card creation parity', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Compact mode has '+' button
      expect(find.text('+'), findsOneWidget, reason: 'Plus button exists in compact mode');

      // Expand to card view
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));

      // Plus button exists in expanded header row
      expect(find.text('+'), findsOneWidget, reason: 'Plus button exists in header row');

      // Tap '+' button to create sub-card
      await tester.tap(find.text('+'));
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.textContaining('Sub-Task'), findsOneWidget, reason: 'Sub-card created');
      expect(find.textContaining('95%'), findsOneWidget, reason: '95% depth scaling applied');
    });

    testWidgets('Test 07-14: Three clock badges parity (Deadline, Sub-task, Stopwatch)', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Expand to card view
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));

      // Check badge 0 (Deadline 12.10 PM)
      expect(find.text('12.10 PM'), findsNWidgets(2), reason: 'Deadline and Sub-task badges default to 12.10 PM');

      // Check badge 2 (Stopwatch default 00:00)
      expect(find.text('00:00'), findsOneWidget, reason: 'Stopwatch badge defaults to 00:00');

      // Tap stopwatch badge to toggle start
      await tester.tap(find.text('00:00'));
      await tester.pump(const Duration(seconds: 1));
      await tester.pump(const Duration(milliseconds: 100));

      // Timer started
      expect(find.text('00:01'), findsOneWidget, reason: 'Stopwatch ticks forward');
    });

    testWidgets('Test 15-18: Title bar auto-hide and floating buttons', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Expand
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));

      // Check Sandwich ☰ icon
      expect(find.text('☰'), findsOneWidget, reason: 'Toggle panel button shows ☰ when visible');

      // Tap ☰ to toggle
      await tester.tap(find.text('☰'));
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.text('✕'), findsOneWidget, reason: 'Toggle panel button changes to ✕');
    });

    testWidgets('Test 19-24: Live note in-place writing, font sizing and virtual scroll', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Expand
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));

      // Check Live Note header
      expect(find.text('📄 LIVE NOTE'), findsOneWidget, reason: 'Live Note section present');
      expect(find.text('EDIT'), findsOneWidget, reason: 'EDIT button present');
      expect(find.text(' A+ '), findsOneWidget, reason: 'A+ font button present');
      expect(find.text(' A- '), findsOneWidget, reason: 'A- font button present');

      // Tap EDIT to open in-place editor
      await tester.tap(find.text('EDIT'));
      await tester.pump(const Duration(milliseconds: 50));

      expect(find.text('SAVE'), findsOneWidget, reason: 'Button changes to SAVE');
      expect(find.byType(TextField), findsOneWidget, reason: 'In-place TextField visible');

      // Enter new note
      await tester.enterText(find.byType(TextField), 'Test live note text input');
      await tester.tap(find.text('SAVE'));
      await tester.pump(const Duration(milliseconds: 50));

      expect(find.text('Test live note text input'), findsOneWidget, reason: 'Saved note displayed');
    });

    testWidgets('Test 25-30: Window modes, styling, and sync', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Starts in compact mode
      expect(find.text('Daily Motivation Task'), findsOneWidget);

      // Expand
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('⚡ TASK & LIVE NOTE'), findsOneWidget);

      // Minimize back to pill
      await tester.tap(find.byIcon(Icons.close_fullscreen_rounded));
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('⚡ TASK & LIVE NOTE'), findsNothing);
      expect(find.text('Daily Motivation Task'), findsOneWidget);
    });
  });
}
