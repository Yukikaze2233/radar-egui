import 'package:flutter/material.dart';
import 'theme/app_theme.dart';
import 'pages/radar_page.dart';

void main() {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Radar HUD',
      theme: AppTheme.darkTheme,
      home: RadarPage(),
      debugShowCheckedModeBanner: false,
    );
  }
}
