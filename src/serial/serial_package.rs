use super::data_format::{RobotInteractionData, RobotInteractionHeader, SerialFrame, SerialFrameHeader};
use super::robot_interaction_id::DeviceId;
use super::serial_crc;
use deku::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};
static PACKET_SEQ: AtomicU8 = AtomicU8::new(0);
pub fn serial_package(cmd_id: u16, data: Vec<u8>) -> SerialFrame {
    let seq = PACKET_SEQ.fetch_add(1, Ordering::SeqCst);
    let mut frame_header: SerialFrameHeader = SerialFrameHeader {
        frame_header_sof: 0xA5,
        frame_header_data_len: data.len() as u16,
        frame_header_seq: seq,
        frame_header_crc8: 0,
    };
    frame_header.frame_header_crc8 = {
        let mut header_bytes = frame_header.to_bytes().unwrap();
        serial_crc::append_crc8(&mut header_bytes).unwrap_or_default()
    };
    let mut package = SerialFrame {
        frame_header,
        cmd_id,
        data,
        frame_crc16: 0,
    };
    package.frame_crc16 = {
        let mut package_bytes = package.to_bytes().unwrap();
        serial_crc::append_crc16(&mut package_bytes).unwrap_or_default()
    };
    package
}
pub fn robot_interaction_package(
    cmd_id: &u16,
    sender_id: &DeviceId,
    receiver_id: &DeviceId,
    user_data: Vec<u8>,
) -> RobotInteractionData {
    let robot_interaction = RobotInteractionData {
        data_cmd_id: *cmd_id,
        sender_id: *sender_id,
        receiver_id: *receiver_id,
        user_data,
    };
    robot_interaction
}

pub fn robot_interaction_to_bytes(pkg: &RobotInteractionData) -> Vec<u8> {
    let header = RobotInteractionHeader {
        data_cmd_id: pkg.data_cmd_id,
        sender_id: pkg.sender_id,
        receiver_id: pkg.receiver_id,
    };
    let mut bytes = header.to_bytes().unwrap();
    bytes.extend_from_slice(&pkg.user_data);
    bytes
}
