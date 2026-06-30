use super::robot_interaction_id::DeviceId;
use deku::prelude::*;
// ─── Command IDs (from DJI referee protocol) ───
//
// Byte order: little-endian
//   Multi-byte fields use from_le_bytes()
//   Bitfields: u8 with & mask >> shift

pub const FRAME_HEADER_SOF: u8 = 0xA5;
pub const FRAME_HEADER_LENGTH: usize = 5;
pub const CMD_ID_LENGTH: usize = 2;
pub const CRC8_LENGTH: usize = 1;
pub const CRC16_LENGTH: usize = 2;
pub const GAME_STATE_CMD_ID: u16 = 0x0001;
pub const GAME_RESULT_CMD_ID: u16 = 0x0002;
pub const SITE_EVENT_CMD_ID: u16 = 0x0101;
pub const DART_LAUNCH_CMD_ID: u16 = 0x0105;
pub const RADAR_MARK_PROCESS_CMD_ID: u16 = 0x020C;
pub const RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID: u16 = 0x020E;
pub const ROBOT_INTERACTION_CMD_ID: u16 = 0x0301;
pub const RADAR_AUTONOMOUS_DECISION_DATA_CMD_ID: u16 = 0x0121;
pub const MINIMAP_RECEIVE_RADAR_CMD_ID: u16 = 0x0305;
pub const SDR_ENEMY_ROBOT_POSITION_CMD_ID: u16 = 0x0A01;
pub const SDR_ENEMY_ROBOT_BLOOD_CMD_ID: u16 = 0x0A02;
pub const SDR_ENEMY_ROBOT_REMAINING_AMMO_CMD_ID: u16 = 0x0A03;
pub const SDR_ENEMY_ROBOT_OVERALL_STATE_CMD_ID: u16 = 0x0A04;
pub const SDR_ENEMY_ROBOT_GAIN_CMD_ID: u16 = 0x0A05;
pub const SDR_JAMMING_KEY_CMD_ID: u16 = 0x0A06;

// ─── Data lengths ───
pub const GAME_STATE_DATA_LEN: usize = 11;
pub const GAME_RESULT_DATA_LEN: usize = 1;
pub const SITE_EVENT_DATA_LEN: usize = 4;
pub const DART_LAUNCH_DATA_LEN: usize = 3;
pub const RADAR_MARK_PROCESS_DATA_LEN: usize = 2;
pub const RADAR_AUTONOMOUS_DECISION_SYNC_DATA_LEN: usize = 1;
pub const ROBOT_INTERACTION_DATA_LEN: usize = 118;
pub const MINIMAP_RECEIVE_RADAR_DATA_LEN: usize = 48;
pub const SDR_ENEMY_ROBOT_POSITION_DATA_LEN: usize = 24;
pub const SDR_ENEMY_ROBOT_BLOOD_DATA_LEN: usize = 12;
pub const SDR_ENEMY_ROBOT_REMAINING_AMMO_DATA_LEN: usize = 10;
pub const SDR_ENEMY_ROBOT_OVERALL_STATE_DATA_LEN: usize = 8;
pub const SDR_ENEMY_ROBOT_GAIN_DATA_LEN: usize = 36;
pub const SDR_JAMMING_KEY_DATA_LEN: usize = 6;

#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SerialFrameHeader {
    pub frame_header_sof: u8,
    pub frame_header_data_len: u16,
    pub frame_header_seq: u8,
    pub frame_header_crc8: u8,
}
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
pub struct SerialFrame {
    pub frame_header: SerialFrameHeader,
    pub cmd_id: u16,
    #[deku(count = "frame_header.frame_header_data_len as usize")]
    pub data: Vec<u8>,
    #[deku(endian = "little")]
    pub frame_crc16: u16,
}
// ─── Regular link protocol data (serial) ───
// Packet format: [cmd_id:2 LE] [data_len:1] [data:N]

// cmd_id = 0x0001, data_len = 11
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct GameStateData {
        /// data[0] bit 0-3  match type
    #[deku(bits = "4")]
    pub game_type: u8,
        /// data[0] bit 4-7  match phase
    #[deku(bits = "4")]
    pub game_progress: u8,
        /// data[1..3] u16 LE  stage remaining time (seconds)
    pub stage_remain_time: u16,
        /// data[3..11] u64 LE  UNIX timestamp
    pub sync_timestamp: u64,
}

