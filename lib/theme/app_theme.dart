import 'package:flutter/material.dart';

class AppTheme {
  // 机器人颜色 - 更鲜艳
  static const Color heroColor = Color(0xFFFF6B6B);      // 柔红
  static const Color engineerColor = Color(0xFF51CF66);   // 翠绿
  static const Color infantry1Color = Color(0xFF339AF0);   // 天蓝
  static const Color infantry2Color = Color(0xFFFFD43B);   // 明黄
  static const Color droneColor = Color(0xFF20C997);       // 碧绿
  static const Color sentinelColor = Color(0xCC845EF7);    // 薰紫
  
  // Laser 类别颜色
  static const Color laserPurple = Color(0xFF845EF7);
  static const Color laserRed = Color(0xFFFF6B6B);
  static const Color laserBlue = Color(0xFF339AF0);
  
  // M3E: 表面色层级
  static const Color surfaceDim = Color(0xFF11111B);
  static const Color surface = Color(0xFF1E1E2E);
  static const Color surfaceBright = Color(0xFF2D2D3F);
  static const Color surfaceContainerLowest = Color(0xFF181825);
  static const Color surfaceContainerLow = Color(0xFF1E1E2E);
  static const Color surfaceContainer = Color(0xFF242436);
  static const Color surfaceContainerHigh = Color(0xFF2D2D3F);
  static const Color surfaceContainerHighest = Color(0xFF383850);
  
  // M3E: 强调色
  static const Color primary = Color(0xFF339AF0);
  static const Color onPrimary = Color(0xFFFFFFFF);
  static const Color primaryContainer = Color(0xFF1A3A5C);
  static const Color onPrimaryContainer = Color(0xFF9DCAFF);
  
  // Material 3 Expressive 深色主题
  static ThemeData get darkTheme {
    return ThemeData(
      useMaterial3: true,
      colorScheme: ColorScheme.fromSeed(
        seedColor: primary,
        brightness: Brightness.dark,
      ),
      // M3E: 更大胆的卡片样式
      cardTheme: CardThemeData(
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(20),  // M3E: 更大圆角
        ),
        clipBehavior: Clip.antiAlias,
      ),
      // M3E: 更现代的 AppBar
      appBarTheme: AppBarTheme(
        centerTitle: false,
        elevation: 0,
        scrolledUnderElevation: 2,
      ),
      // M3E: 更圆滑的进度条
      progressIndicatorTheme: ProgressIndicatorThemeData(
        linearTrackColor: surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
    );
  }
  
  // 获取类别颜色
  static Color getClassColor(int classId) {
    switch (classId) {
      case 0: return laserPurple;
      case 1: return laserRed;
      case 2: return laserBlue;
      default: return Colors.grey;
    }
  }
  
  // 获取类别名称
  static String getClassName(int classId) {
    switch (classId) {
      case 0: return 'Purple';
      case 1: return 'Red';
      case 2: return 'Blue';
      default: return '?';
    }
  }
}
