use std::str;
use std::bytes::Bytes;

// ─── 比赛状态与事件协议数据 (常规链路) ───

pub struct GameStateData {
    pub cmd_id: u16 = 0x0001,
    pub data_length: u8 = 11,
    pub data: [u8; data_length],
    pub game_type: u8,          // data[0] bit 0-3   比赛类型
    pub game_progress: u8,      // data[0] bit 4-7   当前比赛阶段
    pub stage_remain_time: u16, // data[1..3] u16 BE  当前阶段剩余时间(秒)
    pub sync_timestamp: u64,    // data[3..11] u64 BE  UNIX时间戳
}

pub struct GameResultData {
    pub cmd_id: u16 = 0x0002,
    pub data_length: u8 = 1,
    pub data: [u8; data_length],
    pub winner: u8,             // data[0]  获胜方
}

pub struct SiteEventData {
    pub cmd_id: u16 = 0x0101,
    pub data_length: u8 = 4,
    pub data: [u8; data_length],
    pub supply_zone_status: u8,         // data[0] bit 0-2   补给区占领状态
    pub energy_small_status: u8,        // data[0] bit 3-4   小能量机关状态 (0=未激活 1=已激活 2=正在激活)
    pub energy_large_status: u8,        // data[0] bit 5-6   大能量机关状态 (0=未激活 1=已激活 2=正在激活)
    pub central_highland_status: u8,    // data[0..1] bit 7-8  中央高地占领状态 (1=己方 2=对方)
    pub trapezoid_highland_status: u8,  // data[1] bit 1-2   梯形高地占领状态 (1=已占领)
    pub dart_hit_time: u16,             // data[1..3] bit 3-11 飞镖击中己方前哨站/基地时间 (0-420秒)
    pub dart_hit_target: u8,            // data[2..3] bit 4-6  飞镖击中目标 (1=前哨站 2=基地固定 3=随机固定 4=随机移动 5=末端移动)
    pub center_gain_status: u8,         // data[2..3] bit 7-8  中心增益点占领状态(RMUL) (0=未占 1=己方 2=对方 3=双方)
    pub fortress_gain_status: u8,       // data[3] bit 1-2   堡垒增益点占领状态 (0=未占 1=己方 2=对方 3=双方)
    pub outpost_gain_status: u8,        // data[3] bit 3-4   前哨站增益点占领状态 (0=未占 1=己方 2=对方)
    pub base_gain_status: u8,           // data[3] bit 5     基地增益点占领状态 (1=已占领)
}

pub struct DartLaunchData {
    pub cmd_id: u16 = 0x0104,
    pub data_length: u8 = 3,
    pub data: [u8; data_length],
    pub level: u8,              // data[0]  判罚等级 (1=双方黄牌 2=黄牌 3=红牌 4=判负)
    pub offending_robot_id: u8, // data[1]  违规机器人ID (判负/双方黄牌时为0)
    pub count: u8,              // data[2]  对应等级违规次数 (开局为0)
}

pub struct RadarMarkProcessData {
    pub cmd_id: u16 = 0x0105,
    pub data_length: u8 = 3,
    pub data: [u8; data_length],
    pub dart_remaining_time: u8,  // data[0]  飞镖发射剩余时间(秒)
    pub dart_hit_target: u8,      // data[1..2] bit 0-2  最近一次飞镖击中目标 (1=前哨站 2=基地固定 3=随机固定 4=随机移动 5=末端移动)
    pub dart_hit_count: u8,       // data[1..2] bit 3-5  对方目标累计被击中次数 (0-4)
    pub dart_selected_target: u8, // data[1..2] bit 6-8  飞镖选定打击目标 (0=未选定/前哨站 1=基地固定 2=随机固定 3=随机移动 4=末端移动)
}

