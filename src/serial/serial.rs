use super::data_format::{
    SerialProtocolData, DART_LAUNCH_CMD_ID, GAME_RESULT_CMD_ID, GAME_STATE_CMD_ID,
    RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID, RADAR_MARK_PROCESS_CMD_ID, ROBOT_INTERACTION_CMD_ID,
    SITE_EVENT_CMD_ID,
};
use super::serial_package::serial_package;
use super::serial_parser::SerialParser;
use super::serialconfig::SerialConfig;
use deku::prelude::*;
use serial2::{SerialPort, Settings};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct Serial {
    serial_port: SerialPort,
}

impl Serial {
    pub fn new(config: SerialConfig) -> std::io::Result<Self> {
        let mut port = SerialPort::open(config.port_name, |mut s: Settings| {
            s.set_raw();
            s.set_baud_rate(config.baud_rate)?;
            s.set_char_size(serial2::CharSize::Bits8);
            s.set_stop_bits(serial2::StopBits::One);
            Ok(s)
        })?;

        port.set_read_timeout(Duration::from_millis(config.timeout))?;

        Ok(Self { serial_port: port })
    }

    pub fn receive_data(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; 1024];
        let bytes_read = self.serial_port.read(&mut buf)?;
        buf.truncate(bytes_read);
        Ok(buf)
    }

    pub fn send_data(&self, data: &[u8]) -> std::io::Result<()> {
        self.serial_port.write_all(data)?;
        Ok(())
    }
    pub fn clone_serial_port(&self) -> std::io::Result<Self> {
        Ok(Self {
            serial_port: self.serial_port.try_clone()?,
        })
    }
}

pub fn start_receiver(
    mut serial: Serial,
    protocol_data_receiver_state: Arc<Mutex<SerialProtocolData>>,
) -> thread::JoinHandle<()> {
    let mut serial_parser = SerialParser::new(protocol_data_receiver_state);
    let mut data: Vec<u8> = Vec::new();
    thread::spawn(move || loop {
        match serial.receive_data() {
            Ok(add_data) => {
                data.extend_from_slice(&add_data);
                let (parsed, _remaining) = serial_parser.parser(&mut data);
                if parsed {
                    continue;
                }
            }
            Err(e) => {
                println!("Error receiving data: {}", e);
                thread::sleep(Duration::from_millis(50));
            }
        }
    })
}

pub fn start_transmitter(
    serial: Serial,
    tx_state: Arc<Mutex<SerialProtocolData>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let mut data = tx_state.lock().unwrap();
        for idx in 0..7 {
            if data.zmq_produced[idx] == 0 {
                continue;
            }
            let (cmd_id, raw) = match idx {
                0 => (GAME_STATE_CMD_ID, data.game_state_data.to_bytes()),
                1 => (GAME_RESULT_CMD_ID, data.game_result_data.to_bytes()),
                2 => (SITE_EVENT_CMD_ID, data.site_event_data.to_bytes()),
                3 => (DART_LAUNCH_CMD_ID, data.dart_launch_data.to_bytes()),
                4 => (
                    RADAR_MARK_PROCESS_CMD_ID,
                    data.radar_mark_process_data.to_bytes(),
                ),
                5 => (
                    RADAR_AUTONOMOUS_DECISION_SYNC_CMD_ID,
                    data.radar_autonomous_decision_sync_data.to_bytes(),
                ),
                6 => {
                    let b = super::serial_package::robot_interaction_to_bytes(
                        &data.robot_interaction_data,
                    );
                    (ROBOT_INTERACTION_CMD_ID, Ok(b))
                }
                _ => continue,
            };
            if let Ok(data_bytes) = raw {
                let frame = serial_package(cmd_id, data_bytes);
                if let Ok(frame_bytes) = frame.to_bytes() {
                    if let Err(e) = serial.send_data(&frame_bytes) {
                        eprintln!("Transmitter send error: {}", e);
                    }
                }
            }
            data.zmq_produced[idx] = 0;
        }
        drop(data);
        thread::sleep(Duration::from_millis(10));
    })
}
