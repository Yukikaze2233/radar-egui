import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/laser_observation.dart';

class LaserPanel extends StatelessWidget {
  final LaserObservation? info;
  const LaserPanel({Key? key, this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final isOnline = info?.isOnline ?? false;
    
    return ListView(
      padding: EdgeInsets.all(20),  // M3E: 更宽松的布局
      children: [
        // M3E: 更美观的连接状态卡片
        Card(
          color: AppTheme.surfaceContainer,
          child: Padding(
            padding: EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '连接状态',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                    color: colorScheme.onSurface,
                    letterSpacing: 0.5,
                  ),
                ),
                SizedBox(height: 20),
                // M3E: 更有表现力的状态指示
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  decoration: BoxDecoration(
                    color: isOnline 
                        ? AppTheme.engineerColor.withOpacity(0.2)
                        : AppTheme.heroColor.withOpacity(0.2),
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.circle,
                        size: 16,
                        color: isOnline ? AppTheme.engineerColor : AppTheme.heroColor,
                      ),
                      SizedBox(width: 12),
                      Text(
                        isOnline ? '在线' : '离线',
                        style: TextStyle(
                          color: isOnline ? AppTheme.engineerColor : AppTheme.heroColor,
                          fontSize: 18,
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        SizedBox(height: 20),
        
        // M3E: 更美观的目标检测卡片
        Card(
          color: AppTheme.surfaceContainer,
          child: Padding(
            padding: EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '目标检测',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                    color: colorScheme.onSurface,
                    letterSpacing: 0.5,
                  ),
                ),
                SizedBox(height: 20),
                if (info != null && info!.detected) ...[
                  // M3E: 更有表现力的检测状态
                  Container(
                    padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    decoration: BoxDecoration(
                      color: AppTheme.engineerColor.withOpacity(0.2),
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          Icons.check_circle,
                          size: 20,
                          color: AppTheme.engineerColor,
                        ),
                        SizedBox(width: 8),
                        Text(
                          '已检测到目标',
                          style: TextStyle(
                            color: AppTheme.engineerColor,
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                  ),
                  SizedBox(height: 16),
                  _buildRow('中心 X', '${info!.center[0].toStringAsFixed(1)}', colorScheme),
                  _buildRow('中心 Y', '${info!.center[1].toStringAsFixed(1)}', colorScheme),
                  _buildRow('亮度', '${info!.brightness.toStringAsFixed(2)}', colorScheme),
                  _buildRow('轮廓点数', '${info!.contour.length}', colorScheme),
                ] else ...[
                  // M3E: 更有表现力的未检测状态
                  Container(
                    padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    decoration: BoxDecoration(
                      color: AppTheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          Icons.help_outline,
                          size: 20,
                          color: colorScheme.onSurfaceVariant,
                        ),
                        SizedBox(width: 8),
                        Text(
                          '未检测到目标',
                          style: TextStyle(
                            color: colorScheme.onSurfaceVariant,
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
        SizedBox(height: 20),
        
        // M3E: 更美观的模型候选卡片
        Card(
          color: AppTheme.surfaceContainer,
          child: Padding(
            padding: EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '模型候选',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                    color: colorScheme.onSurface,
                    letterSpacing: 0.5,
                  ),
                ),
                SizedBox(height: 20),
                if (info == null || info!.candidates.isEmpty) ...[
                  Container(
                    padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    decoration: BoxDecoration(
                      color: AppTheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          Icons.inbox_outlined,
                          size: 20,
                          color: colorScheme.onSurfaceVariant,
                        ),
                        SizedBox(width: 8),
                        Text(
                          '无候选',
                          style: TextStyle(
                            color: colorScheme.onSurfaceVariant,
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                  ),
                ] else ...[
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
                          DataColumn(label: Text('类别')),
                          DataColumn(label: Text('置信度'), numeric: true),
                          DataColumn(label: Text('中心 X'), numeric: true),
                          DataColumn(label: Text('中心 Y'), numeric: true),
                          DataColumn(label: Text('边界框')),
                        ],
                        rows: info!.candidates.map((cand) {
                          final classColor = AppTheme.getClassColor(cand.classId);
                          final className = AppTheme.getClassName(cand.classId);
                          return DataRow(cells: [
                            DataCell(Text(className, style: TextStyle(color: classColor, fontWeight: FontWeight.bold))),
                            DataCell(Text('${(cand.score * 100).toStringAsFixed(0)}%')),
                            DataCell(Text('${cand.center[0].toStringAsFixed(1)}')),
                            DataCell(Text('${cand.center[1].toStringAsFixed(1)}')),
                            DataCell(Text('${cand.bbox[2].toStringAsFixed(0)}×${cand.bbox[3].toStringAsFixed(0)}')),
                          ]);
                        }).toList(),
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ],
    );
  }
  
  Widget _buildRow(String label, String value, ColorScheme cs) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 8),  // M3E: 更宽松的行间距
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: TextStyle(
                color: cs.onSurfaceVariant,
                fontSize: 14,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          Container(
            padding: EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: AppTheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              value,
              style: TextStyle(
                color: cs.onSurface,
                fontSize: 14,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
