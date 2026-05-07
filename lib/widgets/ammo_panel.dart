import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class AmmoPanel extends StatelessWidget {
  final RoboMasterInfo info;
  const AmmoPanel({Key? key, required this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return Card(
      color: AppTheme.surfaceContainer,
      child: Padding(
        padding: EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '弹药',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
                letterSpacing: 0.5,
              ),
            ),
            SizedBox(height: 20),
            _buildRow('英雄', info.heroAmmunition, AppTheme.heroColor, colorScheme),
            _buildRow('步兵1', info.infantryAmmunition1, AppTheme.infantry1Color, colorScheme),
            _buildRow('步兵2', info.infantryAmmunition2, AppTheme.infantry2Color, colorScheme),
            _buildRow('无人机', info.droneAmmunition, AppTheme.droneColor, colorScheme),
            _buildRow('哨兵', info.sentinelAmmunition, AppTheme.sentinelColor, colorScheme),
          ],
        ),
      ),
    );
  }
  
  Widget _buildRow(String name, int ammo, Color color, ColorScheme cs) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 10),
      child: Row(
        children: [
          SizedBox(
            width: 64,
            child: Text(
              name,
              style: TextStyle(
                color: cs.onSurfaceVariant,
                fontSize: 14,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          // M3E: 更突出的数值显示
          Container(
            padding: EdgeInsets.symmetric(horizontal: 14, vertical: 8),
            decoration: BoxDecoration(
              color: AppTheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(
              '$ammo',
              style: TextStyle(
                color: color,
                fontSize: 28,  // M3E: 更大字号
                fontWeight: FontWeight.w800,  // M3E: 更粗字重
              ),
            ),
          ),
        ],
      ),
    );
  }
}
