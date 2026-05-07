import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class BloodPanel extends StatelessWidget {
  final RoboMasterInfo info;
  const BloodPanel({Key? key, required this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return Card(
      color: AppTheme.surfaceContainer,
      child: Padding(
        padding: EdgeInsets.all(20),  // M3E: 更宽松的内边距
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // M3E: 更大胆的标题
            Text(
              '血量',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
                letterSpacing: 0.5,
              ),
            ),
            SizedBox(height: 20),
            _buildRow('英雄', info.heroBlood, 200, AppTheme.heroColor, colorScheme),
            _buildRow('工程', info.engineerBlood, 200, AppTheme.engineerColor, colorScheme),
            _buildRow('步兵1', info.infantryBlood1, 200, AppTheme.infantry1Color, colorScheme),
            _buildRow('步兵2', info.infantryBlood2, 200, AppTheme.infantry2Color, colorScheme),
            _buildRow('前哨站', info.savenBlood, 200, AppTheme.droneColor, colorScheme),
            _buildRow('哨兵', info.sentinelBlood, 400, AppTheme.sentinelColor, colorScheme),
          ],
        ),
      ),
    );
  }
  
  Widget _buildRow(String name, int current, int max, Color color, ColorScheme cs) {
    final ratio = current / max;
    final fillColor = ratio > 0.5 ? color : ratio > 0.25 ? AppTheme.infantry2Color : AppTheme.heroColor;
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 10),  // M3E: 更宽松的行间距
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
          Expanded(
            child: Row(
              children: [
                Expanded(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(8),  // M3E: 更大圆角
                    child: LinearProgressIndicator(
                      value: ratio.clamp(0.0, 1.0),
                      backgroundColor: AppTheme.surfaceContainerHighest,
                      color: fillColor,
                      minHeight: 24,  // M3E: 更高的进度条
                    ),
                  ),
                ),
                SizedBox(width: 12),
                // M3E: 更突出的数值显示
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceContainerHighest,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(
                    '$current',
                    style: TextStyle(
                      color: cs.onSurface,
                      fontWeight: FontWeight.w700,
                      fontSize: 14,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
