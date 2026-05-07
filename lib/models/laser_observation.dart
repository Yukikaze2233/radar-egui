import 'dart:typed_data';

class ModelCandidate {
  final double score;
  final int classId;
  final List<double> bbox;
  final List<double> center;
  
  ModelCandidate({
    required this.score,
    required this.classId,
    required this.bbox,
    required this.center,
  });
}

class LaserObservation {
  final bool detected;
  final List<double> center;
  final double brightness;
  final List<List<double>> contour;
  final List<ModelCandidate> candidates;
  final DateTime? receivedAt;
  
  LaserObservation({
    required this.detected,
    required this.center,
    required this.brightness,
    required this.contour,
    required this.candidates,
    this.receivedAt,
  });
  
  factory LaserObservation.defaultValues() {
    return LaserObservation(
      detected: false,
      center: [0, 0],
      brightness: 0,
      contour: [],
      candidates: [],
      receivedAt: null,
    );
  }
  
  bool get isOnline {
    if (receivedAt == null) return false;
    return DateTime.now().difference(receivedAt!).inSeconds < 2;
  }
  
  ModelCandidate? get bestCandidate {
    if (candidates.isEmpty) return null;
    return candidates.reduce((a, b) => a.score > b.score ? a : b);
  }
  
  static LaserObservation? parse(List<int> data) {
    const headerSize = 16;
    if (data.length < headerSize + 5) return null;
    
    // 检查 magic (0x4C47 = "LG")
    int magic = data[0] | (data[1] << 8);
    if (magic != 0x4C47) return null;
    
    // 检查 version
    if (data[2] != 1) return null;
    
    // 读取 payload 长度
    int payloadLen = data[12] | (data[13] << 8) | (data[14] << 16) | (data[15] << 24);
    if (data.length < headerSize + payloadLen) return null;
    
    int offset = headerSize;
    
    // detected
    bool detected = data[offset] != 0;
    offset += 1;
    
    // center
    double centerX = _readFloat32LE(data, offset);
    offset += 4;
    double centerY = _readFloat32LE(data, offset);
    offset += 4;
    
    // brightness
    double brightness = _readFloat32LE(data, offset);
    offset += 4;
    
    // contour
    int contourCount = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
    offset += 4;
    
    List<List<double>> contour = [];
    for (int i = 0; i < contourCount; i++) {
      if (offset + 8 > data.length) return null;
      double x = _readFloat32LE(data, offset);
      offset += 4;
      double y = _readFloat32LE(data, offset);
      offset += 4;
      contour.add([x, y]);
    }
    
    // candidates
    int candCount = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
    offset += 4;
    
    List<ModelCandidate> candidates = [];
    for (int i = 0; i < candCount; i++) {
      if (offset + 28 > data.length) return null;
      
      double score = _readFloat32LE(data, offset);
      offset += 4;
      int classId = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
      offset += 4;
      double bboxX = _readFloat32LE(data, offset);
      offset += 4;
      double bboxY = _readFloat32LE(data, offset);
      offset += 4;
      double bboxW = _readFloat32LE(data, offset);
      offset += 4;
      double bboxH = _readFloat32LE(data, offset);
      offset += 4;
      double cx = _readFloat32LE(data, offset);
      offset += 4;
      double cy = _readFloat32LE(data, offset);
      offset += 4;
      
      candidates.add(ModelCandidate(
        score: score,
        classId: classId,
        bbox: [bboxX, bboxY, bboxW, bboxH],
        center: [cx, cy],
      ));
    }
    
    return LaserObservation(
      detected: detected,
      center: [centerX, centerY],
      brightness: brightness,
      contour: contour,
      candidates: candidates,
      receivedAt: DateTime.now(),
    );
  }
  
  static double _readFloat32LE(List<int> data, int offset) {
    // 使用 ByteData 正确解析 IEEE 754 float
    final bytes = ByteData(4);
    bytes.setUint8(0, data[offset]);
    bytes.setUint8(1, data[offset + 1]);
    bytes.setUint8(2, data[offset + 2]);
    bytes.setUint8(3, data[offset + 3]);
    return bytes.getFloat32(0, Endian.little);
  }
}
