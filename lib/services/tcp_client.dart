import 'dart:async';
import 'dart:io';
import '../models/robo_master_info.dart';

class TcpClient {
  final String host;
  final int port;
  final Function(RoboMasterInfo) onData;
  final Function(String) onError;
  final Function(bool) onConnectionChanged;
  
  Socket? _socket;
  bool _isConnected = false;
  Timer? _reconnectTimer;
  List<int> _buffer = [];
  static const int _bufferThreshold = 200;
  
  TcpClient({
    required this.host,
    required this.port,
    required this.onData,
    required this.onError,
    required this.onConnectionChanged,
  });
  
  bool get isConnected => _isConnected;
  
  Future<void> connect() async {
    try {
      _socket = await Socket.connect(host, port);
      _isConnected = true;
      onConnectionChanged(true);
      
      _socket!.listen(
        (data) => _onData(data),
        onError: (error) => _onError(error.toString()),
        onDone: () => _onDisconnected(),
      );
    } catch (e) {
      _onError(e.toString());
      _scheduleReconnect();
    }
  }
  
  void _onData(List<int> data) {
    _buffer.addAll(data);
    
    while (_buffer.length >= _bufferThreshold) {
      final info = RoboMasterInfo.parse(_buffer);
      if (info != null) {
        onData(info);
        _buffer.clear();
        break;
      } else {
        _buffer.removeAt(0);
      }
    }
  }
  
  void _onError(String error) {
    onError(error);
    _onDisconnected();
  }
  
  void _onDisconnected() {
    _isConnected = false;
    onConnectionChanged(false);
    _socket?.destroy();
    _socket = null;
    _buffer.clear();
    _scheduleReconnect();
  }
  
  void _scheduleReconnect() {
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(Duration(seconds: 2), () => connect());
  }
  
  void disconnect() {
    _reconnectTimer?.cancel();
    _socket?.destroy();
    _socket = null;
    _isConnected = false;
    _buffer.clear();
  }
}
