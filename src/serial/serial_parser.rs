use super::serial_crc;
use crate::serial::data_format::{
    self, CMD_ID_LENGTH, CRC16_LENGTH, DART_LAUNCH_CMD_ID, FRAME_HEADER_LENGTH, FRAME_HEADER_SOF,
    GAME_RESULT_CMD_ID, GAME_STATE_CMD_ID, IDX_DART_LAUNCH, IDX_GAME_RESULT, IDX_GAME_STATE,
    IDX_RADAR_AUTONOMOUS_DECISION_SYNC, IDX_RADAR_MARK_PROCESS, IDX_ROBOT_INTERACTION,
    IDX_SITE_EVENT, RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID, RADAR_MARK_PROCESS_CMD_ID,
    ROBOT_INTERACTION_CMD_ID, SITE_EVENT_CMD_ID,
};
use deku::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

pub struct SerialParser {
    frame_header: data_format::SerialFrameHeader,
    protocol_data: Arc<Mutex<data_format::SerialProtocolData>>,
}

impl SerialParser {
    pub fn new(protocol_data_input: Arc<Mutex<data_format::SerialProtocolData>>) -> Self {
        SerialParser {
            frame_header: data_format::SerialFrameHeader::default(),
            protocol_data: protocol_data_input,
        }
    }
    /// Scan `read_buffer` for complete frames and write parsed data into shared state.
    /// Returns whether at least one frame was successfully parsed.
    pub fn parser<'a>(&mut self, read_buffer: &'a mut Vec<u8>) -> (bool, &'a mut Vec<u8>) {
        let mut parsed_any = false;
        let mut index = 0;
        while index < read_buffer.len() {
            if read_buffer[index] != FRAME_HEADER_SOF {
                index += 1;
                continue;
            }
            let header_end = index + FRAME_HEADER_LENGTH;
            if header_end > read_buffer.len() {
                break;
            }
            if !serial_crc::verify_crc8(&read_buffer[index..header_end]) {
                index += 1;
                continue;
            }

            self.frame_header.frame_header_sof = read_buffer[index];
            self.frame_header.frame_header_data_len =
                u16::from_le_bytes([read_buffer[index + 1], read_buffer[index + 2]]);
            self.frame_header.frame_header_seq = read_buffer[index + 3];
            self.frame_header.frame_header_crc8 = read_buffer[index + 4];

            let data_len = self.frame_header.frame_header_data_len as usize;
            let package_start = index;
            let package_end = index + FRAME_HEADER_LENGTH + CMD_ID_LENGTH + data_len + CRC16_LENGTH;
            if package_end > read_buffer.len() {
                break;
            }
            if !serial_crc::verify_crc16(&read_buffer[package_start..package_end]) {
                index += FRAME_HEADER_LENGTH
                    + CMD_ID_LENGTH
                    + self.frame_header.frame_header_data_len as usize
                    + CRC16_LENGTH;
                continue;
            }

            let cmd_id = u16::from_le_bytes([read_buffer[index + 5], read_buffer[index + 6]]);
            let data_start = index + FRAME_HEADER_LENGTH + CMD_ID_LENGTH;
            let data = &read_buffer[data_start..data_start + data_len];

            match cmd_id {
                GAME_STATE_CMD_ID => {
                    if let Ok((_, v)) = data_format::GameStateData::from_bytes((data, 0)) {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.game_state_data = v;
                        lock.serial_produced[IDX_GAME_STATE] = 1;
                        parsed_any = true;
                    }
                }
                GAME_RESULT_CMD_ID => {
                    if let Ok((_, v)) = data_format::GameResultData::from_bytes((data, 0)) {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.game_result_data = v;
                        lock.serial_produced[IDX_GAME_RESULT] = 1;
                        parsed_any = true;
                    }
                }
                SITE_EVENT_CMD_ID => {
                    if let Ok((_, v)) = data_format::SiteEventData::from_bytes((data, 0)) {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.site_event_data = v;
                        lock.serial_produced[IDX_SITE_EVENT] = 1;
                        parsed_any = true;
                    }
                }
                DART_LAUNCH_CMD_ID => {
                    if let Ok((_, v)) = data_format::DartLaunchData::from_bytes((data, 0)) {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.dart_launch_data = v;
                        lock.serial_produced[IDX_DART_LAUNCH] = 1;
                        parsed_any = true;
                    }
                }
                RADAR_MARK_PROCESS_CMD_ID => {
                    if let Ok((_, v)) = data_format::RadarMarkProcessData::from_bytes((data, 0)) {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.radar_mark_process_data = v;
                        lock.serial_produced[IDX_RADAR_MARK_PROCESS] = 1;
                        parsed_any = true;
                    }
                }
                RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID => {
                    if let Ok((_, v)) =
                        data_format::RadarAutonomousDecisionSyncData::from_bytes((data, 0))
                    {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.radar_autonomous_decision_sync_data = v;
                        lock.serial_produced[IDX_RADAR_AUTONOMOUS_DECISION_SYNC] = 1;
                        parsed_any = true;
                    }
                }
                ROBOT_INTERACTION_CMD_ID => {
                    if let Ok((remaining, header)) =
                        data_format::RobotInteractionHeader::from_bytes((data, 0))
                    {
                        let mut lock = self.protocol_data.lock().unwrap();
                        lock.robot_interaction_data = data_format::RobotInteractionData {
                            data_cmd_id: header.data_cmd_id,
                            sender_id: header.sender_id,
                            receiver_id: header.receiver_id,
                            user_data: remaining.0.to_vec(),
                        };
                        lock.serial_produced[IDX_ROBOT_INTERACTION] = 1;
                        parsed_any = true;
                    }
                }
                _ => {}
            }
            index = package_end;
        }
        read_buffer.drain(0..index);
        (parsed_any, read_buffer)
    }
}