pub struct RadarAutonomousDecisionSyncData {
    pub cmd_id: u16 = 0x020E,
    pub data_length: u8 = 1,
    pub data: [u8; data_length],
    pub double_weakness_chance: u8, // data[0] bit 0-1  双倍易伤机会次数 (0-2)
    pub double_weakness_active: u8, // data[0] bit 2    对方是否正在被触发双倍易伤 (0=否 1=是)
    pub encryption_level: u8,       // data[0] bit 3-4  己方加密等级/干扰波难度 (1-3)
    pub key_modifiable: u8,         // data[0] bit 5    当前是否可修改密钥 (1=可修改)
}

pub struct RobotInteractionData {
    pub cmd_id: u16 = 0x0301,
    pub data_length: u8 = 118,
    pub data: [u8; data_length],
    pub data_cmd_id: u16,   // data[0..2] u16 BE  子内容ID
    pub sender_id: u16,     // data[2..4] u16 BE  发送者ID
    pub receiver_id: u16,   // data[4..6] u16 BE  接收者ID
    pub user_data: [u8; 112], // data[6..118]     内容数据段 (最大112字节)
}

pub struct MinimapReceiveRadarData {
    pub cmd_id: u16 = 0x0305,
    pub data_length: u8 = 48,
    pub data: [u8; data_length],
    // 对方机器人坐标
    pub opponent_hero_x: i16,       // data[0..2] i16 BE  对方英雄 x (cm)
    pub opponent_hero_y: i16,       // data[2..4] i16 BE  对方英雄 y (cm)
    pub opponent_engineer_x: i16,   // data[4..6] i16 BE  对方工程 x (cm)
    pub opponent_engineer_y: i16,   // data[6..8] i16 BE  对方工程 y (cm)
    pub opponent_infantry_3_x: i16, // data[8..10] i16 BE 对方3号步兵 x (cm)
    pub opponent_infantry_3_y: i16, // data[10..12] i16 BE 对方3号步兵 y (cm)
    pub opponent_infantry_4_x: i16, // data[12..14] i16 BE 对方4号步兵 x (cm)
    pub opponent_infantry_4_y: i16, // data[14..16] i16 BE 对方4号步兵 y (cm)
    pub opponent_aerial_x: i16,     // data[16..18] i16 BE 对方空中 x (cm)
    pub opponent_aerial_y: i16,     // data[18..20] i16 BE 对方空中 y (cm)
    pub opponent_sentry_x: i16,     // data[20..22] i16 BE 对方哨兵 x (cm)
    pub opponent_sentry_y: i16,     // data[22..24] i16 BE 对方哨兵 y (cm)
    // 己方机器人坐标
    pub ally_hero_x: i16,           // data[24..26] i16 BE 己方英雄 x (cm)
    pub ally_hero_y: i16,           // data[26..28] i16 BE 己方英雄 y (cm)
    pub ally_engineer_x: i16,       // data[28..30] i16 BE 己方工程 x (cm)
    pub ally_engineer_y: i16,       // data[30..32] i16 BE 己方工程 y (cm)
    pub ally_infantry_3_x: i16,     // data[32..34] i16 BE 己方3号步兵 x (cm)
    pub ally_infantry_3_y: i16,     // data[34..36] i16 BE 己方3号步兵 y (cm)
    pub ally_infantry_4_x: i16,     // data[36..38] i16 BE 己方4号步兵 x (cm)
    pub ally_infantry_4_y: i16,     // data[38..40] i16 BE 己方4号步兵 y (cm)
    pub ally_aerial_x: i16,         // data[40..42] i16 BE 己方空中 x (cm)
    pub ally_aerial_y: i16,         // data[42..44] i16 BE 己方空中 y (cm)
    pub ally_sentry_x: i16,         // data[44..46] i16 BE 己方哨兵 x (cm)
    pub ally_sentry_y: i16,         // data[46..48] i16 BE 己方哨兵 y (cm)
}

// ─── 雷达无线链路数据 (SDR) ───

