import 'package:flutter/material.dart';
import '../models/robo_master_info.dart';
import 'blood_panel.dart';
import 'ammo_panel.dart';
import 'economy_panel.dart';
import 'occupation_panel.dart';
import 'gain_panel.dart';

class StatusPanels extends StatelessWidget {
  final RoboMasterInfo? info;
  const StatusPanels({Key? key, this.info}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    if (info == null) {
      return Center(child: Text('等待数据...'));
    }
    return ListView(
      padding: EdgeInsets.all(16),
      children: [
        BloodPanel(info: info!),
        SizedBox(height: 16),
        AmmoPanel(info: info!),
        SizedBox(height: 16),
        EconomyPanel(info: info!),
        SizedBox(height: 16),
        OccupationPanel(info: info!),
        SizedBox(height: 16),
        GainPanel(info: info!),
      ],
    );
  }
}
