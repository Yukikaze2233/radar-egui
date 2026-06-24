use super::serial_crc;
use crate::serial::data_format::{
    self, CMD_ID_LENGTH, CRC16_LENGTH, DART_LAUNCH_CMD_ID, FRAME_HEADER_LENGTH, FRAME_HEADER_SOF,
    GAME_RESULT_CMD_ID, GAME_STATE_CMD_ID, RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID,
    RADAR_MARK_PROCESS_CMD_ID, SITE_EVENT_CMD_ID,
};
use deku::prelude::*;

pub struct SerialParser {
    frame_header: data_format::SerialFrameHeader,
    protocol_data: data_format::SerialProtocolData,
}

impl SerialParser {
    pub fn new() -> Self {
        SerialParser {
            frame_header: data_format::SerialFrameHeader::default(),
            protocol_data: data_format::SerialProtocolData::default(),
        }
    }

    /// 只读访问已解析的协议数据
    pub fn protocol_data(&self) -> &data_format::SerialProtocolData {
        &self.protocol_data
    }

    /// 扫描 read_buffer, 解析其中的完整帧并写入 self.protocol_data
    /// 返回本次是否至少成功解析了一帧
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
                        self.protocol_data.game_state_data = v;
                        parsed_any = true;
                    }
                }
                GAME_RESULT_CMD_ID => {
                    if let Ok((_, v)) = data_format::GameResultData::from_bytes((data, 0)) {
                        self.protocol_data.game_result_data = v;
                        parsed_any = true;
                    }
                }
                SITE_EVENT_CMD_ID => {
                    if let Ok((_, v)) = data_format::SiteEventData::from_bytes((data, 0)) {
                        self.protocol_data.site_event_data = v;
                        parsed_any = true;
                    }
                }
                DART_LAUNCH_CMD_ID => {
                    if let Ok((_, v)) = data_format::DartLaunchData::from_bytes((data, 0)) {
                        self.protocol_data.dart_launch_data = v;
                        parsed_any = true;
                    }
                }
                RADAR_MARK_PROCESS_CMD_ID => {
                    if let Ok((_, v)) = data_format::RadarMarkProcessData::from_bytes((data, 0)) {
                        self.protocol_data.radar_mark_process_data = v;
                        parsed_any = true;
                    }
                }
                RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID => {
                    if let Ok((_, v)) =
                        data_format::RadarAutonomousDecisionSyncData::from_bytes((data, 0))
                    {
                        self.protocol_data.radar_autonomous_decision_sync_data = v;
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
