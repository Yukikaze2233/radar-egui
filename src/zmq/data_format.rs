use serde::{Deserialize, Serialize};

// ── ZMQ message type constants ──

// PUB (Rust → C++/Python)
pub const ZMQ_PUB_GAME_STATE: u16 = 0x1001;
pub const ZMQ_PUB_RADAR_MARK: u16 = 0x1002;
pub const ZMQ_PUB_RADAR_SYNC: u16 = 0x1003;

// SUB (C++/Python → Rust)
pub const ZMQ_SUB_LIDAR_LOCATION: u16 = 0x2001;
pub const ZMQ_SUB_SDR: u16 = 0x2002;
pub const ZMQ_SUB_LASER: u16 = 0x2003;

#[derive(Debug, Serialize, Deserialize)]
pub struct ZmqMessageId {
    pub cmd_id: u16,
}
// ── PUB transmit (Rust → ZMQ → C++/Python) ──

/// Game state broadcast (ZMQ_PUB_GAME_STATE)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransmitGameState {
    pub cmd_id: u16,
    pub game_type: u8,
    pub game_progress: u8,
    pub stage_remain_time: u16,
    pub sync_timestamp: u64,
}

/// Radar mark progress broadcast (ZMQ_PUB_RADAR_MARK)
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

/// Radar autonomous decision sync broadcast (ZMQ_PUB_RADAR_SYNC)
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

/// SDR enemy robot position (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrPosition {
    pub hero_x: i16, pub hero_y: i16,
    pub engineer_x: i16, pub engineer_y: i16,
    pub infantry_3_x: i16, pub infantry_3_y: i16,
    pub infantry_4_x: i16, pub infantry_4_y: i16,
    pub aerial_x: i16, pub aerial_y: i16,
    pub sentry_x: i16, pub sentry_y: i16,
}

/// SDR enemy robot blood (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrBlood {
    pub hero_blood: u16, pub engineer_blood: u16,
    pub infantry_3_blood: u16, pub infantry_4_blood: u16,
    pub reserved: u16, pub sentry_blood: u16,
}

/// SDR enemy robot remaining ammo (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrAmmo {
    pub hero_ammo: u16, pub infantry_3_ammo: u16, pub infantry_4_ammo: u16,
    pub aerial_ammo: u16, pub sentry_ammo: u16,
}

/// SDR enemy robot overall state (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrState {
    pub remaining_gold: u16, pub total_gold: u16,
    pub supply_zone_status: u8, pub central_highland_status: u8,
    pub trapezoid_highland_status: u8, pub fortress_gain_status: u8,
    pub outpost_gain_status: u8, pub base_gain_status: u8,
    pub tunnel_1_status: u8, pub tunnel_2_status: u8,
    pub tunnel_3_status: u8, pub tunnel_4_status: u8,
    pub highland_upper_status: u8, pub ramp_rear_status: u8,
    pub road_upper_status: u8,
    pub occupation_status: [u8; 6],
}

/// SDR enemy robot gain (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrGain {
    pub hero_hp_recovery: u8, pub hero_cooling_acceleration: u16,
    pub hero_defence: u8, pub hero_negative_defence: u8, pub hero_attack: u16,
    pub engineer_hp_recovery: u8, pub engineer_cooling_acceleration: u16,
    pub engineer_defence: u8, pub engineer_negative_defence: u8, pub engineer_attack: u16,
    pub infantry_3_hp_recovery: u8, pub infantry_3_cooling_acceleration: u16,
    pub infantry_3_defence: u8, pub infantry_3_negative_defence: u8, pub infantry_3_attack: u16,
    pub infantry_4_hp_recovery: u8, pub infantry_4_cooling_acceleration: u16,
    pub infantry_4_defence: u8, pub infantry_4_negative_defence: u8, pub infantry_4_attack: u16,
    pub sentry_hp_recovery: u8, pub sentry_cooling_acceleration: u16,
    pub sentry_defence: u8, pub sentry_negative_defence: u8, pub sentry_attack: u16,
    pub sentry_posture: u8,
}

/// SDR jamming key (part of ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdrKey {
    pub key: [u8; 6],
}

/// Full SDR signal bundle from ZMQ SUB (ZMQ_SUB_SDR).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveSdr {
    pub cmd_id: u16,
    pub position: ReceiveSdrPosition,
    pub blood: ReceiveSdrBlood,
    pub ammo: ReceiveSdrAmmo,
    pub state: ReceiveSdrState,
    pub gain: ReceiveSdrGain,
    pub key: ReceiveSdrKey,
}

/// Laser observation data (model candidate).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveModelCandidate {
    pub score: f32,
    pub class_id: i32,
    pub bbox: [f32; 4],
    pub center: [f32; 2],
}

/// Laser guidance observation received via ZMQ SUB.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiveLaser {
    pub cmd_id: u16,
    pub detected: bool,
    pub center: [f32; 2],
    pub brightness: f32,
    pub contour: Vec<[f32; 2]>,
    pub candidates: Vec<ReceiveModelCandidate>,
}
