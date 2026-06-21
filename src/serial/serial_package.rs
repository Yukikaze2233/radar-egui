use super::data_format::{SerialFrame, SerialFrameHeader};
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
        if let Ok(crc8) = serial_crc::append_crc8(&mut header_bytes) {
            crc8
        } else {
            0
        }
    };
    let mut package = SerialFrame {
        frame_header,
        cmd_id,
        data,
        frame_crc16: 0,
    };
    package.frame_crc16 = {
        let mut package_bytes = package.to_bytes().unwrap();
        if let Ok(crc16) = serial_crc::append_crc16(&mut package_bytes) {
            crc16
        } else {
            0
        }
    };
    package
}
