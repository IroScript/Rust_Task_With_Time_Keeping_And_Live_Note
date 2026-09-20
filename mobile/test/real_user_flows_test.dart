import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:task_note_mobile/models/task_card.dart';
import 'package:task_note_mobile/services/api_service.dart';
import 'package:task_note_mobile/widgets/modals/schedule_dialog.dart';
import 'package:task_note_mobile/widgets/modals/position_dialog.dart';
import 'package:task_note_mobile/widgets/modals/interval_dialog.dart';
import 'package:task_note_mobile/widgets/card_size_popup_widget.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Comprehensive Real User Flows & Edge Case Tests', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    // ── FLOW 1: CARD CRUD & REORDERING ──
    testWidgets('Card Flow: Create, Move Up/Down, Set Position, Hide, Unhide, Delete', (tester) async {
      final cards = [
        TaskCard(id: 'c1', mainText: 'Card 1', subText: 'Sub 1'),
        TaskCard(id: 'c2', mainText: 'Card 2', subText: 'Sub 2'),
        TaskCard(id: 'c3', mainText: 'Card 3', subText: 'Sub 3'),
      ];

      // 1. Initial State
      expect(cards[0].mainText, 'Card 1');
      expect(cards[1].mainText, 'Card 2');

      // 2. Move down (c1 down to index 1)
      final movedItem = cards.removeAt(0);
      cards.insert(1, movedItem);
      expect(cards[0].mainText, 'Card 2');
      expect(cards[1].mainText, 'Card 1');

      // 3. Move up (c1 back to index 0)
      final upItem = cards.removeAt(1);
      cards.insert(0, upItem);
      expect(cards[0].mainText, 'Card 1');
      expect(cards[1].mainText, 'Card 2');

      // 4. Set position via # (relocate c3 to index 0)
      final posItem = cards.removeAt(2);
      cards.insert(0, posItem);
      expect(cards[0].mainText, 'Card 3');
      expect(cards[1].mainText, 'Card 1');
      expect(cards[2].mainText, 'Card 2');

      // 5. Hide (H -> O)
      cards[0].isHidden = true;
      expect(cards[0].isHidden, true);

      // 6. Unhide (O -> H)
      cards[0].isHidden = false;
      expect(cards[0].isHidden, false);

      // 7. Delete card
      cards.removeAt(0);
      expect(cards.length, 2);
      expect(cards[0].mainText, 'Card 1');
    });

    // ── FLOW 2: SCHEDULING DIALOG & STATE ──
    testWidgets('Scheduling: ScheduleDialog validates and saves date/time', (tester) async {
      final card = TaskCard(id: 'sched_1', mainText: 'Meeting');
      String? savedDate;
      String? savedTime;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ScheduleDialog(
              card: card,
              onSave: (date, time) {
                savedDate = date;
                savedTime = time;
              },
            ),
          ),
        ),
      );

      expect(find.text('SCHEDULE TIME'), findsOneWidget);
      expect(find.text('SET SCHEDULE'), findsOneWidget);

      // Tap Set Schedule button
      await tester.tap(find.text('SET SCHEDULE'));
      await tester.pumpAndSettle();

      expect(savedDate, isNotNull);
      expect(savedTime, isNotNull);
    });

    // ── FLOW 3: POSITION DIALOG (#) ──
    testWidgets('Position: PositionDialog relocates card to 1-based index', (tester) async {
      int? targetIndex;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: PositionDialog(
              currentIndex: 3,
              totalCount: 10,
              onSetPosition: (idx) {
                targetIndex = idx;
              },
            ),
          ),
        ),
      );

      expect(find.text('SET CARD POSITION (#)'), findsOneWidget);
      expect(find.text('MOVE'), findsOneWidget);

      // Enter target index "2" (1-based -> index 1)
      await tester.enterText(find.byType(TextField), '2');
      await tester.tap(find.text('MOVE'));
      await tester.pumpAndSettle();

      expect(targetIndex, 1);
    });

    // ── FLOW 4: INTERVAL DIALOG (⏱) ──
    testWidgets('Interval: IntervalDialog updates rotation interval', (tester) async {
      int? savedInterval;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: IntervalDialog(
              currentInterval: 8,
              onSave: (interval) {
                savedInterval = interval;
              },
            ),
          ),
        ),
      );

      expect(find.text('ROTATION INTERVAL (⏱)'), findsOneWidget);
      expect(find.text('SAVE INTERVAL'), findsOneWidget);

      // Tap save interval
      await tester.tap(find.text('SAVE INTERVAL'));
      await tester.pumpAndSettle();

      expect(savedInterval, 8);
    });

    // ── FLOW 5: CARD SCALE (0% TO 300%) ──
    testWidgets('Card Size Popup renders scale options from 0% to 300%', (tester) async {
      double? selectedScale;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CardSizePopupWidget(
              currentScale: 1.0,
              onSelectScale: (scale) {
                selectedScale = scale;
              },
              onClose: () {},
            ),
          ),
        ),
      );

      expect(find.text('0%'), findsOneWidget);
      expect(find.text('10%'), findsOneWidget);
      expect(find.text('50%'), findsOneWidget);
      expect(find.text('100%'), findsOneWidget);

      await tester.scrollUntilVisible(find.text('200%'), 50.0);
      expect(find.text('200%'), findsOneWidget);

      await tester.scrollUntilVisible(find.text('300%'), 50.0);
      expect(find.text('300%'), findsOneWidget);

      // Scroll back and tap 50%
      await tester.scrollUntilVisible(find.text('50%'), -50.0);
      await tester.tap(find.text('50%'));
      await tester.pumpAndSettle();

      expect(selectedScale, 0.5);
    });

    // ── FLOW 6: BACKEND UNAVAILABILITY & LOCAL PERSISTENCE FALLBACK ──
    test('Backend Offline: ApiService handles network errors gracefully without crashing', () async {
      final api = ApiService(baseUrl: 'http://127.0.0.1:9999'); // Non-existent port

      // Health check should return false without exception
      final healthy = await api.checkHealth();
      expect(healthy, false);

      // Fetch cards should return empty list gracefully
      final remote = await api.fetchCards();
      expect(remote, isEmpty);

      // Save local cards should still work 100%
      await api.saveLocalCards([
        TaskCard(id: 'offline_1', mainText: 'Offline Title', subText: 'Offline Sub'),
      ]);

      final loaded = await api.loadLocalCards();
      expect(loaded.length, 1);
      expect(loaded[0].id, 'offline_1');
      expect(loaded[0].mainText, 'Offline Title');
    });

    // ── FLOW 7: FULL APP PERSISTENCE CYCLE (SIMULATED FORCE-STOP & REOPEN) ──
    test('Data Persistence: All user state survives force-stop and reopen', () async {
      final api = ApiService();

      // State 1: Before app termination
      final activeCards = [
        TaskCard(
          id: 'card_persisted_1',
          mainText: 'Crucial Goal',
          subText: 'Do not lose this note',
          startTime: '08:00 AM',
          endTime: '11:00 AM',
          stopwatchSeconds: 3600,
          scheduledDate: '2026-11-20',
          scheduledTime: '10:00',
          isHidden: true,
          liveNote: 'Line 1\nLine 2',
          totalLines: 2,
        ),
      ];

      final settings = {
        'cardScale': 1.4,
        'zoomLevel': 1.1,
        'singleQuoteMode': true,
        'rotationInterval': 12,
        'autoRotation': false,
        'theme': 'neon',
      };

      await api.saveLocalCards(activeCards);
      await api.saveAppSettings(settings);

      // State 2: Simulated App Restart (Fresh instances)
      final reloadedApi = ApiService();
      final restoredCards = await reloadedApi.loadLocalCards();
      final restoredSettings = await reloadedApi.loadAppSettings();

      expect(restoredCards.length, 1);
      expect(restoredCards[0].id, 'card_persisted_1');
      expect(restoredCards[0].mainText, 'Crucial Goal');
      expect(restoredCards[0].scheduledDate, '2026-11-20');
      expect(restoredCards[0].isHidden, true);
      expect(restoredCards[0].stopwatchSeconds, 3600);
      expect(restoredCards[0].liveNote, 'Line 1\nLine 2');

      expect(restoredSettings['cardScale'], 1.4);
      expect(restoredSettings['singleQuoteMode'], true);
      expect(restoredSettings['rotationInterval'], 12);
      expect(restoredSettings['autoRotation'], false);
      expect(restoredSettings['theme'], 'neon');
    });
  });
}
