use std::process::Stdio;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::watch;

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub async fn run_video_client(
    host: &str,
    port: u16,
    width: u32,
    height: u32,
    shared: Arc<Mutex<Option<VideoFrame>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let frame_size = (width * height * 3) as usize;

    loop {
        if *shutdown.borrow() {
            break;
        }

        let sdp = format!(
            "v=0\r\no=- 0 0 IN IP4 {host}\r\ns=No Name\r\nc=IN IP4 {host}\r\nt=0 0\r\nm=video {port} RTP/AVP 96\r\na=rtpmap:96 H264/90000\r\na=fmtp:96 packetization-mode=1\r\n"
        );

        log::info!(
            "Launching ffmpeg RTP receiver ({}:{}, {}x{})...",
            host, port, width, height
        );

        let mut child = match Command::new("ffmpeg")
            .args([
                "-protocol_whitelist", "file,udp,rtp",
                "-f", "sdp",
                "-i", "-",
                "-an",
                "-f", "rawvideo",
                "-pix_fmt", "bgr24",
                "pipe:1",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to spawn ffmpeg: {}", e);
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
                    _ = shutdown.changed() => return,
                }
                continue;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(sdp.as_bytes()).await;
        }

        let mut stdout = child.stdout.take().expect("stdout not piped");
        let mut frame_buf = vec![0u8; frame_size];

        loop {
            tokio::select! {
                result = stdout.read_exact(&mut frame_buf) => {
                    match result {
                        Ok(n) if n == frame_size => {
                            if let Ok(mut state) = shared.lock() {
                                *state = Some(VideoFrame {
                                    data: frame_buf.clone(),
                                    width,
                                    height,
                                });
                            }
                        }
                        Ok(n) => {
                            log::warn!("ffmpeg: partial frame ({} / {}), restarting", n, frame_size);
                            break;
                        }
                        Err(e) => {
                            log::warn!("ffmpeg pipe error: {}", e);
                            break;
                        }
                    }
                }
                _ = shutdown.changed() => {
                    log::info!("Video client shutdown signal received.");
                    let _ = child.kill().await;
                    return;
                }
            }
        }

        let _ = child.kill().await;
        log::warn!("ffmpeg exited, restarting in 2s...");
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
            _ = shutdown.changed() => return,
        }
    }
}
