use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::watch;

use crate::laser_protocol;
use crate::state_snapshots::LaserObservationWriter;

pub async fn run_laser_client(
    port: u16,
    writer: LaserObservationWriter,
    mut shutdown: watch::Receiver<bool>,
) {
    let addr = format!("0.0.0.0:{}", port);
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => {
            log::info!("Laser UDP listener bound to {}", addr);
            s
        }
        Err(e) => {
            log::error!("Failed to bind UDP socket: {}", e);
            return;
        }
    };

    let mut buf = vec![0u8; 4096];

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, src)) => {
                        log::debug!("UDP received {} bytes from {}", len, src);
                        match laser_protocol::parse_laser_packet(&buf[..len]) {
                            Some(obs) => {
                                log::debug!("Parsed: detected={}, center=[{:.1}, {:.1}], candidates={}",
                                    obs.detected, obs.center[0], obs.center[1], obs.candidates.len());
                                writer.publish(obs);
                            }
                            None => {
                                log::warn!("Failed to parse UDP packet ({} bytes)", len);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("UDP recv error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
            _ = shutdown.changed() => {
                log::info!("Laser client shutdown signal received.");
                return;
            }
        }
    }
}
