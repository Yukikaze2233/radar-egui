use std::sync::{Arc, Mutex};

use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;
use tokio::sync::watch;

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub async fn run_video_client(
    path: &str,
    shared: Arc<Mutex<Option<VideoFrame>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let stream = match UnixStream::connect(path).await {
        Ok(s) => {
            log::info!("Video client connected to {}", path);
            s
        }
        Err(e) => {
            log::error!("Failed to connect to video socket {}: {}", path, e);
            return;
        }
    };

    let (mut reader, _writer) = stream.into_split();
    let mut header_buf = [0u8; 8];

    loop {
        tokio::select! {
            result = reader.read_exact(&mut header_buf) => {
                if result.is_ok() {
                    let width = u32::from_le_bytes(header_buf[0..4].try_into().unwrap());
                    let height = u32::from_le_bytes(header_buf[4..8].try_into().unwrap());
                    let data_len = (width * height * 3) as usize;

                    let mut data = vec![0u8; data_len];
                    if reader.read_exact(&mut data).await.is_err() {
                        log::warn!("Video socket: failed to read frame data");
                        break;
                    }

                    if let Ok(mut state) = shared.lock() {
                        *state = Some(VideoFrame { data, width, height });
                    }
                } else {
                    let e = result.unwrap_err();
                    log::warn!("Video socket disconnected: {}", e);
                    break;
                }
            }
            _ = shutdown.changed() => {
                log::info!("Video client shutdown signal received.");
                return;
            }
        }
    }
}
