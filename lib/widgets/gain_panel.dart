import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';

class GainPanel extends StatelessWidget {
  final RoboMasterInfo info;
  const GainPanel({Key? key, required this.info}) : super(key: key);
  
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
              '增益',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w700,
                color: colorScheme.onSurface,
                letterSpacing: 0.5,
              ),
            ),
            SizedBox(height: 20),
            // M3E: 更美观的表格
            Container(
              decoration: BoxDecoration(
                color: AppTheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(16),
              ),
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: DataTable(
                  columns: [
                    DataColumn(label: Text('机器人')),
                    DataColumn(label: Text('回血'), numeric: true),
                    DataColumn(label: Text('冷却'), numeric: true),
                    DataColumn(label: Text('防御'), numeric: true),
                    DataColumn(label: Text('降防'), numeric: true),
                    DataColumn(label: Text('攻击'), numeric: true),
                  ],
                  rows: [
                    _buildRow('英雄', info.heroGain, AppTheme.heroColor),
                    _buildRow('工程', info.engineerGain, AppTheme.engineerColor),
                    _buildRow('步兵1', info.infantryGain1, AppTheme.infantry1Color),
                    _buildRow('步兵2', info.infantryGain2, AppTheme.infantry2Color),
                    _buildRow('哨兵', info.sentinelGain, AppTheme.sentinelColor),
                  ],
                ),
              ),
            ),
            SizedBox(height: 16),
            Divider(color: AppTheme.surfaceContainerHighest),
            SizedBox(height: 12),
            // M3E: 更突出的哨兵姿态
            Row(
              children: [
                Text(
                  '哨兵姿态',
                  style: TextStyle(
                    color: colorScheme.onSurfaceVariant,
                    fontSize: 14,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                SizedBox(width: 16),
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  decoration: BoxDecoration(
                    color: AppTheme.primaryContainer,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${info.sentinelPosture}',
                    style: TextStyle(
                      color: AppTheme.onPrimaryContainer,
                      fontSize: 18,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
  
  DataRow _buildRow(String name, List<int> gain, Color color) {
    final cooling = gain[1] | (gain[2] << 8);
    final attack = gain[5] | (gain[6] << 8);
    return DataRow(cells: [
      DataCell(Text(name, style: TextStyle(color: color, fontWeight: FontWeight.bold))),
      DataCell(Text('${gain[0]}')),
      DataCell(Text('$cooling')),
      DataCell(Text('${gain[3]}')),
      DataCell(Text('${gain[4]}')),
      DataCell(Text('$attack')),
    ]);
  }
}
