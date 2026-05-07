import 'dart:async';
import 'dart:io';
import '../models/laser_observation.dart';

class UdpClient {
  final int port;
  final Function(LaserObservation) onData;
  final Function(String) onError;
  final Function(bool) onConnectionChanged;
  
  RawDatagramSocket? _socket;
  bool _isListening = false;
  Timer? _timeoutTimer;
  
  UdpClient({
    required this.port,
    required this.onData,
    required this.onError,
    required this.onConnectionChanged,
  });
  
  bool get isListening => _isListening;
  
  Future<void> startListening() async {
    try {
      _socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, port);
      _isListening = true;
      onConnectionChanged(true);
      
      _socket!.listen((RawSocketEvent event) {
        if (event == RawSocketEvent.read) {
          final datagram = _socket!.receive();
          if (datagram != null) {
            _onData(datagram.data);
          }
        }
      });
      
      // 超时检测
      _resetTimeout();
    } catch (e) {
      onError(e.toString());
      onConnectionChanged(false);
    }
  }
  
  void _onData(List<int> data) {
    _resetTimeout();
    final obs = LaserObservation.parse(data);
    if (obs != null) {
      onData(obs);
    }
  }
  
  void _resetTimeout() {
    _timeoutTimer?.cancel();
    _timeoutTimer = Timer(Duration(seconds: 3), () {
      onConnectionChanged(false);
    });
  }
  
  void stopListening() {
    _timeoutTimer?.cancel();
    _socket?.close();
    _socket = null;
    _isListening = false;
    // 不触发回调，避免在 dispose 时调用 setState
  }
}
