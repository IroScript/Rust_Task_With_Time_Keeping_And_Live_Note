import 'package:flutter_test/flutter_test.dart';
import 'package:task_note_mobile/main.dart';

void main() {
  testWidgets('TaskNoteApp smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const TaskNoteApp());
    expect(find.text('DAILY MOTIVATION'), findsOneWidget);
  });
}
