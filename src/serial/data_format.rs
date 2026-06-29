use super::robot_interaction_id::DeviceId;
use deku::prelude::*;
// ─── 命令ID (C++ #define 的 Rust 版) ───
//
// 字节序: 全部 LE (小端序)
//   多字节字段统一用 from_le_bytes() 解析
//   位域: u8 直接用 & mask >> shift

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
pub const MINIMAP_RECEIVE_RADAR_CMD_ID: u16 = 0x0305;
pub const SDR_ENEMY_ROBOT_POSITION_CMD_ID: u16 = 0x0A01;
pub const SDR_ENEMY_ROBOT_BLOOD_CMD_ID: u16 = 0x0A02;
pub const SDR_ENEMY_ROBOT_REMAINING_AMMO_CMD_ID: u16 = 0x0A03;
pub const SDR_ENEMY_ROBOT_OVERALL_STATE_CMD_ID: u16 = 0x0A04;
pub const SDR_ENEMY_ROBOT_GAIN_CMD_ID: u16 = 0x0A05;
pub const SDR_JAMMING_KEY_CMD_ID: u16 = 0x0A06;

// ─── 命令ID ───
pub const GAME_STATE_DATA_LEN: usize = 11;
pub const GAME_RESULT_DATA_LEN: usize = 1;
pub const SITE_EVENT_DATA_LEN: usize = 4;
pub const DART_LAUNCH_DATA_LEN: usize = 3;
pub const RADAR_MARK_PROCESS_DATA_LEN: usize = 2;
pub const RADAR_AUTONOMOUS_DECISION_SYNC_DATA_LEN: usize = 1;
pub const ROBOT_INTERACTION_DATA_LEN: usize = 118;
pub const RADAR_AUTONOMOUS_DECISION_DATA_CMD_ID: u16 = 0x0121;
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
// ─── 比赛状态与事件协议数据 (常规链路) ───
// 包格式: [cmd_id:2 LE] [data_len:1] [data:N]

// cmd_id = 0x0001, data_len = 11
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct GameStateData {
    /// data[0] bit 0-3  比赛类型
    #[deku(bits = "4")]
    pub game_type: u8,
    /// data[0] bit 4-7  当前比赛阶段
    #[deku(bits = "4")]
    pub game_progress: u8,
    /// data[1..3] u16 LE  当前阶段剩余时间(秒)
    pub stage_remain_time: u16,
    /// data[3..11] u64 LE  UNIX时间戳
    pub sync_timestamp: u64,
}

// cmd_id = 0x0002, data_len = 1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct GameResultData {
    /// data[0]  获胜方
    pub winner: u8,
}

// cmd_id = 0x0101, data_len = 4
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct SiteEventData {
    /// event_data bit 0-2  己方补给区占领状态
    #[deku(bits = "3")]
    pub supply_zone_status: u8,
    /// event_data bit 3-4  己方小能量机关状态 (0=未激活 1=已激活 2=正在激活)
    #[deku(bits = "2")]
    pub energy_small_status: u8,
    /// event_data bit 5-6  己方大能量机关状态 (0=未激活 1=已激活 2=正在激活)
    #[deku(bits = "2")]
    pub energy_large_status: u8,
    /// event_data bit 7-8  己方中央高地占领状态 (1=己方 2=对方)
    #[deku(bits = "2")]
    pub central_highland_status: u8,
    /// event_data bit 9-10  己方梯形高地占领状态 (1=已占领)
    #[deku(bits = "2")]
    pub trapezoid_highland_status: u8,
    /// event_data bit 11-19  对方飞镖最后击中己方前哨站/基地时间 (0-420秒)
    #[deku(bits = "9")]
    pub dart_hit_time: u16,
    /// event_data bit 20-22  对方飞镖最后击中目标 (1=前哨站 2=基地固定 3=随机固定 4=随机移动 5=末端移动)
    #[deku(bits = "3")]
    pub dart_hit_target: u8,
    /// event_data bit 23-24  中心增益点占领状态(RMUL) (0=未占 1=己方 2=对方 3=双方)
    #[deku(bits = "2")]
    pub center_gain_status: u8,
    /// event_data bit 25-26  己方堡垒增益点占领状态 (0=未占 1=己方 2=对方 3=双方)
    #[deku(bits = "2")]
    pub fortress_gain_status: u8,
    /// event_data bit 27-28  己方前哨站增益点占领状态 (0=未占 1=己方 2=对方)
    #[deku(bits = "2")]
    pub outpost_gain_status: u8,
    /// event_data bit 29  己方基地增益点占领状态 (1=已占领)
    #[deku(bits = "1", pad_bits_after = "2")]
    pub base_gain_status: u8,
}

