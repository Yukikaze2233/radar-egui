class RoboMasterInfo {
  // 位置 (0x0A01)
  final List<int> heroPosition;
  final List<int> engineerPosition;
  final List<int> infantryPosition1;
  final List<int> infantryPosition2;
  final List<int> dronePosition;
  final List<int> sentinelPosition;
  
  // 血量 (0x0A02)
  final int heroBlood;
  final int engineerBlood;
  final int infantryBlood1;
  final int infantryBlood2;
  final int savenBlood;
  final int sentinelBlood;
  
  // 弹药 (0x0A03)
  final int heroAmmunition;
  final int infantryAmmunition1;
  final int infantryAmmunition2;
  final int droneAmmunition;
  final int sentinelAmmunition;
  
  // 经济 (0x0A04)
  final int economicRemain;
  final int economicTotal;
  final List<int> occupationStatus;
  
  // 增益 (0x0A05)
  final List<int> heroGain;
  final List<int> engineerGain;
  final List<int> infantryGain1;
  final List<int> infantryGain2;
  final List<int> sentinelGain;
  final int sentinelPosture;
  
  RoboMasterInfo({
    required this.heroPosition,
    required this.engineerPosition,
    required this.infantryPosition1,
    required this.infantryPosition2,
    required this.dronePosition,
    required this.sentinelPosition,
    required this.heroBlood,
    required this.engineerBlood,
    required this.infantryBlood1,
    required this.infantryBlood2,
    required this.savenBlood,
    required this.sentinelBlood,
    required this.heroAmmunition,
    required this.infantryAmmunition1,
    required this.infantryAmmunition2,
    required this.droneAmmunition,
    required this.sentinelAmmunition,
    required this.economicRemain,
    required this.economicTotal,
    required this.occupationStatus,
    required this.heroGain,
    required this.engineerGain,
    required this.infantryGain1,
    required this.infantryGain2,
    required this.sentinelGain,
    required this.sentinelPosture,
  });
  
  factory RoboMasterInfo.defaultValues() {
    return RoboMasterInfo(
      heroPosition: [0, 0],
      engineerPosition: [0, 0],
      infantryPosition1: [0, 0],
      infantryPosition2: [0, 0],
      dronePosition: [0, 0],
      sentinelPosition: [0, 0],
      heroBlood: 0,
      engineerBlood: 0,
      infantryBlood1: 0,
      infantryBlood2: 0,
      savenBlood: 0,
      sentinelBlood: 0,
      heroAmmunition: 0,
      infantryAmmunition1: 0,
      infantryAmmunition2: 0,
      droneAmmunition: 0,
      sentinelAmmunition: 0,
      economicRemain: 0,
      economicTotal: 0,
      occupationStatus: [0, 0, 0, 0, 0, 0],
      heroGain: [0, 0, 0, 0, 0, 0, 0],
      engineerGain: [0, 0, 0, 0, 0, 0, 0],
      infantryGain1: [0, 0, 0, 0, 0, 0, 0],
      infantryGain2: [0, 0, 0, 0, 0, 0, 0],
      sentinelGain: [0, 0, 0, 0, 0, 0, 0],
      sentinelPosture: 0,
    );
  }
  
  static RoboMasterInfo? parse(List<int> data) {
    if (data.length < 90) return null;
    
    List<int> heroPosition = [0, 0];
    List<int> engineerPosition = [0, 0];
    List<int> infantryPosition1 = [0, 0];
    List<int> infantryPosition2 = [0, 0];
    List<int> dronePosition = [0, 0];
    List<int> sentinelPosition = [0, 0];
    int heroBlood = 0, engineerBlood = 0, infantryBlood1 = 0;
    int infantryBlood2 = 0, savenBlood = 0, sentinelBlood = 0;
    int heroAmmunition = 0, infantryAmmunition1 = 0, infantryAmmunition2 = 0;
    int droneAmmunition = 0, sentinelAmmunition = 0;
    int economicRemain = 0, economicTotal = 0;
    List<int> occupationStatus = [0, 0, 0, 0, 0, 0];
    List<int> heroGain = [0, 0, 0, 0, 0, 0, 0];
    List<int> engineerGain = [0, 0, 0, 0, 0, 0, 0];
    List<int> infantryGain1 = [0, 0, 0, 0, 0, 0, 0];
    List<int> infantryGain2 = [0, 0, 0, 0, 0, 0, 0];
    List<int> sentinelGain = [0, 0, 0, 0, 0, 0, 0];
    int sentinelPosture = 0;
    bool foundAny = false;
    
    for (int i = 0; i < data.length; i++) {
      if (i + 2 > data.length) break;
      int cmdId = (data[i] << 8) | data[i + 1];
      
      switch (cmdId) {
        case 0x0A01:
          if (i + 26 <= data.length) {
            heroPosition = [_readInt16BE(data, i + 2), _readInt16BE(data, i + 4)];
            engineerPosition = [_readInt16BE(data, i + 6), _readInt16BE(data, i + 8)];
            infantryPosition1 = [_readInt16BE(data, i + 10), _readInt16BE(data, i + 12)];
            infantryPosition2 = [_readInt16BE(data, i + 14), _readInt16BE(data, i + 16)];
            dronePosition = [_readInt16BE(data, i + 18), _readInt16BE(data, i + 20)];
            sentinelPosition = [_readInt16BE(data, i + 22), _readInt16BE(data, i + 24)];
            foundAny = true;
          }
          break;
        case 0x0A02:
          if (i + 14 <= data.length) {
            heroBlood = _readUint16BE(data, i + 2);
            engineerBlood = _readUint16BE(data, i + 4);
            infantryBlood1 = _readUint16BE(data, i + 6);
            infantryBlood2 = _readUint16BE(data, i + 8);
            savenBlood = _readUint16BE(data, i + 10);
            sentinelBlood = _readUint16BE(data, i + 12);
            foundAny = true;
          }
          break;
        case 0x0A03:
          if (i + 12 <= data.length) {
            heroAmmunition = _readUint16BE(data, i + 2);
            infantryAmmunition1 = _readUint16BE(data, i + 4);
            infantryAmmunition2 = _readUint16BE(data, i + 6);
            droneAmmunition = _readUint16BE(data, i + 8);
            sentinelAmmunition = _readUint16BE(data, i + 10);
            foundAny = true;
          }
          break;
        case 0x0A04:
          if (i + 12 <= data.length) {
            economicRemain = _readUint16BE(data, i + 2);
            economicTotal = _readUint16BE(data, i + 4);
            occupationStatus = data.sublist(i + 6, i + 12);
            foundAny = true;
          }
          break;
        case 0x0A05:
          if (i + 38 <= data.length) {
            heroGain = data.sublist(i + 2, i + 9);
            engineerGain = data.sublist(i + 9, i + 16);
            infantryGain1 = data.sublist(i + 16, i + 23);
            infantryGain2 = data.sublist(i + 23, i + 30);
            sentinelGain = data.sublist(i + 30, i + 37);
            sentinelPosture = data[i + 37];
            foundAny = true;
          }
          break;
      }
    }
    
    if (!foundAny) return null;
    
    return RoboMasterInfo(
      heroPosition: heroPosition,
      engineerPosition: engineerPosition,
      infantryPosition1: infantryPosition1,
      infantryPosition2: infantryPosition2,
      dronePosition: dronePosition,
      sentinelPosition: sentinelPosition,
      heroBlood: heroBlood,
      engineerBlood: engineerBlood,
      infantryBlood1: infantryBlood1,
      infantryBlood2: infantryBlood2,
      savenBlood: savenBlood,
      sentinelBlood: sentinelBlood,
      heroAmmunition: heroAmmunition,
      infantryAmmunition1: infantryAmmunition1,
      infantryAmmunition2: infantryAmmunition2,
      droneAmmunition: droneAmmunition,
      sentinelAmmunition: sentinelAmmunition,
      economicRemain: economicRemain,
      economicTotal: economicTotal,
      occupationStatus: occupationStatus,
      heroGain: heroGain,
      engineerGain: engineerGain,
      infantryGain1: infantryGain1,
      infantryGain2: infantryGain2,
      sentinelGain: sentinelGain,
      sentinelPosture: sentinelPosture,
    );
  }
  
  static int _readInt16BE(List<int> data, int offset) {
    return (data[offset] << 8) | data[offset + 1];
  }
  
  static int _readUint16BE(List<int> data, int offset) {
    return (data[offset] << 8) | data[offset + 1];
  }
  
  bool get isDefault {
    return heroPosition[0] == 0 && heroPosition[1] == 0 && heroBlood == 0 && heroAmmunition == 0;
  }
}
