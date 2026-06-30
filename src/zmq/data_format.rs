use serde::{Deserialize, Serialize};

// ── PUB transmit (Rust → ZMQ → C++/Python) ──

/// Game state broadcast (cmd 0x0001)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransmitGameState {
    pub cmd_id: u16,
    pub game_type: u8,
    pub game_progress: u8,
    pub stage_remain_time: u16,
    pub sync_timestamp: u64,
}

/// Radar mark progress broadcast (cmd 0x020C)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransmitRadarMarkProcess {
    pub cmd_id: u16,
    pub opponent_hero_vulnerable: u8,
    pub opponent_engineer_vulnerable: u8,
    pub opponent_infantry_3_vulnerable: u8,
    pub opponent_infantry_4_vulnerable: u8,
    pub opponent_aerial_marked: u8,
    pub opponent_sentry_vulnerable: u8,
    pub ally_hero_marked: u8,
    pub ally_engineer_marked: u8,
    pub ally_infantry_3_marked: u8,
    pub ally_infantry_4_marked: u8,
    pub ally_aerial_marked: u8,
    pub ally_sentry_marked: u8,
}

/// Radar autonomous decision sync broadcast (cmd 0x020E)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransmitRadarSync {
    pub cmd_id: u16,
    pub double_weakness_chance: u8,
    pub double_weakness_active: u8,
    pub encryption_rank: u8,
    pub key_modifiable: u8,
}

// ── SUB receive (C++/Python → ZMQ → Rust) ──

/// Lidar location data from external `alliance_radar_location_lidar`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveLidarLocation {
    pub cmd_id: u16,
    pub opponent_hero_x: u16,
    pub opponent_hero_y: u16,
    pub opponent_engineer_x: u16,
    pub opponent_engineer_y: u16,
    pub opponent_infantry_3_x: u16,
    pub opponent_infantry_3_y: u16,
    pub opponent_infantry_4_x: u16,
    pub opponent_infantry_4_y: u16,
    pub opponent_aerial_x: u16,
    pub opponent_aerial_y: u16,
    pub opponent_sentry_x: u16,
    pub opponent_sentry_y: u16,
    pub ally_hero_x: u16,
    pub ally_hero_y: u16,
    pub ally_engineer_x: u16,
    pub ally_engineer_y: u16,
    pub ally_infantry_3_x: u16,
    pub ally_infantry_3_y: u16,
    pub ally_infantry_4_x: u16,
    pub ally_infantry_4_y: u16,
    pub ally_aerial_x: u16,
    pub ally_aerial_y: u16,
    pub ally_sentry_x: u16,
    pub ally_sentry_y: u16,
}

/// Full SDR signal bundle from GNU Radio via ZMQ.
/// Combines fields from serial::data_format SDR structs:
///   SdrEnemyRobotPositionData   (0x0A01)
///   SdrEnemyRobotBloodData      (0x0A02)
///   SdrEnemyRobotRemainingAmmoData (0x0A03)
///   SdrEnemyRobotOverallStateData (0x0A04)
///   SdrEnemyRobotGainData       (0x0A05)
///   SdrJammingKeyData           (0x0A06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdr {
    pub cmd_id: u16,
    // Position (0x0A01) — 6 robots × i16 x/y
    pub hero_x: i16,
    pub hero_y: i16,
    pub engineer_x: i16,
    pub engineer_y: i16,
    pub infantry_3_x: i16,
    pub infantry_3_y: i16,
    pub infantry_4_x: i16,
    pub infantry_4_y: i16,
    pub aerial_x: i16,
    pub aerial_y: i16,
    pub sentry_x: i16,
    pub sentry_y: i16,

    // Blood (0x0A02) — 6 robots × u16
    pub hero_blood: u16,
    pub engineer_blood: u16,
    pub infantry_3_blood: u16,
    pub infantry_4_blood: u16,
    pub reserved: u16,
    pub sentry_blood: u16,

    // Ammunition (0x0A03) — 5 robots × u16
    pub hero_ammo: u16,
    pub infantry_3_ammo: u16,
    pub infantry_4_ammo: u16,
    pub aerial_ammo: u16,
    pub sentry_ammo: u16,

    // Economy & state (0x0A04)
    pub remaining_gold: u16,
    pub total_gold: u16,
    pub supply_zone_status: u8,
    pub central_highland_status: u8,
    pub trapezoid_highland_status: u8,
    pub fortress_gain_status: u8,
    pub outpost_gain_status: u8,
    pub base_gain_status: u8,
    pub tunnel_1_status: u8,
    pub tunnel_2_status: u8,
    pub tunnel_3_status: u8,
    pub tunnel_4_status: u8,
    pub highland_upper_status: u8,
    pub ramp_rear_status: u8,
    pub road_upper_status: u8,

    // Gains (0x0A05) — 5 robots × gain fields
    pub hero_hp_recovery: u8,
    pub hero_cooling_acceleration: u16,
    pub hero_defence: u8,
    pub hero_negative_defence: u8,
    pub hero_attack: u16,
    pub engineer_hp_recovery: u8,
    pub engineer_cooling_acceleration: u16,
    pub engineer_defence: u8,
    pub engineer_negative_defence: u8,
    pub engineer_attack: u16,
    pub infantry_3_hp_recovery: u8,
    pub infantry_3_cooling_acceleration: u16,
    pub infantry_3_defence: u8,
    pub infantry_3_negative_defence: u8,
    pub infantry_3_attack: u16,
    pub infantry_4_hp_recovery: u8,
    pub infantry_4_cooling_acceleration: u16,
    pub infantry_4_defence: u8,
    pub infantry_4_negative_defence: u8,
    pub infantry_4_attack: u16,
    pub sentry_hp_recovery: u8,
    pub sentry_cooling_acceleration: u16,
    pub sentry_defence: u8,
    pub sentry_negative_defence: u8,
    pub sentry_attack: u16,
    pub sentry_posture: u8,

    // Jamming key (0x0A06)
    pub key: [u8; 6],
}
