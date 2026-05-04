use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::watch;

use crate::protocol::{self, RoboMasterSignalInfo};

/// Connect to SDR signal stream at `addr`, parse incoming data, and update shared state.
///
/// Behavior mirrors Python `tcp_gnuradio_signal_receiver`:
///   - Connect to 127.0.0.1:2000
///   - recv(1024) in a loop
///   - Buffer accumulates until >= 200 bytes, then parse and clear
///   - Auto-reconnect on connection loss
///   - Graceful shutdown via `shutdown` watch channel
pub async fn run_signal_client(
    addr: &str,
    shared: Arc<Mutex<RoboMasterSignalInfo>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut buffer: Vec<u8> = Vec::new();
    const BUFFER_THRESHOLD: usize = 200;

    loop {
        if *shutdown.borrow() {
            break;
        }

        log::info!("Connecting to SDR signal server at {}...", addr);
        let stream = match TcpStream::connect(addr).await {
            Ok(s) => {
                log::info!("Connected to SDR signal server.");
                s
            }
            Err(e) => {
                log::warn!("Connection failed: {}. Retrying in 2s...", e);
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                    _ = shutdown.changed() => {}
                }
                continue;
            }
        };

        let (mut reader, _writer) = stream.into_split();
        let mut recv_buf = [0u8; 1024];

        loop {
            tokio::select! {
                result = reader.read(&mut recv_buf) => {
                    match result {
                        Ok(0) => {
                            log::warn!("Connection closed. Reconnecting...");
                            break;
                        }
                        Ok(n) => {
                            buffer.extend_from_slice(&recv_buf[..n]);
                            if buffer.len() >= BUFFER_THRESHOLD {
                                if let Some(info) = protocol::parse_signal(&buffer) {
                                    if let Ok(mut state) = shared.lock() {
                                        *state = info;
                                    }
                                }
                                buffer.clear();
                            }
                        }
                        Err(e) => {
                            log::warn!("Read error: {}. Reconnecting...", e);
                            break;
                        }
                    }
                }
                _ = shutdown.changed() => {
                    log::info!("Shutdown signal received.");
                    return;
                }
            }
        }
    }
}