// cmd_id = 0x0002, data_len = 1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct GameResultData {
    /// data[0]  winner
    pub winner: u8,
}

// cmd_id = 0x0101, data_len = 4
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct SiteEventData {
    /// event_data bit 0-2  supply zone occupation
    #[deku(bits = "3")]
    pub supply_zone_status: u8,
    /// event_data bit 3-4  small energy status (0=inactive 1=active 2=activating)
    #[deku(bits = "2")]
    pub energy_small_status: u8,
    /// event_data bit 5-6  large energy status (0=inactive 1=active 2=activating)
    #[deku(bits = "2")]
    pub energy_large_status: u8,
    /// event_data bit 7-8  central highland (1=ally 2=enemy)
    #[deku(bits = "2")]
    pub central_highland_status: u8,
    /// event_data bit 9-10  trapezoid highland (1=occupied)
    #[deku(bits = "2")]
    pub trapezoid_highland_status: u8,
    /// event_data bit 11-19  enemy dart last hit time on outpost/base (0-420s)
    #[deku(bits = "9")]
    pub dart_hit_time: u16,
    /// event_data bit 20-22  enemy dart last hit target (1=outpost 2=base_fixed 3=random_fixed 4=random_move 5=end_move)
    #[deku(bits = "3")]
    pub dart_hit_target: u8,
    /// event_data bit 23-24  center gain point (0=none 1=ally 2=enemy 3=both)
    #[deku(bits = "2")]
    pub center_gain_status: u8,
    /// event_data bit 25-26  fortress gain point (0=none 1=ally 2=enemy 3=both)
    #[deku(bits = "2")]
    pub fortress_gain_status: u8,
    /// event_data bit 27-28  outpost gain point (0=none 1=ally 2=enemy)
    #[deku(bits = "2")]
    pub outpost_gain_status: u8,
        /// event_data bit 29  base gain point (1=occupied)
    #[deku(bits = "1", pad_bits_after = "2")]
    pub base_gain_status: u8,
}

// cmd_id = 0x0105, data_len = 3   dart launch data
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct DartLaunchData {
    /// data[0]  dart launch remaining time (seconds)
    pub dart_remaining_time: u8,
    /// dart_info bit 0-2  last dart hit target (0=default 1=outpost 2=base_fixed 3=random_fixed 4=random_move 5=end_move)
    #[deku(bits = "3")]
    pub dart_hit_target: u8,
    /// dart_info bit 3-5  enemy target cumulative hit count (0-4)
    #[deku(bits = "3")]
    pub dart_hit_count: u8,
    /// dart_info bit 6-8  dart selected target (0=unselected/outpost 1=base_fixed 2=random_fixed 3=random_move 4=end_move)
    #[deku(bits = "3", pad_bits_after = "7")]
    pub dart_selected_target: u8,
}

// cmd_id = 0x020C, data_len = 2   radar mark progress data
// mark_progress bitfield: enemy marked >=100 -> 1; ally marked >=50 -> 1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct RadarMarkProcessData {
    /// mark_progress bit 0  opponent hero vulnerable
    #[deku(bits = "1")]
    pub opponent_hero_vulnerable: u8,
    /// mark_progress bit 1  opponent engineer vulnerable
    #[deku(bits = "1")]
    pub opponent_engineer_vulnerable: u8,
    /// mark_progress bit 2  opponent infantry 3 vulnerable
    #[deku(bits = "1")]
    pub opponent_infantry_3_vulnerable: u8,
    /// mark_progress bit 3  opponent infantry 4 vulnerable
    #[deku(bits = "1")]
    pub opponent_infantry_4_vulnerable: u8,
    /// mark_progress bit 4  opponent aerial marked
    #[deku(bits = "1")]
    pub opponent_aerial_marked: u8,
    /// mark_progress bit 5  opponent sentry vulnerable
    #[deku(bits = "1")]
    pub opponent_sentry_vulnerable: u8,
    /// mark_progress bit 6  ally hero marked
    #[deku(bits = "1")]
    pub ally_hero_marked: u8,
    /// mark_progress bit 7  ally engineer marked
    #[deku(bits = "1")]
    pub ally_engineer_marked: u8,
    /// mark_progress bit 8  ally infantry 3 marked
    #[deku(bits = "1")]
    pub ally_infantry_3_marked: u8,
    /// mark_progress bit 9  ally infantry 4 marked
    #[deku(bits = "1")]
    pub ally_infantry_4_marked: u8,
    /// mark_progress bit 10  ally aerial marked
    #[deku(bits = "1")]
    pub ally_aerial_marked: u8,
    /// mark_progress bit 11  ally sentry marked
    #[deku(bits = "1", pad_bits_after = "4")]
    pub ally_sentry_marked: u8,
}