pub struct SDR_EnemyRobotPositionData {
    pub cmd_id: u16 = 0x0A01,
    pub data_length: u8 = 24,
    pub data: [u8; data_length],
    pub hero_x: i16,        // data[0..2] i16 BE  对方英雄 x (cm)
    pub hero_y: i16,        // data[2..4] i16 BE  对方英雄 y (cm)
    pub engineer_x: i16,    // data[4..6] i16 BE  对方工程 x (cm)
    pub engineer_y: i16,    // data[6..8] i16 BE  对方工程 y (cm)
    pub infantry_3_x: i16,  // data[8..10] i16 BE 对方3号步兵 x (cm)
    pub infantry_3_y: i16,  // data[10..12] i16 BE 对方3号步兵 y (cm)
    pub infantry_4_x: i16,  // data[12..14] i16 BE 对方4号步兵 x (cm)
    pub infantry_4_y: i16,  // data[14..16] i16 BE 对方4号步兵 y (cm)
    pub aerial_x: i16,      // data[16..18] i16 BE 对方空中 x (cm)
    pub aerial_y: i16,      // data[18..20] i16 BE 对方空中 y (cm)
    pub sentry_x: i16,      // data[20..22] i16 BE 对方哨兵 x (cm)
    pub sentry_y: i16,      // data[22..24] i16 BE 对方哨兵 y (cm)
}

pub struct SDR_EnemyRobotBloodData {
    pub cmd_id: u16 = 0x0A02,
    pub data_length: u8 = 12,
    pub data: [u8; data_length],
    pub hero_blood: u16,        // data[0..2] u16 BE  对方英雄血量
    pub engineer_blood: u16,    // data[2..4] u16 BE  对方工程血量
    pub infantry_3_blood: u16,  // data[4..6] u16 BE  对方3号步兵血量
    pub infantry_4_blood: u16,  // data[6..8] u16 BE  对方4号步兵血量
    pub reserved: u16,          // data[8..10] u16 BE 空中机器人(保留)
    pub sentry_blood: u16,      // data[10..12] u16 BE 对方哨兵血量
}

pub struct SDR_EnemyRobotRemainingAmmoData {
    pub cmd_id: u16 = 0x0A03,
    pub data_length: u8 = 10,
    pub data: [u8; data_length],
    pub hero_ammo: u16,        // data[0..2] u16 BE  对方英雄允许发弹量
    pub infantry_3_ammo: u16,  // data[2..4] u16 BE  对方3号步兵允许发弹量 (含堡垒储备)
    pub infantry_4_ammo: u16,  // data[4..6] u16 BE  对方4号步兵允许发弹量
    pub aerial_ammo: u16,      // data[6..8] u16 BE  对方空中允许发弹量
    pub sentry_ammo: u16,      // data[8..10] u16 BE 对方哨兵允许发弹量
}

pub struct SDR_EnemyRobotOverallStateData {
    pub cmd_id: u16 = 0x0A04,
    pub data_length: u8 = 8,
    pub data: [u8; data_length],
    pub remaining_gold: u16,         // data[0..2] u16 BE  对方剩余金币数
    pub total_gold: u16,             // data[2..4] u16 BE  对方累计总金币数
    pub supply_zone_status: u8,      // data[4] bit 0     对方补给区占领状态
    pub central_highland_status: u8, // data[4] bit 1-2   对方中央高地占领状态 (1=对方 2=己方)
    pub trapezoid_highland_status: u8, // data[4] bit 3   对方梯形高地占领状态 (1=已占领)
    pub fortress_gain_status: u8,    // data[4] bit 4-5   对方堡垒增益点占领状态 (0=未占 1=对方 2=己方 3=双方)
    pub outpost_gain_status: u8,     // data[4] bit 6-7   对方前哨站增益点占领状态 (0=未占 1=对方 2=己方)
    pub base_gain_status: u8,        // data[5] bit 0     对方基地增益点占领状态 (1=已占领)
    pub tunnel_1_status: u8,         // data[5] bit 1     隧道1 (靠近对方飞坡前) 对方机器人检测 (1=检测到)
    pub tunnel_2_status: u8,         // data[5] bit 2     隧道2 (靠近对方飞坡后) 对方机器人检测 (1=检测到)
    pub tunnel_3_status: u8,         // data[5] bit 3     隧道3 (靠近己方飞坡前) 对方机器人检测 (1=检测到)
    pub tunnel_4_status: u8,         // data[5] bit 4     隧道4 (靠近己方飞坡后) 对方机器人检测 (1=检测到)
    pub highland_upper_status: u8,   // data[5] bit 5     高地(上部) 对方机器人检测 (1=检测到)
    pub ramp_rear_status: u8,        // data[5] bit 6     飞坡(后部) 对方机器人检测 (1=检测到)
    pub road_upper_status: u8,       // data[5] bit 7     公路(上部) 对方机器人检测 (1=检测到)
}

