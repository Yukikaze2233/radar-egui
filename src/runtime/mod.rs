use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use tokio::sync::watch;

use crate::laser::video::{self, VideoFrameWriter};
use crate::pointcloud::reader;
use crate::shared_data::SharedData;
use crate::state::{LaserObservationWriter, PointCloudFrameWriter};

fn spawn_runtime_task<M, F>(make_future: M)
where
    M: FnOnce() -> F + Send + 'static,
    F: Future<Output = ()> + 'static,
{
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(make_future());
    });
}

// ── ZMQ SUB runtime ──

pub struct ZmqSubRuntime {
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
    tx_slot: Arc<Mutex<Option<std::sync::mpsc::Sender<usize>>>>,
}

impl ZmqSubRuntime {
    pub fn start(addrs: &[String], shared: Arc<Mutex<SharedData>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let tx_slot = Arc::new(Mutex::new(None));
        let addrs = addrs.to_vec();
        let sub_socket = crate::zmq::zmq::zmq_init_sub(1, &addrs).expect("ZMQ SUB init failed");
        let handle =
            crate::zmq::zmq::zmq_start_sub(sub_socket, shared, stop.clone(), tx_slot.clone());
        Self {
            stop,
            handle: Mutex::new(Some(handle)),
            tx_slot,
        }
    }

    /// Attach (or detach with `None`) the serial TX notification channel.
    /// The ZMQ SUB thread notifies the serial transmitter on SDR / Lidar messages.
    pub fn set_tx_notify(&self, tx: Option<std::sync::mpsc::Sender<usize>>) {
        if let Ok(mut slot) = self.tx_slot.lock() {
            *slot = tx;
        }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        // The SUB thread blocks on recv_bytes and cannot be interrupted without
        // a receive timeout; detach it instead of joining (thread exits at
        // process teardown).
        if let Ok(mut h) = self.handle.lock() {
            h.take();
        }
    }

    pub fn is_started(&self) -> bool {
        !self.stop.load(Ordering::Relaxed)
    }
}

// ── ZMQ PUB runtime ──

pub struct ZmqPubRuntime {
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
    pub pub_tx: Mutex<Option<std::sync::mpsc::Sender<usize>>>,
}

impl ZmqPubRuntime {
    pub fn start(bind_addrs: &[&str], shared: Arc<Mutex<SharedData>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let (pub_tx, pub_rx) = std::sync::mpsc::channel();
        let sockets: Vec<_> = bind_addrs
            .iter()
            .map(|addr| crate::zmq::zmq::zmq_init_pub(1, addr).expect("ZMQ PUB init failed"))
            .collect();
        let handle = crate::zmq::zmq::zmq_start_pub(sockets, shared, pub_rx, stop.clone());
        Self {
            stop,
            handle: Mutex::new(Some(handle)),
            pub_tx: Mutex::new(Some(pub_tx)),
        }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(mut tx) = self.pub_tx.lock() {
            tx.take();
        }
        if let Ok(mut h) = self.handle.lock() {
            if let Some(handle) = h.take() {
                let _ = handle.join();
            }
        }
    }
}

// ── Noise Stream (127.0.0.1:3000, 7 bytes) ──

pub struct NoiseStreamRuntime {
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl NoiseStreamRuntime {
    pub fn start(shared: Arc<Mutex<SharedData>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let handle = std::thread::spawn(move || {
            use std::net::UdpSocket;
            let socket = match UdpSocket::bind("127.0.0.1:3000") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to bind noise stream to 127.0.0.1:3000: {}", e);
                    return;
                }
            };
            let mut buf = [0u8; 7];
            log::info!("Noise stream listener started on 127.0.0.1:3000");
            while !stop.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        if len == 7 {
                            let data = buf[..7].try_into().unwrap();
                            let mut shared = shared.lock().unwrap();
                            shared.noise = data;
                            log::debug!("Noise stream received: {:02X?}", data);
                        }
                    }
                    Err(e) => {
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        log::warn!("Noise stream recv error: {}", e);
                    }
                }
            }
            log::info!("Noise stream listener stopped");
        });
        Self {
            stop,
            handle: Mutex::new(Some(handle)),
        }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(mut h) = self.handle.lock() {
            h.take();
        }
    }

    pub fn is_started(&self) -> bool {
        !self.stop.load(Ordering::Relaxed)
    }
}

// ── Video (SHM) ──

pub struct VideoRuntime {
    shutdown_tx: watch::Sender<bool>,
    started: bool,
    writer: VideoFrameWriter,
}

impl VideoRuntime {
    pub fn new(writer: VideoFrameWriter) -> Self {
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);
        Self {
            shutdown_tx,
            started: false,
            writer,
        }
    }

    pub fn ensure_started(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let _ = self.shutdown_tx.send(true);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;
        let writer = self.writer.clone();
        spawn_runtime_task(move || async move {
            video::run_video_client(writer, shutdown_rx).await;
        });
    }
}

// ── PointCloud (SHM) ──

pub struct PointCloudRuntime {
    shutdown_tx: watch::Sender<bool>,
    started: bool,
    writer: PointCloudFrameWriter,
}

impl PointCloudRuntime {
    pub fn new(writer: PointCloudFrameWriter) -> Self {
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);
        Self {
            shutdown_tx,
            started: false,
            writer,
        }
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn ensure_started(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let _ = self.shutdown_tx.send(true);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;
        let writer = self.writer.clone();
        spawn_runtime_task(move || async move {
            reader::run_pointcloud_client(writer, shutdown_rx).await;
        });
    }
}
