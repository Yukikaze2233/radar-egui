use std::thread;
use std::time::Duration;

use super::serial_parser::SerialParser;
use serial2::{SerialPort, Settings};

use crate::serial::{serial_parser, serialconfig::SerialConfig};

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
}

pub fn start_receiver(mut serial: Serial) -> thread::JoinHandle<()> {
    let mut serial_parser = SerialParser::new();
    let mut data: Vec<u8> = Vec::new();
    thread::spawn(move || loop {
        match serial.receive_data() {
            Ok(add_data) => {
                data.extend_from_slice(&add_data);
                let (parsed, _remaining) = serial_parser.parser(&mut data);
                if parsed {
                    println!("Parsed protocol data: {:?}", serial_parser.protocol_data());
                }
            }
            Err(e) => {
                println!("Error receiving data: {}", e);
            }
        }
    })
}

pub fn start_transmitter(serial: Serial) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        match serial.send_data(&data) {
            Ok(_) => println!("Sent data: {:?}", data),
            Err(e) => eprintln!("Error sending data: {}", e),
        }
        thread::sleep(Duration::from_secs(1));
    })
}
