import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class MinimapWidget extends StatelessWidget {
  final RoboMasterInfo? info;
  
  const MinimapWidget({Key? key, this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    return Card(
      child: ClipRRect(
        borderRadius: BorderRadius.circular(12),
        child: CustomPaint(
          painter: MinimapPainter(info: info),
          size: Size.infinite,
        ),
      ),
    );
  }
}

class MinimapPainter extends CustomPainter {
  final RoboMasterInfo? info;
  
  MinimapPainter({this.info});
  
  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final scale = size.width * 0.45 / 3000.0;
    
    // 背景
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), Paint()..color = Color(0xFF1E1E2E));
    
    // 网格
    final gridPaint = Paint()..color = Color(0xFF313244)..strokeWidth = 0.5;
    for (int i = 0; i <= 10; i++) {
      final t = i / 10.0;
      final x = size.width * t;
      final y = size.height * t;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), gridPaint);
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }
    
    // 十字准星
    final crosshairPaint = Paint()..color = Color(0xFF45475A)..strokeWidth = 0.5;
    canvas.drawLine(Offset(center.dx, 0), Offset(center.dx, size.height), crosshairPaint);
    canvas.drawLine(Offset(0, center.dy), Offset(size.width, center.dy), crosshairPaint);
    
    // 机器人
    if (info != null) {
      _drawRobots(canvas, center, scale);
    }
  }
  
  void _drawRobots(Canvas canvas, Offset center, double scale) {
    final robots = [
      ('英雄', info!.heroPosition, AppTheme.heroColor),
      ('工程', info!.engineerPosition, AppTheme.engineerColor),
      ('步兵1', info!.infantryPosition1, AppTheme.infantry1Color),
      ('步兵2', info!.infantryPosition2, AppTheme.infantry2Color),
      ('无人机', info!.dronePosition, AppTheme.droneColor),
      ('哨兵', info!.sentinelPosition, AppTheme.sentinelColor),
    ];
    
    for (final robot in robots) {
      final name = robot.$1;
      final pos = robot.$2;
      final color = robot.$3;
      
      final screenPos = Offset(
        center.dx + pos[0] * scale,
        center.dy - pos[1] * scale,
      );
      
      canvas.drawCircle(screenPos, 7, Paint()..color = color);
      
      final textPainter = TextPainter(
        text: TextSpan(text: name, style: TextStyle(color: Color(0xFFA6ADC8), fontSize: 14)),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(canvas, screenPos + Offset(14, -10));
    }
  }
  
  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}
