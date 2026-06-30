use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tokio::sync::watch;

use crate::laser::video::{self, VideoFrameWriter};
use crate::pointcloud::reader;
use crate::state::{LaserObservationWriter, PointCloudFrameWriter, ZmqWriter};

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

// ── ZMQ-based runtimes (std::thread, no tokio) ──

pub struct ZmqSdrRuntime {
    stop: Arc<AtomicBool>,
}

impl ZmqSdrRuntime {
    pub fn start(addr: &str, writer: ZmqWriter) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let addr = addr.to_owned();

        thread::spawn(move || {
            let (_, sub, _) =
                crate::zmq::zmq::zmq_init(1, "", &[addr]).expect("ZMQ SUB init failed");
            while !stop_clone.load(Ordering::Relaxed) {
                match sub.recv_bytes(0) {
                    Ok(bytes) => {
                        if let Ok(sdr) =
                            serde_json::from_slice::<crate::zmq::data_format::ReceiveSdr>(&bytes)
                        {
                            writer.publish_sdr(sdr);
                        }
                    }
                    Err(e) => {
                        log::warn!("ZMQ SDR recv error: {}", e);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        });

        Self { stop }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub struct ZmqLaserRuntime {
    stop: Arc<AtomicBool>,
}

impl ZmqLaserRuntime {
    pub fn start(addr: &str, writer: LaserObservationWriter) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let addr = addr.to_owned();

        thread::spawn(move || {
            let (_, sub, _) =
                crate::zmq::zmq::zmq_init(1, "", &[addr]).expect("ZMQ SUB init failed");
            while !stop_clone.load(Ordering::Relaxed) {
                match sub.recv_bytes(0) {
                    Ok(bytes) => {
                        if let Ok(laser) =
                            serde_json::from_slice::<crate::zmq::data_format::ReceiveLaser>(&bytes)
                        {
                            let observation = crate::laser::protocol::LaserObservation {
                                detected: laser.detected,
                                center: laser.center,
                                brightness: laser.brightness,
                                contour: laser.contour,
                                candidates: laser
                                    .candidates
                                    .iter()
                                    .map(|c| crate::laser::protocol::ModelCandidate {
                                        score: c.score,
                                        class_id: c.class_id,
                                        bbox: c.bbox,
                                        center: c.center,
                                    })
                                    .collect(),
                                received_at: Some(std::time::Instant::now()),
                            };
                            writer.publish(observation);
                        }
                    }
                    Err(e) => {
                        log::warn!("ZMQ Laser recv error: {}", e);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        });

        Self { stop }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn is_started(&self) -> bool {
        !self.stop.load(Ordering::Relaxed)
    }

    pub fn ensure_started(&mut self) {}
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
