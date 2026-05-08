use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::watch;

pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const MAX_DIMENSION: u32 = 4096;
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const MAX_BACKOFF: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(1);

const SHM_MAGIC: u32 = 0x4C465248; // "LFRH"
const HEADER_SIZE: usize = 64;

pub async fn run_video_client(
    shm_name: &str,
    shared: Arc<Mutex<Option<VideoFrame>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut backoff = INITIAL_BACKOFF;

    loop {
        if *shutdown.borrow() {
            break;
        }

        log::info!("[shm] opening shared memory: {}", shm_name);

        let mapping = match open_shm(shm_name) {
            Ok(m) => {
                log::info!("[shm] attached ({}x{}, {:.1} MB)",
                    m.width, m.height,
                    m.map_len as f64 / (1024.0 * 1024.0));
                backoff = INITIAL_BACKOFF;
                m
            }
            Err(e) => {
                log::warn!("[shm] open failed: {} (retry in {:?})", e, backoff);
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = shutdown.changed() => return,
                }
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        let mut last_seq: u32 = 0;

        // Poll loop
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = shutdown.changed() => return,
            }

            // Read frame_seq atomically (acquire load via volatile read)
            let header = mapping.header();
            let seq = header.frame_seq();

            if seq == last_seq {
                continue;
            }

            // Validate magic is still intact (producer may have restarted)
            if header.magic() != SHM_MAGIC {
                log::warn!("[shm] magic mismatch, producer may have restarted");
                break;
            }

            last_seq = seq;

            // Read from the buffer that the producer is NOT currently writing to
            let write_idx = header.write_idx();
            let read_idx = 1 - write_idx;

            let frame_size = (mapping.width * mapping.height * 3) as usize;
            let buf_offset = HEADER_SIZE + (read_idx as usize) * frame_size;

            // Safety: we validated dimensions on open, and double-buffer protocol
            // ensures producer won't write to read_idx buffer
            let frame_data = unsafe {
                std::slice::from_raw_parts(
                    (mapping.ptr as *const u8).add(buf_offset),
                    frame_size,
                )
            };

            if let Ok(mut state) = shared.lock() {
                // Reuse allocation if possible
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

        // Unmap before retry
        drop(mapping);
        log::warn!("[shm] disconnected, reconnecting in {:?}...", backoff);
        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = shutdown.changed() => return,
        }
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

struct ShmMapping {
    ptr: *mut u8,
    map_len: usize,
    width: u32,
    height: u32,
}

// Safety: the shared memory region is read-only from consumer side,
// and we only access it through atomic/volatile reads for the header.
unsafe impl Send for ShmMapping {}

impl ShmMapping {
    fn header(&self) -> ShmHeaderView {
        ShmHeaderView { ptr: self.ptr }
    }
}

impl Drop for ShmMapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.map_len);
        }
    }
}

struct ShmHeaderView {
    ptr: *mut u8,
}

impl ShmHeaderView {
    fn magic(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr as *const u32) }
    }

    fn frame_seq(&self) -> u32 {
        // Atomic acquire load via volatile + fence
        let val = unsafe {
            std::ptr::read_volatile(self.ptr.add(16) as *const u32)
        };
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        val
    }

    fn write_idx(&self) -> u32 {
        unsafe { std::ptr::read_volatile(self.ptr.add(20) as *const u32) }
    }
}

fn open_shm(shm_name: &str) -> Result<ShmMapping, String> {
    use std::ffi::CString;

    let c_name = CString::new(shm_name).map_err(|e| e.to_string())?;

    let fd = unsafe { libc::shm_open(c_name.as_ptr(), libc::O_RDONLY, 0) };
    if fd < 0 {
        return Err(format!("shm_open: {}", std::io::Error::last_os_error()));
    }

    // Read the header first to get dimensions
    let header_map = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            HEADER_SIZE,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };

    if header_map == libc::MAP_FAILED {
        unsafe { libc::close(fd) };
        return Err(format!("mmap header: {}", std::io::Error::last_os_error()));
    }

    let magic = unsafe { std::ptr::read_volatile(header_map as *const u32) };
    if magic != SHM_MAGIC {
        unsafe {
            libc::munmap(header_map, HEADER_SIZE);
            libc::close(fd);
        }
        return Err(format!("bad magic: 0x{:08X} (expected 0x{:08X})", magic, SHM_MAGIC));
    }

    let width = unsafe { std::ptr::read_volatile((header_map as *const u32).add(1)) };
    let height = unsafe { std::ptr::read_volatile((header_map as *const u32).add(2)) };

    unsafe { libc::munmap(header_map, HEADER_SIZE) };

    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        unsafe { libc::close(fd) };
        return Err(format!("invalid dimensions: {}x{}", width, height));
    }

    let frame_size = (width as usize) * (height as usize) * 3;
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
        width,
        height,
    })
}
