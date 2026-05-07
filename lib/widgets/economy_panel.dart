import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class EconomyPanel extends StatelessWidget {
  final RoboMasterInfo info;
  const EconomyPanel({Key? key, required this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final ratio = info.economicTotal > 0 ? info.economicRemain / info.economicTotal : 0.0;
    return Card(
      color: AppTheme.surfaceContainer,
      child: Padding(
        padding: EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '经济',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
                letterSpacing: 0.5,
              ),
            ),
            SizedBox(height: 20),
            // M3E: 更突出的数值显示
            Container(
              padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: AppTheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(16),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    '${info.economicRemain}',
                    style: TextStyle(
                      fontSize: 36,  // M3E: 更大字号
                      fontWeight: FontWeight.w800,
                      color: AppTheme.primary,
                    ),
                  ),
                  Text(
                    ' / ${info.economicTotal}',
                    style: TextStyle(
                      fontSize: 18,
                      color: colorScheme.onSurfaceVariant,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ],
              ),
            ),
            SizedBox(height: 16),
            ClipRRect(
              borderRadius: BorderRadius.circular(8),
              child: LinearProgressIndicator(
                value: ratio.clamp(0.0, 1.0),
                backgroundColor: AppTheme.surfaceContainerHighest,
                color: AppTheme.primary,
                minHeight: 24,  // M3E: 更高的进度条
              ),
            ),
          ],
        ),
      ),
    );
  }
}