pub struct SDR_EnemyRobotGainData {
    pub cmd_id: u16 = 0x0A05,
    pub data_length: u8 = 36,
    pub data: [u8; data_length],
    // 英雄机器人
    pub hero_hp_recovery: u8,               // data[0]         回血增益 (百分比)
    pub hero_cooling_acceleration: u16,      // data[1..3] u16 LE 射击热量冷却增益 (直接值)
    pub hero_defence: u8,                   // data[3]         防御增益 (百分比)
    pub hero_negative_defence: u8,           // data[4]         负防御增益 (百分比)
    pub hero_attack: u16,                    // data[5..7] u16 LE 攻击增益 (百分比)
    // 工程机器人
    pub engineer_hp_recovery: u8,            // data[7]         回血增益 (百分比)
    pub engineer_cooling_acceleration: u16,   // data[8..10] u16 LE 射击热量冷却增益 (直接值)
    pub engineer_defence: u8,                // data[10]        防御增益 (百分比)
    pub engineer_negative_defence: u8,        // data[11]        负防御增益 (百分比)
    pub engineer_attack: u16,                // data[12..14] u16 LE 攻击增益 (百分比)
    // 3号步兵机器人
    pub infantry_3_hp_recovery: u8,          // data[14]        回血增益 (百分比)
    pub infantry_3_cooling_acceleration: u16, // data[15..17] u16 LE 射击热量冷却增益 (直接值)
    pub infantry_3_defence: u8,              // data[17]        防御增益 (百分比)
    pub infantry_3_negative_defence: u8,      // data[18]        负防御增益 (百分比)
    pub infantry_3_attack: u16,              // data[19..21] u16 LE 攻击增益 (百分比)
    // 4号步兵机器人
    pub infantry_4_hp_recovery: u8,          // data[21]        回血增益 (百分比)
    pub infantry_4_cooling_acceleration: u16, // data[22..24] u16 LE 射击热量冷却增益 (直接值)
    pub infantry_4_defence: u8,              // data[24]        防御增益 (百分比)
    pub infantry_4_negative_defence: u8,      // data[25]        负防御增益 (百分比)
    pub infantry_4_attack: u16,              // data[26..28] u16 LE 攻击增益 (百分比)
    // 哨兵机器人
    pub sentry_hp_recovery: u8,              // data[28]        回血增益 (百分比)
    pub sentry_cooling_acceleration: u16,     // data[29..31] u16 LE 射击热量冷却增益 (直接值)
    pub sentry_defence: u8,                  // data[31]        防御增益 (百分比)
    pub sentry_negative_defence: u8,          // data[32]        负防御增益 (百分比)
    pub sentry_attack: u16,                  // data[33..35] u16 LE 攻击增益 (百分比)
    pub sentry_posture: u8,                  // data[35]        哨兵姿态 (1=进攻 2=防御 3=移动)
}

pub struct SDR_JammingKeyData {
    pub cmd_id: u16 = 0x0A06,
    pub data_length: u8 = 6,
    pub data: [u8; data_length],
    pub key: [u8; 6], // data[0..6]  干扰密钥 (ASCII编码 字母或数字)
}
