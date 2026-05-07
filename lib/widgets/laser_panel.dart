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
      padding: EdgeInsets.all(16),
      children: [
        // 连接状态
        Card(
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('连接状态', style: Theme.of(context).textTheme.titleLarge),
                SizedBox(height: 16),
                Row(
                  children: [
                    Icon(Icons.circle, size: 12, color: isOnline ? Colors.green : Colors.red),
                    SizedBox(width: 8),
                    Text(isOnline ? '在线' : '离线', style: TextStyle(color: isOnline ? Colors.green : Colors.red, fontSize: 16)),
                  ],
                ),
              ],
            ),
          ),
        ),
        SizedBox(height: 16),
        
        // 目标检测
        Card(
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('目标检测', style: Theme.of(context).textTheme.titleLarge),
                SizedBox(height: 16),
                if (info != null && info!.detected) ...[
                  Text('已检测到目标', style: TextStyle(color: Colors.green, fontSize: 16)),
                  SizedBox(height: 12),
                  _buildRow('中心 X', '${info!.center[0].toStringAsFixed(1)}', colorScheme),
                  _buildRow('中心 Y', '${info!.center[1].toStringAsFixed(1)}', colorScheme),
                  _buildRow('亮度', '${info!.brightness.toStringAsFixed(2)}', colorScheme),
                  _buildRow('轮廓点数', '${info!.contour.length}', colorScheme),
                ] else ...[
                  Text('未检测到目标', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 16)),
                ],
              ],
            ),
          ),
        ),
        SizedBox(height: 16),
        
        // 模型候选
        Card(
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('模型候选', style: Theme.of(context).textTheme.titleLarge),
                SizedBox(height: 16),
                if (info == null || info!.candidates.isEmpty) ...[
                  Text('无候选', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 16)),
                ] else ...[
                  SingleChildScrollView(
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
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(width: 80, child: Text(label, style: TextStyle(color: cs.onSurfaceVariant, fontSize: 14))),
          Text(value, style: TextStyle(color: cs.onSurface, fontSize: 16)),
        ],
      ),
    );
  }
}
