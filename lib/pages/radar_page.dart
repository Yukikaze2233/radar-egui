import 'dart:async';
import 'package:flutter/material.dart';
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
    final colorScheme = Theme.of(context).colorScheme;
    
    return Scaffold(
      appBar: AppBar(
        title: Text('Radar HUD'),
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(text: 'Radar', icon: Icon(Icons.radar)),
            Tab(text: 'Laser', icon: Icon(Icons.lens)),
          ],
        ),
        actions: [
          Center(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Icon(Icons.circle, size: 12, color: _radarConnected ? Colors.green : Colors.red),
                  SizedBox(width: 8),
                  Text(_radarConnected ? 'Radar' : 'Off', style: TextStyle(color: _radarConnected ? Colors.green : Colors.red)),
                  SizedBox(width: 16),
                  Icon(Icons.circle, size: 12, color: _laserConnected ? Colors.green : Colors.red),
                  SizedBox(width: 8),
                  Text(_laserConnected ? 'Laser' : 'Off', style: TextStyle(color: _laserConnected ? Colors.green : Colors.red)),
                ],
              ),
            ),
          ),
        ],
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          // Radar 标签页
          Row(
            children: [
              Expanded(flex: 2, child: Padding(padding: EdgeInsets.all(12), child: MinimapWidget(info: _radarInfo))),
              Expanded(flex: 3, child: StatusPanels(info: _radarInfo)),
            ],
          ),
          // Laser 标签页
          LaserPanel(info: _laserInfo),
        ],
      ),
      bottomNavigationBar: Container(
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        color: colorScheme.surfaceVariant,
        child: Row(
          children: [
            Text('运行时间: ${_uptime.inHours}h ${_uptime.inMinutes % 60}m ${_uptime.inSeconds % 60}s', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 12)),
            SizedBox(width: 16),
            Text('数据: $_radarDataCount', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 12)),
            SizedBox(width: 16),
            Text('目标: 127.0.0.1:2000', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 12)),
            Spacer(),
            Text('Laser UDP: 0.0.0.0:5001', style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 12)),
          ],
        ),
      ),
    );
  }
}