// cmd_id = 0x0105, data_len = 3   飞镖发射相关数据
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct DartLaunchData {
    /// data[0]  己方飞镖发射剩余时间(秒)
    pub dart_remaining_time: u8,
    /// dart_info bit 0-2  最近一次飞镖击中目标 (0=默认 1=前哨站 2=基地固定 3=随机固定 4=随机移动 5=末端移动)
    #[deku(bits = "3")]
    pub dart_hit_target: u8,
    /// dart_info bit 3-5  对方目标累计被击中次数 (0-4)
    #[deku(bits = "3")]
    pub dart_hit_count: u8,
    /// dart_info bit 6-8  飞镖选定打击目标 (0=未选定/前哨站 1=基地固定 2=随机固定 3=随机移动 4=末端移动)
    #[deku(bits = "3", pad_bits_after = "7")]
    pub dart_selected_target: u8,
}

// cmd_id = 0x020C, data_len = 2   雷达标记进度数据
// mark_progress 位域: 对方机器人被标记进度≥100 时为1; 己方机器人被标记进度≥50 时为1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct RadarMarkProcessData {
    /// mark_progress bit 0  对方英雄机器人易伤情况
    #[deku(bits = "1")]
    pub opponent_hero_vulnerable: u8,
    /// mark_progress bit 1  对方工程机器人易伤情况
    #[deku(bits = "1")]
    pub opponent_engineer_vulnerable: u8,
    /// mark_progress bit 2  对方3号步兵机器人易伤情况
    #[deku(bits = "1")]
    pub opponent_infantry_3_vulnerable: u8,
    /// mark_progress bit 3  对方4号步兵机器人易伤情况
    #[deku(bits = "1")]
    pub opponent_infantry_4_vulnerable: u8,
    /// mark_progress bit 4  对方空中机器人特殊标识情况
    #[deku(bits = "1")]
    pub opponent_aerial_marked: u8,
    /// mark_progress bit 5  对方哨兵机器人易伤情况
    #[deku(bits = "1")]
    pub opponent_sentry_vulnerable: u8,
    /// mark_progress bit 6  己方英雄机器人特殊标识情况
    #[deku(bits = "1")]
    pub ally_hero_marked: u8,
    /// mark_progress bit 7  己方工程机器人特殊标识情况
    #[deku(bits = "1")]
    pub ally_engineer_marked: u8,
    /// mark_progress bit 8  己方3号步兵机器人特殊标识情况
    #[deku(bits = "1")]
    pub ally_infantry_3_marked: u8,
    /// mark_progress bit 9  己方4号步兵机器人特殊标识情况
    #[deku(bits = "1")]
    pub ally_infantry_4_marked: u8,
    /// mark_progress bit 10  己方空中机器人特殊标识情况
    #[deku(bits = "1")]
    pub ally_aerial_marked: u8,
    /// mark_progress bit 11  己方哨兵机器人特殊标识情况
    #[deku(bits = "1", pad_bits_after = "4")]
    pub ally_sentry_marked: u8,
}

// cmd_id = 0x020E, data_len = 1
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct RadarAutonomousDecisionSyncData {
    /// radar_info bit 0-1  双倍易伤机会次数 (0-2)
    #[deku(bits = "2")]
    pub double_weakness_chance: u8,
    /// radar_info bit 2  对方是否正在被触发双倍易伤 (0=否 1=是)
    #[deku(bits = "1")]
    pub double_weakness_active: u8,
    /// radar_info bit 3-4  己方加密等级/干扰波难度 (1-3)
    #[deku(bits = "2")]
    pub encryption_level: u8,
    /// radar_info bit 5  当前是否可修改密钥 (1=可修改)
    #[deku(bits = "1", pad_bits_after = "2")]
    pub key_modifiable: u8,
}

