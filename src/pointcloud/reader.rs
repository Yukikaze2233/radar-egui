use std::time::{Duration, Instant};

use tokio::sync::watch;

use super::protocol::{self, PointCloudFrame, HEADER_SIZE, SHM_MAGIC};
use crate::state::PointCloudFrameWriter;

const SHM_NAME: &str = "/pointcloud_frame";
const MAX_DIMENSION: u32 = 1_000_000; // 最大点数
const POLL_INTERVAL: Duration = Duration::from_millis(2);
const STALE_FRAME_TIMEOUT: Duration = Duration::from_millis(800);

struct ShmMapping {
    ptr: *mut u8,
    map_len: usize,
    max_points: u32,
    stride: u32,
}

unsafe impl Send for ShmMapping {}

struct ShmHeader {
    ptr: *mut u8,
}

impl ShmHeader {
    fn magic(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr as *const u32) }
    }
    fn point_count(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(8) as *const u32) }
    }
    fn max_points(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(12) as *const u32) }
    }
    fn frame_seq(&self) -> u32 {
        let val = unsafe { std::ptr::read_volatile(self.ptr.add(16) as *const u32) };
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        val
    }
    fn write_idx(&self) -> u32 {
        let val = unsafe { std::ptr::read_volatile(self.ptr.add(20) as *const u32) };
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        val
    }
    fn stride(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(24) as *const u32) }
    }
}

impl Drop for ShmMapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.map_len);
        }
    }
}

fn open_shm() -> Result<ShmMapping, String> {
    use std::ffi::CString;

    let c_name = CString::new(SHM_NAME).map_err(|e| e.to_string())?;
    let fd = unsafe { libc::shm_open(c_name.as_ptr(), libc::O_RDONLY, 0) };
    if fd < 0 {
        return Err(format!("shm_open {}: {}", SHM_NAME, std::io::Error::last_os_error()));
    }

    let header_ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            HEADER_SIZE,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if header_ptr == libc::MAP_FAILED {
        unsafe { libc::close(fd) };
        return Err(format!("mmap header: {}", std::io::Error::last_os_error()));
    }

    let header = ShmHeader { ptr: header_ptr as *mut u8 };

    let magic = header.magic();
    if magic != SHM_MAGIC {
        unsafe {
            libc::munmap(header_ptr, HEADER_SIZE);
            libc::close(fd);
        }
        return Err(format!(
            "bad magic: 0x{:08X} (expected 0x{:08X})",
            magic, SHM_MAGIC
        ));
    }

    let max_points = header.max_points();
    let stride = header.stride().max(protocol::DEFAULT_STRIDE);
    unsafe { libc::munmap(header_ptr, HEADER_SIZE) };

    if max_points == 0 || max_points > MAX_DIMENSION {
        unsafe { libc::close(fd) };
        return Err(format!("invalid max_points: {}", max_points));
    }

    let buf_size = (max_points as usize) * (stride as usize);
    let map_len = HEADER_SIZE + 2 * buf_size;

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            map_len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    unsafe { libc::close(fd) };

    if ptr == libc::MAP_FAILED {
        return Err(format!("mmap full: {}", std::io::Error::last_os_error()));
    }

    Ok(ShmMapping {
        ptr: ptr as *mut u8,
        map_len,
        max_points,
        stride,
    })
}

pub async fn run_pointcloud_client(
    writer: PointCloudFrameWriter,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut backoff = Duration::from_millis(500);

    loop {
        if *shutdown.borrow() {
            break;
        }

        log::info!("[pcd] opening {}", SHM_NAME);

        let mapping = match open_shm() {
            Ok(m) => {
                let map_mb = m.map_len as f64 / (1024.0 * 1024.0);
                log::info!(
                    "[pcd] attached (max_points={}, stride={}, {:.1} MB)",
                    m.max_points,
                    m.stride,
                    map_mb
                );
                m
            }
            Err(e) => {
                writer.clear();
                log::warn!("[pcd] open failed: {} (retry in {:?})", e, backoff);
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = shutdown.changed() => return,
                }
                backoff = (backoff * 2).min(Duration::from_secs(5));
                continue;
            }
        };

        backoff = Duration::from_millis(500);
        let header = ShmHeader { ptr: mapping.ptr };
        let buf_size = (mapping.max_points as usize) * (mapping.stride as usize);
        let mut last_seq = 0u32;
        let mut first_frame_received = false;
        let mut last_frame_update = Instant::now();

        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = shutdown.changed() => return,
            }

            let seq = header.frame_seq();
            if seq == last_seq {
                // Only trigger stale remap for live streams (seq was changing before),
                // not for truly static one-shot point clouds.
                if first_frame_received
                    && last_frame_update.elapsed() > STALE_FRAME_TIMEOUT
                {
                    log::warn!(
                        "[pcd] frame sequence stalled for {:?}, remapping",
                        STALE_FRAME_TIMEOUT
                    );
                    break;
                }
                continue;
            }
            last_seq = seq;
            first_frame_received = true;
            last_frame_update = Instant::now();

            if header.magic() != SHM_MAGIC {
                log::warn!("[pcd] magic mismatch, producer restarted");
                break;
            }

            let write_idx = header.write_idx();
            if write_idx > 1 {
                log::warn!("[pcd] invalid write_idx {}, skipping frame", write_idx);
                continue;
            }
            let point_count = header.point_count();
            let stride = header.stride().max(protocol::DEFAULT_STRIDE);

            let buf_offset = HEADER_SIZE + (write_idx as usize) * buf_size;
            let raw_data =
                unsafe { std::slice::from_raw_parts(mapping.ptr.add(buf_offset), buf_size) };

            let mut frame = PointCloudFrame::from_raw(raw_data, point_count, stride);
            frame.seq = seq;

            writer.update(frame);
        }

        log::warn!("[pcd] disconnected, reconnecting...");
        writer.clear();
        drop(mapping);
        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = shutdown.changed() => return,
        }
        backoff = (backoff * 2).min(Duration::from_secs(5));
    }
}
