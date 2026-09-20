import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:task_note_mobile/models/task_card.dart';
import 'package:task_note_mobile/widgets/task_card_widget.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Responsive Mobile Screen Dimension Tests', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    final testCard = TaskCard(
      id: 'resp_1',
      mainText: 'Responsive Layout Test Quote',
      subText: 'Ensuring zero overflow on various phone sizes',
      startTime: '10:00 AM',
      endTime: '12:00 PM',
      stopwatchSeconds: 45,
    );

    // 1. Small phone (iPhone SE / small Android: 320 x 568)
    testWidgets('Small screen (320x568) renders without overflow', (tester) async {
      tester.view.physicalSize = const Size(320, 568);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: TaskCardWidget(
                card: testCard,
                index: 0,
                totalCards: 1,
                isSelected: true,
                cardScale: 1.0,
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
        ),
      );

      expect(tester.takeException(), isNull, reason: 'No RenderFlex overflow on small screen');
      expect(find.text('Responsive Layout Test Quote'), findsOneWidget);
    });

    // 2. Standard phone (390 x 844)
    testWidgets('Standard phone (390x844) renders without overflow', (tester) async {
      tester.view.physicalSize = const Size(390, 844);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: TaskCardWidget(
                card: testCard,
                index: 0,
                totalCards: 1,
                isSelected: true,
                cardScale: 1.0,
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
        ),
      );

      expect(tester.takeException(), isNull);
      expect(find.text('Responsive Layout Test Quote'), findsOneWidget);
    });

    // 3. Tablet / Large screen (768 x 1024)
    testWidgets('Tablet screen (768x1024) renders without overflow', (tester) async {
      tester.view.physicalSize = const Size(768, 1024);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: TaskCardWidget(
                card: testCard,
                index: 0,
                totalCards: 1,
                isSelected: true,
                cardScale: 1.0,
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
        ),
      );

      expect(tester.takeException(), isNull);
      expect(find.text('Responsive Layout Test Quote'), findsOneWidget);
    });
  });
}