// cmd_id = 0x020E, data_len = 1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct RadarAutonomousDecisionSyncData {
    /// radar_info bit 0-1  double weakness chance count (0-2)
    #[deku(bits = "2")]
    pub double_weakness_chance: u8,
    /// radar_info bit 2  double weakness active (0=no 1=yes)
    #[deku(bits = "1")]
    pub double_weakness_active: u8,
    /// radar_info bit 3-4  encryption rank (1-3)
    #[deku(bits = "2")]
    pub encryption_rank: u8,
    /// radar_info bit 5  key modifiable (1=yes)
    #[deku(bits = "1", pad_bits_after = "2")]
    pub key_modifiable: u8,
}

// cmd_id = 0x0301, data_len = 118
#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct RobotInteractionHeader {
    /// data[0..2] u16 LE  sub-content ID
    pub data_cmd_id: u16,
    /// data[2..4] u16 LE  sender ID
    pub sender_id: DeviceId,
    /// data[4..6] u16 LE  receiver ID
    pub receiver_id: DeviceId,
}

#[derive(Debug, Clone)]
pub struct RobotInteractionData {
    pub data_cmd_id: u16,
    pub sender_id: DeviceId,
    pub receiver_id: DeviceId,
    pub user_data: Vec<u8>,
}

impl Default for RobotInteractionData {
    fn default() -> Self {
        Self {
            data_cmd_id: 0,
            sender_id: DeviceId::Default,
            receiver_id: DeviceId::Default,
            user_data: Vec::new(),
        }
    }
}

// ─── Robot interaction sub-content (0x0301 sub cmd_id) ───

// sub-content cmd_id = 0x0121, 8 bytes
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct RadarAutonomousDecisionData {
    pub radar_cmd: u8,
    pub password_cmd: u8,
    pub password: [u8; 6],
}

