use std::io;
use std::sync::{Arc, Mutex};

use super::data_format::{TransmitGameState, TransmitRadarMarkProcess, TransmitRadarSync};
use crate::serial::data_format::{
    SerialProtocolData, GAME_STATE_CMD_ID, RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID,
    RADAR_MARK_PROCESS_CMD_ID,
};

fn invalid_cmd_id(cmd: u16) -> serde_json::Error {
    serde_json::Error::io(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("unknown cmd_id: {:#06x}", cmd),
    ))
}

/// Read the specified `cmd_id` field from shared state and serialize to a JSON string.
pub fn zmq_package(
    cmd_id: u16,
    protocol_data: Arc<Mutex<SerialProtocolData>>,
) -> Result<String, serde_json::Error> {
    let lock = protocol_data.lock().unwrap();

    let json = match cmd_id {
        GAME_STATE_CMD_ID => {
            let src = &lock.game_state_data;
            let data = TransmitGameState {
                cmd_id,
                game_type: src.game_type,
                game_progress: src.game_progress,
                stage_remain_time: src.stage_remain_time,
                sync_timestamp: src.sync_timestamp,
            };
            serde_json::to_string(&data)?
        }
        RADAR_MARK_PROCESS_CMD_ID => {
            let src = &lock.radar_mark_process_data;
            let data = TransmitRadarMarkProcess {
                cmd_id,
                opponent_hero_vulnerable: src.opponent_hero_vulnerable,
                opponent_engineer_vulnerable: src.opponent_engineer_vulnerable,
                opponent_infantry_3_vulnerable: src.opponent_infantry_3_vulnerable,
                opponent_infantry_4_vulnerable: src.opponent_infantry_4_vulnerable,
                opponent_aerial_marked: src.opponent_aerial_marked,
                opponent_sentry_vulnerable: src.opponent_sentry_vulnerable,
                ally_hero_marked: src.ally_hero_marked,
                ally_engineer_marked: src.ally_engineer_marked,
                ally_infantry_3_marked: src.ally_infantry_3_marked,
                ally_infantry_4_marked: src.ally_infantry_4_marked,
                ally_aerial_marked: src.ally_aerial_marked,
                ally_sentry_marked: src.ally_sentry_marked,
            };
            serde_json::to_string(&data)?
        }
        RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID => {
            let src = &lock.radar_autonomous_decision_sync_data;
            let data = TransmitRadarSync {
                cmd_id,
                double_weakness_chance: src.double_weakness_chance,
                double_weakness_active: src.double_weakness_active,
                encryption_rank: src.encryption_rank,
                key_modifiable: src.key_modifiable,
            };
            serde_json::to_string(&data)?
        }
        _ => return Err(invalid_cmd_id(cmd_id)),
    };

    Ok(json)
}