// cmd_id = 0x0301, data_len = 118
#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct RobotInteractionHeader {
    /// data[0..2] u16 LE  子内容ID
    pub data_cmd_id: u16,
    /// data[2..4] u16 LE  发送者ID
    pub sender_id: DeviceId,
    /// data[4..6] u16 LE  接收者ID
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

// ─── 机器人交互子内容 (0x0301 的子内容 ID) ───

// 子内容 cmd_id = 0x0121, 8 bytes
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

// ─── 雷达无线链路数据 (SDR) ───

// cmd_id = 0x0A01, data_len = 24
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotPositionData {
    /// data[0..2] i16 LE  对方英雄 x (cm)
    pub hero_x: i16,
    /// data[2..4] i16 LE  对方英雄 y (cm)
    pub hero_y: i16,
    /// data[4..6] i16 LE  对方工程 x (cm)
    pub engineer_x: i16,
    /// data[6..8] i16 LE  对方工程 y (cm)
    pub engineer_y: i16,
    /// data[8..10] i16 LE 对方3号步兵 x (cm)
    pub infantry_3_x: i16,
    /// data[10..12] i16 LE 对方3号步兵 y (cm)
    pub infantry_3_y: i16,
    /// data[12..14] i16 LE 对方4号步兵 x (cm)
    pub infantry_4_x: i16,
    /// data[14..16] i16 LE 对方4号步兵 y (cm)
    pub infantry_4_y: i16,
    /// data[16..18] i16 LE 对方空中 x (cm)
    pub aerial_x: i16,
    /// data[18..20] i16 LE 对方空中 y (cm)
    pub aerial_y: i16,
    /// data[20..22] i16 LE 对方哨兵 x (cm)
    pub sentry_x: i16,
    /// data[22..24] i16 LE 对方哨兵 y (cm)
    pub sentry_y: i16,
}

// cmd_id = 0x0A02, data_len = 12
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotBloodData {
    /// data[0..2] u16 LE  对方英雄血量
    pub hero_blood: u16,
    /// data[2..4] u16 LE  对方工程血量
    pub engineer_blood: u16,
    /// data[4..6] u16 LE  对方3号步兵血量
    pub infantry_3_blood: u16,
    /// data[6..8] u16 LE  对方4号步兵血量
    pub infantry_4_blood: u16,
    /// data[8..10] u16 LE 空中机器人(保留)
    pub reserved: u16,
    /// data[10..12] u16 LE 对方哨兵血量
    pub sentry_blood: u16,
}

/// cmd_id = 0x0A03, data_len = 10
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotRemainingAmmoData {
    /// data[0..2] u16 LE  对方英雄允许发弹量
    pub hero_ammo: u16,
    /// data[2..4] u16 LE  对方3号步兵允许发弹量 (含堡垒储备)
    pub infantry_3_ammo: u16,
    /// data[4..6] u16 LE  对方4号步兵允许发弹量
    pub infantry_4_ammo: u16,
    /// data[6..8] u16 LE  对方空中允许发弹量
    pub aerial_ammo: u16,
    /// data[8..10] u16 LE 对方哨兵允许发弹量
    pub sentry_ammo: u16,
}

// cmd_id = 0x0A04, data_len = 8
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little", bit_order = "lsb")]
pub struct SdrEnemyRobotOverallStateData {
    /// data[0..2] u16 LE  对方剩余金币数
    pub remaining_gold: u16,
    /// data[2..4] u16 LE  对方累计总金币数
    pub total_gold: u16,
    /// data[4] bit 0     对方补给区占领状态
    #[deku(bits = "1")]
    pub supply_zone_status: u8,
    /// data[4] bit 1-2   对方中央高地占领状态 (1=对方 2=己方)
    #[deku(bits = "2")]
    pub central_highland_status: u8,
    /// data[4] bit 3     对方梯形高地占领状态 (1=已占领)
    #[deku(bits = "1")]
    pub trapezoid_highland_status: u8,
    /// data[4] bit 4-5   对方堡垒增益点占领状态 (0=未占 1=对方 2=己方 3=双方)
    #[deku(bits = "2")]
    pub fortress_gain_status: u8,
    /// data[4] bit 6-7   对方前哨站增益点占领状态 (0=未占 1=对方 2=己方)
    #[deku(bits = "2")]
    pub outpost_gain_status: u8,
    /// data[5] bit 0     对方基地增益点占领状态 (1=已占领)
    #[deku(bits = "1")]
    pub base_gain_status: u8,
    /// data[5] bit 1     隧道1 (靠近对方飞坡前) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub tunnel_1_status: u8,
    /// data[5] bit 2     隧道2 (靠近对方飞坡后) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub tunnel_2_status: u8,
    /// data[5] bit 3     隧道3 (靠近己方飞坡前) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub tunnel_3_status: u8,
    /// data[5] bit 4     隧道4 (靠近己方飞坡后) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub tunnel_4_status: u8,
    /// data[5] bit 5     高地(上部) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub highland_upper_status: u8,
    /// data[5] bit 6     飞坡(后部) 对方机器人检测 (1=检测到)
    #[deku(bits = "1")]
    pub ramp_rear_status: u8,
    /// data[5] bit 7     公路(上部) 对方机器人检测 (1=检测到)  (bytes 6-7 保留)
    #[deku(bits = "1", pad_bits_after = "16")]
    pub road_upper_status: u8,
}

