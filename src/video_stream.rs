use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;
use tokio::sync::watch;

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const MAX_DIMENSION: u32 = 4096;
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const MAX_BACKOFF: Duration = Duration::from_secs(5);

pub async fn run_video_client(
    socket_path: &str,
    shared: Arc<Mutex<Option<VideoFrame>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut backoff = INITIAL_BACKOFF;

    loop {
        if *shutdown.borrow() {
            break;
        }

        log::info!("Connecting to video socket: {}", socket_path);

        let stream = tokio::select! {
            result = UnixStream::connect(socket_path) => result,
            _ = shutdown.changed() => return,
        };

        let mut stream = match stream {
            Ok(s) => {
                log::info!("Video socket connected");
                backoff = INITIAL_BACKOFF;
                s
            }
            Err(e) => {
                log::warn!("Video socket connect failed: {} (retry in {:?})", e, backoff);
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = shutdown.changed() => return,
                }
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        // Read frames until error or shutdown
        let mut header = [0u8; 8];
        loop {
            let read_result = tokio::select! {
                r = stream.read_exact(&mut header) => r,
                _ = shutdown.changed() => return,
            };

            if let Err(e) = read_result {
                log::warn!("Video header read error: {}", e);
                break;
            }

            let width = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
            let height = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

            if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
                log::warn!("Invalid frame dimensions: {}x{}, reconnecting", width, height);
                break;
            }

            let frame_size = (width * height * 3) as usize;
            let mut frame_buf = vec![0u8; frame_size];

            let read_result = tokio::select! {
                r = stream.read_exact(&mut frame_buf) => r,
                _ = shutdown.changed() => return,
            };

            if let Err(e) = read_result {
                log::warn!("Video frame read error: {}", e);
                break;
            }

            if let Ok(mut state) = shared.lock() {
                *state = Some(VideoFrame {
                    data: frame_buf,
                    width,
                    height,
                });
            }
        }

        log::warn!("Video socket disconnected, reconnecting in {:?}...", backoff);
        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = shutdown.changed() => return,
        }
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}
