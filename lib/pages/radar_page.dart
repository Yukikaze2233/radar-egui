import 'dart:async';
import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/robo_master_info.dart';
import '../models/laser_observation.dart';
import '../services/tcp_client.dart';
import '../services/udp_client.dart';
import '../widgets/minimap.dart';
import '../widgets/status_panels.dart';
import '../widgets/laser_panel.dart';

class RadarPage extends StatefulWidget {
  @override
  _RadarPageState createState() => _RadarPageState();
}

class _RadarPageState extends State<RadarPage> with SingleTickerProviderStateMixin {
  // Radar 状态
  RoboMasterInfo? _radarInfo;
  bool _radarConnected = false;
  late TcpClient _tcpClient;
  int _radarDataCount = 0;
  
  // Laser 状态
  LaserObservation? _laserInfo;
  bool _laserConnected = false;
  late UdpClient _udpClient;
  
  // UI 状态
  late TabController _tabController;
  Timer? _uptimeTimer;
  Duration _uptime = Duration.zero;
  DateTime _startTime = DateTime.now();
  
  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    
    // TCP 客户端 (Radar)
    _tcpClient = TcpClient(
      host: '127.0.0.1',
      port: 2000,
      onData: (info) {
        setState(() {
          _radarInfo = info;
          _radarDataCount++;
        });
      },
      onError: (error) => print('TCP Error: $error'),
      onConnectionChanged: (connected) {
        setState(() => _radarConnected = connected);
      },
    );
    _tcpClient.connect();
    
    // UDP 客户端 (Laser)
    _udpClient = UdpClient(
      port: 5001,
      onData: (obs) {
        setState(() => _laserInfo = obs);
      },
      onError: (error) => print('UDP Error: $error'),
      onConnectionChanged: (connected) {
        setState(() => _laserConnected = connected);
      },
    );
    _udpClient.startListening();
    
    // 更新运行时间
    _uptimeTimer = Timer.periodic(Duration(seconds: 1), (timer) {
      setState(() => _uptime = DateTime.now().difference(_startTime));
    });
  }
  
  @override
  void dispose() {
    _uptimeTimer?.cancel();
    _tcpClient.disconnect();
    _udpClient.stopListening();
    _tabController.dispose();
    super.dispose();
  }
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // M3E: 更现代的 AppBar
      appBar: AppBar(
        title: Text(
          'Radar HUD',
          style: TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.w800,
            letterSpacing: 0.5,
          ),
        ),
        // M3E: 更美观的 TabBar
        bottom: TabBar(
          controller: _tabController,
          indicatorSize: TabBarIndicatorSize.label,
          indicatorWeight: 3,
          tabs: [
            Tab(text: 'Radar', icon: Icon(Icons.radar)),
            Tab(text: 'Laser', icon: Icon(Icons.lens)),
          ],
        ),
        actions: [
          // M3E: 更美观的连接状态
          Center(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  _buildStatusIndicator('Radar', _radarConnected),
                  SizedBox(width: 12),
                  _buildStatusIndicator('Laser', _laserConnected),
                ],
              ),
            ),
          ),
        ],
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          // Radar 标签页 - M3E: 更宽松的布局
          Row(
            children: [
              Expanded(
                flex: 2,
                child: Padding(
                  padding: EdgeInsets.all(16),  // M3E: 更大的边距
                  child: MinimapWidget(info: _radarInfo),
                ),
              ),
              Expanded(
                flex: 3,
                child: StatusPanels(info: _radarInfo),
              ),
            ],
          ),
          // Laser 标签页
          LaserPanel(info: _laserInfo),
        ],
      ),
      // M3E: 更美观的底部状态栏
      bottomNavigationBar: Container(
        padding: EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        decoration: BoxDecoration(
          color: AppTheme.surfaceContainer,
          border: Border(
            top: BorderSide(
              color: AppTheme.surfaceContainerHighest,
              width: 1,
            ),
          ),
        ),
        child: Row(
          children: [
            _buildStatusItem('运行时间', '${_uptime.inHours}h ${_uptime.inMinutes % 60}m ${_uptime.inSeconds % 60}s'),
            SizedBox(width: 24),
            _buildStatusItem('数据', '$_radarDataCount'),
            SizedBox(width: 24),
            _buildStatusItem('目标', '127.0.0.1:2000'),
            Spacer(),
            _buildStatusItem('Laser', 'UDP:5001'),
          ],
        ),
      ),
    );
  }
  
  // M3E: 状态指示器组件
  Widget _buildStatusIndicator(String label, bool isConnected) {
    return Container(
      padding: EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: isConnected 
            ? AppTheme.engineerColor.withOpacity(0.2)
            : AppTheme.heroColor.withOpacity(0.2),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.circle,
            size: 8,
            color: isConnected ? AppTheme.engineerColor : AppTheme.heroColor,
          ),
          SizedBox(width: 6),
          Text(
            label,
            style: TextStyle(
              color: isConnected ? AppTheme.engineerColor : AppTheme.heroColor,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
  
  // M3E: 状态项组件
  Widget _buildStatusItem(String label, String value) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            color: AppTheme.surfaceContainerHighest,
            fontSize: 10,
            fontWeight: FontWeight.w500,
          ),
        ),
        SizedBox(height: 2),
        Text(
          value,
          style: TextStyle(
            color: AppTheme.onPrimaryContainer,
            fontSize: 12,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    );
  }
}