// cmd_id = 0x0305, data_len = 48
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct MinimapReceiveRadarData {
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

// ─── SDR wireless link data (0x0A01–0x0A06) ───

// cmd_id = 0x0A01, data_len = 24
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotPositionData {
    /// data[0..2] i16 LE  enemy hero x (cm)
    pub hero_x: i16,
    /// data[2..4] i16 LE  enemy hero y (cm)
    pub hero_y: i16,
    /// data[4..6] i16 LE  enemy engineer x (cm)
    pub engineer_x: i16,
    /// data[6..8] i16 LE  enemy engineer y (cm)
    pub engineer_y: i16,
    /// data[8..10] i16 LE  enemy infantry 3 x (cm)
    pub infantry_3_x: i16,
    /// data[10..12] i16 LE  enemy infantry 3 y (cm)
    pub infantry_3_y: i16,
    /// data[12..14] i16 LE  enemy infantry 4 x (cm)
    pub infantry_4_x: i16,
    /// data[14..16] i16 LE  enemy infantry 4 y (cm)
    pub infantry_4_y: i16,
    /// data[16..18] i16 LE  enemy aerial x (cm)
    pub aerial_x: i16,
    /// data[18..20] i16 LE  enemy aerial y (cm)
    pub aerial_y: i16,
    /// data[20..22] i16 LE  enemy sentry x (cm)
    pub sentry_x: i16,
    /// data[22..24] i16 LE  enemy sentry y (cm)
    pub sentry_y: i16,
}

// cmd_id = 0x0A02, data_len = 12
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotBloodData {
    /// data[0..2] u16 LE  enemy hero blood
    pub hero_blood: u16,
    /// data[2..4] u16 LE  enemy engineer blood
    pub engineer_blood: u16,
    /// data[4..6] u16 LE  enemy infantry 3 blood
    pub infantry_3_blood: u16,
    /// data[6..8] u16 LE  enemy infantry 4 blood
    pub infantry_4_blood: u16,
    /// data[8..10] u16 LE  aerial (reserved)
    pub reserved: u16,
    /// data[10..12] u16 LE  enemy sentry blood
    pub sentry_blood: u16,
}

/// cmd_id = 0x0A03, data_len = 10
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotRemainingAmmoData {
    /// data[0..2] u16 LE  enemy hero ammo
    pub hero_ammo: u16,
    /// data[2..4] u16 LE  enemy infantry 3 ammo (incl. fortress reserve)
    pub infantry_3_ammo: u16,
    /// data[4..6] u16 LE  enemy infantry 4 ammo
    pub infantry_4_ammo: u16,
    /// data[6..8] u16 LE  enemy aerial ammo
    pub aerial_ammo: u16,
    /// data[8..10] u16 LE  enemy sentry ammo
    pub sentry_ammo: u16,
}

// cmd_id = 0x0A04, data_len = 8
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct SdrEnemyRobotOverallStateData {
    /// data[0..2] u16 LE  enemy remaining gold
    pub remaining_gold: u16,
    /// data[2..4] u16 LE  enemy total gold
    pub total_gold: u16,
    /// data[4] bit 0     enemy supply zone occupied
    #[deku(bits = "1")]
    pub supply_zone_status: u8,
    /// data[4] bit 1-2   enemy central highland (1=enemy 2=ally)
    #[deku(bits = "2")]
    pub central_highland_status: u8,
    /// data[4] bit 3     enemy trapezoid highland occupied
    #[deku(bits = "1")]
    pub trapezoid_highland_status: u8,
    /// data[4] bit 4-5   enemy fortress gain point (0=none 1=enemy 2=ally 3=both)
    #[deku(bits = "2")]
    pub fortress_gain_status: u8,
    /// data[4] bit 6-7   enemy outpost gain point (0=none 1=enemy 2=ally)
    #[deku(bits = "2")]
    pub outpost_gain_status: u8,
    /// data[5] bit 0     enemy base gain point occupied
    #[deku(bits = "1")]
    pub base_gain_status: u8,
    /// data[5] bit 1     tunnel 1 (near enemy ramp-front) detection
    #[deku(bits = "1")]
    pub tunnel_1_status: u8,
    /// data[5] bit 2     tunnel 2 (near enemy ramp-rear) detection
    #[deku(bits = "1")]
    pub tunnel_2_status: u8,
    /// data[5] bit 3     tunnel 3 (near ally ramp-front) detection
    #[deku(bits = "1")]
    pub tunnel_3_status: u8,
    /// data[5] bit 4     tunnel 4 (near ally ramp-rear) detection
    #[deku(bits = "1")]
    pub tunnel_4_status: u8,
    /// data[5] bit 5     highland upper detection
    #[deku(bits = "1")]
    pub highland_upper_status: u8,
    /// data[5] bit 6     ramp rear detection
    #[deku(bits = "1")]
    pub ramp_rear_status: u8,
    /// data[5] bit 7     road upper detection  (bytes 6-7 reserved)
    #[deku(bits = "1", pad_bits_after = "16")]
    pub road_upper_status: u8,
}

// cmd_id = 0x0A05, data_len = 36
// Per-robot gain: [hp_recovery(1) cooling(2 LE) defense(1) neg_defense(1) attack(2 LE)] = 7 bytes
// 5 robots + sentinel_posture(1) = 36 bytes
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotGainData {
    /// data[0]         hp recovery (percent)
    pub hero_hp_recovery: u8,
    /// data[1..3] u16 LE cooling acceleration (raw value)
    pub hero_cooling_acceleration: u16,
    /// data[3]         defense (percent)
    pub hero_defence: u8,
    /// data[4]         negative defense (percent)
    pub hero_negative_defence: u8,
    /// data[5..7] u16 LE attack (percent)
    pub hero_attack: u16,
    /// data[7]         hp recovery (percent)
    pub engineer_hp_recovery: u8,
    /// data[8..10] u16 LE cooling acceleration (raw value)
    pub engineer_cooling_acceleration: u16,
    /// data[10]        defense (percent)
    pub engineer_defence: u8,
    /// data[11]        negative defense (percent)
    pub engineer_negative_defence: u8,
    /// data[12..14] u16 LE attack (percent)
    pub engineer_attack: u16,
    /// data[14]        hp recovery (percent)
    pub infantry_3_hp_recovery: u8,
    /// data[15..17] u16 LE cooling acceleration (raw value)
    pub infantry_3_cooling_acceleration: u16,
    /// data[17]        defense (percent)
    pub infantry_3_defence: u8,
    /// data[18]        negative defense (percent)
    pub infantry_3_negative_defence: u8,
    /// data[19..21] u16 LE attack (percent)
    pub infantry_3_attack: u16,
    /// data[21]        hp recovery (percent)
    pub infantry_4_hp_recovery: u8,
    /// data[22..24] u16 LE cooling acceleration (raw value)
    pub infantry_4_cooling_acceleration: u16,
    /// data[24]        defense (percent)
    pub infantry_4_defence: u8,
    /// data[25]        negative defense (percent)
    pub infantry_4_negative_defence: u8,
    /// data[26..28] u16 LE attack (percent)
    pub infantry_4_attack: u16,
    /// data[28]        hp recovery (percent)
    pub sentry_hp_recovery: u8,
    /// data[29..31] u16 LE cooling acceleration (raw value)
    pub sentry_cooling_acceleration: u16,
    /// data[31]        defense (percent)
    pub sentry_defence: u8,
    /// data[32]        negative defense (percent)
    pub sentry_negative_defence: u8,
    /// data[33..35] u16 LE attack (percent)
    pub sentry_attack: u16,
    /// data[35]        sentry posture (1=attack 2=defense 3=move)
    pub sentry_posture: u8,
}

// cmd_id = 0x0A06, data_len = 6
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrJammingKeyData {
    /// data[0..6]  jamming key (ASCII alphanumeric)
    pub key: [u8; 6],
}

// Aggregate protocol data struct
#[derive(Debug, Clone, Default)]
pub struct SerialProtocolData {
    pub game_state_data: GameStateData,
    pub game_result_data: GameResultData,
    pub site_event_data: SiteEventData,
    pub dart_launch_data: DartLaunchData,
    pub radar_mark_process_data: RadarMarkProcessData,
    pub radar_autonomous_decision_sync_data: RadarAutonomousDecisionSyncData,
    pub robot_interaction_data: RobotInteractionData,
    pub radar_autonomous_decision_data: RadarAutonomousDecisionData,
    pub minimap_receive_radar_data: MinimapReceiveRadarData,
    pub sdr_enemy_robot_position_data: SdrEnemyRobotPositionData,
    pub sdr_enemy_robot_blood_data: SdrEnemyRobotBloodData,
    pub sdr_enemy_robot_remaining_ammo_data: SdrEnemyRobotRemainingAmmoData,
    pub sdr_enemy_robot_overall_state_data: SdrEnemyRobotOverallStateData,
    pub sdr_enemy_robot_gain_data: SdrEnemyRobotGainData,
    pub sdr_jamming_key_data: SdrJammingKeyData,
    pub serial_produced: [u8; 15],
    pub zmq_produced: [u8; 15],
}

pub const IDX_GAME_STATE: usize = 0;
pub const IDX_GAME_RESULT: usize = 1;
pub const IDX_SITE_EVENT: usize = 2;
pub const IDX_DART_LAUNCH: usize = 3;
pub const IDX_RADAR_MARK_PROCESS: usize = 4;
pub const IDX_RADAR_AUTONOMOUS_DECISION_SYNC: usize = 5;
pub const IDX_ROBOT_INTERACTION: usize = 6;
pub const IDX_RADAR_AUTONOMOUS_DECISION: usize = 7;
pub const IDX_MINIMAP_RECEIVE_RADAR: usize = 8;
pub const IDX_SDR_ENEMY_ROBOT_POSITION: usize = 9;
pub const IDX_SDR_ENEMY_ROBOT_BLOOD: usize = 10;
pub const IDX_SDR_ENEMY_ROBOT_REMAINING_AMMO: usize = 11;
pub const IDX_SDR_ENEMY_ROBOT_OVERALL_STATE: usize = 12;
pub const IDX_SDR_ENEMY_ROBOT_GAIN: usize = 13;
pub const IDX_SDR_JAMMING_KEY: usize = 14;