// cmd_id = 0x0A05, data_len = 36
// 每机器人增益: [hp_recovery(1) cooling(2 LE) defense(1) neg_defense(1) attack(2 LE)] = 7字节
// 5机器人 + sentinel_posture(1) = 36 字节
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrEnemyRobotGainData {
    /// data[0]         回血增益 (百分比)
    pub hero_hp_recovery: u8,
    /// data[1..3] u16 LE 射击热量冷却增益 (直接值)
    pub hero_cooling_acceleration: u16,
    /// data[3]         防御增益 (百分比)
    pub hero_defence: u8,
    /// data[4]         负防御增益 (百分比)
    pub hero_negative_defence: u8,
    /// data[5..7] u16 LE 攻击增益 (百分比)
    pub hero_attack: u16,
    /// data[7]         回血增益 (百分比)
    pub engineer_hp_recovery: u8,
    /// data[8..10] u16 LE 射击热量冷却增益 (直接值)
    pub engineer_cooling_acceleration: u16,
    /// data[10]        防御增益 (百分比)
    pub engineer_defence: u8,
    /// data[11]        负防御增益 (百分比)
    pub engineer_negative_defence: u8,
    /// data[12..14] u16 LE 攻击增益 (百分比)
    pub engineer_attack: u16,
    /// data[14]        回血增益 (百分比)
    pub infantry_3_hp_recovery: u8,
    /// data[15..17] u16 LE 射击热量冷却增益 (直接值)
    pub infantry_3_cooling_acceleration: u16,
    /// data[17]        防御增益 (百分比)
    pub infantry_3_defence: u8,
    /// data[18]        负防御增益 (百分比)
    pub infantry_3_negative_defence: u8,
    /// data[19..21] u16 LE 攻击增益 (百分比)
    pub infantry_3_attack: u16,
    /// data[21]        回血增益 (百分比)
    pub infantry_4_hp_recovery: u8,
    /// data[22..24] u16 LE 射击热量冷却增益 (直接值)
    pub infantry_4_cooling_acceleration: u16,
    /// data[24]        防御增益 (百分比)
    pub infantry_4_defence: u8,
    /// data[25]        负防御增益 (百分比)
    pub infantry_4_negative_defence: u8,
    /// data[26..28] u16 LE 攻击增益 (百分比)
    pub infantry_4_attack: u16,
    /// data[28]        回血增益 (百分比)
    pub sentry_hp_recovery: u8,
    /// data[29..31] u16 LE 射击热量冷却增益 (直接值)
    pub sentry_cooling_acceleration: u16,
    /// data[31]        防御增益 (百分比)
    pub sentry_defence: u8,
    /// data[32]        负防御增益 (百分比)
    pub sentry_negative_defence: u8,
    /// data[33..35] u16 LE 攻击增益 (百分比)
    pub sentry_attack: u16,
    /// data[35]        哨兵姿态 (1=进攻 2=防御 3=移动)
    pub sentry_posture: u8,
}

// cmd_id = 0x0A06, data_len = 6
#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct SdrJammingKeyData {
    /// data[0..6]  干扰密钥 (ASCII编码 字母或数字)
    pub key: [u8; 6],
}

// 总协议数据结构
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
