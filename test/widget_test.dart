import 'package:flutter_test/flutter_test.dart';
import 'package:radar_flutter/main.dart';

void main() {
  testWidgets('App should render', (WidgetTester tester) async {
    await tester.pumpWidget(MyApp());
    await tester.pumpAndSettle();
    expect(find.text('Radar HUD'), findsOneWidget);
  });
}
