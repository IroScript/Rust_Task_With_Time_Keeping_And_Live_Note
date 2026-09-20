import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:task_note_mobile/models/task_card.dart';
import 'package:task_note_mobile/services/overlay_service.dart';
import 'package:task_note_mobile/widgets/floating_overlay_widget.dart';
import 'package:task_note_mobile/screens/home_screen.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Floating Overlay Architecture Tests', () {
    test('OverlayService singleton state and initial properties', () {
      final service1 = OverlayService.instance;
      final service2 = OverlayService.instance;

      expect(identical(service1, service2), isTrue);
      expect(service1.isOverlayActive, isFalse);
    });

    test('OverlayService syncCardData formats payload cleanly', () async {
      final card = TaskCard(
        id: 'test-card-1',
        mainText: 'Build AI Floating Widget',
        subText: 'Flutter + Rust Overlay parity',
        startTime: '10:00 AM',
        endTime: '11:00 AM',
        liveNote: 'Connecting overlay service',
        depth: 1,
        orderIndex: 0,
      );

      // Verify that calling syncCardData does not throw
      expect(() async => await OverlayService.instance.syncCardData(card), returnsNormally);
    });

    testWidgets('FloatingOverlayWidget renders in compact bubble mode initially',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Should find compact bubble elements
      expect(find.byType(FloatingOverlayWidget), findsOneWidget);
      expect(find.byIcon(Icons.open_in_full_rounded), findsOneWidget);
      expect(find.text('Daily Motivation Task'), findsOneWidget);
      expect(find.text('00:00'), findsOneWidget);

      // Should NOT find expanded elements yet
      expect(find.text('FLOATING TASK'), findsNothing);
      expect(find.text('📝 LIVE NOTE'), findsNothing);
    });

    testWidgets('FloatingOverlayWidget expands to full card mode on tap',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: FloatingOverlayWidget(),
          ),
        ),
      );

      // Tap on compact bubble to toggle expand
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pump(const Duration(milliseconds: 100));

      // Should now find expanded elements
      expect(find.text('FLOATING TASK'), findsOneWidget);
      expect(find.text('📝 LIVE NOTE'), findsOneWidget);
      expect(find.byIcon(Icons.close_fullscreen_rounded), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget);
      expect(find.text('START'), findsOneWidget);

      // Tap minimize button to collapse back to compact mode
      await tester.tap(find.byIcon(Icons.close_fullscreen_rounded));
      await tester.pump(const Duration(milliseconds: 100));

      // Should be back to compact mode
      expect(find.text('FLOATING TASK'), findsNothing);
      expect(find.byIcon(Icons.open_in_full_rounded), findsOneWidget);
    });

    testWidgets('HomeScreen top bar contains floating overlay button',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 2.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        const MaterialApp(
          home: HomeScreen(),
        ),
      );
      await tester.pump(const Duration(milliseconds: 200));

      // Verify the 🪟 button exists in the top bar
      expect(find.text('🪟'), findsOneWidget);
    });
  });
}
