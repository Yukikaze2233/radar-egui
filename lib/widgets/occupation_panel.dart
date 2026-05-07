import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class OccupationPanel extends StatelessWidget {
  final RoboMasterInfo info;
  const OccupationPanel({Key? key, required this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final labels = ['A', 'B', 'C', 'D', 'E', 'F'];
    return Card(
      color: AppTheme.surfaceContainer,
      child: Padding(
        padding: EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '占领状态',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
                letterSpacing: 0.5,
              ),
            ),
            SizedBox(height: 20),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceAround,
              children: List.generate(6, (index) {
                final isOccupied = info.occupationStatus[index] != 0;
                return Column(
                  children: [
                    // M3E: 更大、更有表现力的占领点
                    Container(
                      width: 56, height: 56,
                      decoration: BoxDecoration(
                        color: isOccupied ? AppTheme.primary : AppTheme.surfaceContainerHighest,
                        borderRadius: BorderRadius.circular(16),  // M3E: 更大圆角
                        boxShadow: isOccupied ? [
                          BoxShadow(
                            color: AppTheme.primary.withOpacity(0.3),
                            blurRadius: 8,
                            offset: Offset(0, 4),
                          ),
                        ] : null,
                      ),
                      child: Center(
                        child: Text(
                          labels[index],
                          style: TextStyle(
                            color: isOccupied ? AppTheme.onPrimary : colorScheme.onSurfaceVariant,
                            fontSize: 22,
                            fontWeight: FontWeight.w800,  // M3E: 更粗字重
                          ),
                        ),
                      ),
                    ),
                    SizedBox(height: 8),
                    // M3E: 更清晰的状态标签
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                      decoration: BoxDecoration(
                        color: isOccupied ? AppTheme.primary.withOpacity(0.2) : AppTheme.surfaceContainerHighest,
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: Text(
                        isOccupied ? '已占领' : '未占领',
                        style: TextStyle(
                          color: isOccupied ? AppTheme.onPrimaryContainer : colorScheme.onSurfaceVariant,
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ],
                );
              }),
            ),
          ],
        ),
      ),
    );
  }
}
