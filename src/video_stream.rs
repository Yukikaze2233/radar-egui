use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::watch;

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const SHM_NAME: &str = "/laser_frame";
const SHM_MAGIC: u32 = 0x4C465248;
const HEADER_SIZE: usize = 64;
const MAX_DIMENSION: u32 = 4096;
const POLL_INTERVAL: Duration = Duration::from_millis(2);

struct ShmMapping {
    ptr: *mut u8,
    map_len: usize,
    width: u32,
    height: u32,
}

unsafe impl Send for ShmMapping {}

struct ShmHeader {
    ptr: *mut u8,
}

impl ShmHeader {
    fn magic(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr as *const u32) }
    }
    fn width(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(4) as *const u32) }
    }
    fn height(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(8) as *const u32) }
    }
    fn frame_seq(&self) -> u32 {
        let val = unsafe { std::ptr::read_volatile(self.ptr.add(16) as *const u32) };
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        val
    }
    fn write_idx(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(20) as *const u32) }
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
        return Err(format!("shm_open: {}", std::io::Error::last_os_error()));
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

    if header.magic() != SHM_MAGIC {
        unsafe {
            libc::munmap(header_ptr, HEADER_SIZE);
            libc::close(fd);
        }
        return Err(format!("bad magic: 0x{:08X} (expected 0x{:08X})", header.magic(), SHM_MAGIC));
    }

    let w = header.width();
    let h = header.height();
    unsafe { libc::munmap(header_ptr, HEADER_SIZE) };

    if w == 0 || h == 0 || w > MAX_DIMENSION || h > MAX_DIMENSION {
        unsafe { libc::close(fd) };
        return Err(format!("invalid dimensions: {}x{}", w, h));
    }

    let frame_size = (w as usize) * (h as usize) * 3;
    let map_len = HEADER_SIZE + 2 * frame_size;

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
        width: w,
        height: h,
    })
}

pub async fn run_video_client(
    shared: Arc<Mutex<Option<VideoFrame>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut backoff = Duration::from_millis(500);

    loop {
        if *shutdown.borrow() {
            break;
        }

        log::info!("[shm] opening {}", SHM_NAME);

        let mapping = match open_shm() {
            Ok(m) => {
                let map_mb = m.map_len as f64 / (1024.0 * 1024.0);
                log::info!("[shm] attached ({}x{}, {:.1} MB)", m.width, m.height, map_mb);
                m
            }
            Err(e) => {
                log::warn!("[shm] open failed: {} (retry in {:?})", e, backoff);
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
        let frame_size = (mapping.width * mapping.height * 3) as usize;
        let mut last_seq = header.frame_seq();

        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = shutdown.changed() => return,
            }

            let seq = header.frame_seq();
            if seq == last_seq {
                continue;
            }
            last_seq = seq;

            if header.magic() != SHM_MAGIC {
                log::warn!("[shm] magic mismatch, producer restarted");
                break;
            }

            let write_idx = header.write_idx();
            let read_idx = 1 - write_idx;
            let buf_offset = HEADER_SIZE + (read_idx as usize) * frame_size;

            let frame_data = unsafe {
                std::slice::from_raw_parts(mapping.ptr.add(buf_offset), frame_size)
            };

            if let Ok(mut state) = shared.lock() {
                match state.as_mut() {
                    Some(existing)
                        if existing.data.len() == frame_size
                            && existing.width == mapping.width
                            && existing.height == mapping.height =>
                    {
                        existing.data.copy_from_slice(frame_data);
                    }
                    _ => {
                        *state = Some(VideoFrame {
                            data: frame_data.to_vec(),
                            width: mapping.width,
                            height: mapping.height,
                        });
                    }
                }
            }
        }

        log::warn!("[shm] disconnected, reconnecting...");
        drop(mapping);
        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = shutdown.changed() => return,
        }
        backoff = (backoff * 2).min(Duration::from_secs(5));
    }
}
