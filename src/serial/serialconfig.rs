/// Serial port configuration.
pub struct SerialConfig {
    /// Port device path, e.g. "/dev/ttyUSB0"
    pub port_name: String,
    /// Baud rate, e.g. 115200
    pub baud_rate: u32,
    /// Read timeout in milliseconds
    pub timeout: u64,
}