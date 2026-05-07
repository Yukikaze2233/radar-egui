import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class MinimapWidget extends StatelessWidget {
  final RoboMasterInfo? info;
  
  const MinimapWidget({Key? key, this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    return Card(
      color: AppTheme.surfaceContainer,
      clipBehavior: Clip.antiAlias,
      child: CustomPaint(
        painter: MinimapPainter(info: info),
        size: Size.infinite,
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
    
    // M3E: 更深的背景
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Paint()..color = AppTheme.surfaceDim,
    );
    
    // M3E: 更细腻的网格
    final gridPaint = Paint()
      ..color = AppTheme.surfaceContainerHighest.withOpacity(0.5)
      ..strokeWidth = 0.5;
    for (int i = 0; i <= 10; i++) {
      final t = i / 10.0;
      final x = size.width * t;
      final y = size.height * t;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), gridPaint);
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }
    
    // M3E: 更明显的十字准星
    final crosshairPaint = Paint()
      ..color = AppTheme.surfaceContainerHighest
      ..strokeWidth = 1.0;
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
      
      // M3E: 更大、更有表现力的机器人点
      // 发光效果
      canvas.drawCircle(
        screenPos,
        12,
        Paint()
          ..color = color.withOpacity(0.3)
          ..maskFilter = MaskFilter.blur(BlurStyle.normal, 4),
      );
      // 主体
      canvas.drawCircle(screenPos, 8, Paint()..color = color);
      // 高光
      canvas.drawCircle(
        screenPos - Offset(2, 2),
        3,
        Paint()..color = Colors.white.withOpacity(0.3),
      );
      
      // M3E: 更清晰的标签
      final textPainter = TextPainter(
        text: TextSpan(
          text: name,
          style: TextStyle(
            color: AppTheme.onPrimaryContainer,
            fontSize: 13,
            fontWeight: FontWeight.w600,
          ),
        ),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(canvas, screenPos + Offset(16, -12));
    }
  }
  
  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}
