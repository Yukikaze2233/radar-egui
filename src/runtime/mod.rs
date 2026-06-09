use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread;

use tokio::sync::watch;

use crate::laser_protocol::LaserObservation;
use crate::protocol::RoboMasterSignalInfo;
use crate::state_snapshots::RadarFeedMetadata;
use crate::tcp_client;
use crate::udp_client;
use crate::video_stream::{self, VideoFrame};

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

pub struct RadarRuntime {
    shutdown_tx: watch::Sender<bool>,
}

impl RadarRuntime {
    pub fn start(
        addr: impl Into<String>,
        shared: Arc<Mutex<RoboMasterSignalInfo>>,
        metadata: Arc<Mutex<RadarFeedMetadata>>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let addr = addr.into();
        spawn_runtime_task(move || async move {
            tcp_client::run_signal_client(&addr, shared, metadata, shutdown_rx).await;
        });

        Self { shutdown_tx }
    }

    pub fn restart(
        &mut self,
        addr: impl Into<String>,
        shared: Arc<Mutex<RoboMasterSignalInfo>>,
        metadata: Arc<Mutex<RadarFeedMetadata>>,
    ) {
        let _ = self.shutdown_tx.send(true);
        *self = Self::start(addr, shared, metadata);
    }
}

pub struct LaserRuntime {
    shutdown_tx: watch::Sender<bool>,
    started: bool,
}

impl Default for LaserRuntime {
    fn default() -> Self {
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);

        Self {
            shutdown_tx,
            started: false,
        }
    }
}

impl LaserRuntime {
    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn ensure_started(&mut self, port: u16, shared: Arc<Mutex<LaserObservation>>) {
        if self.started {
            return;
        }

        self.started = true;
        self.spawn(port, shared);
    }

    pub fn restart(&mut self, port: u16, shared: Arc<Mutex<LaserObservation>>) {
        let _ = self.shutdown_tx.send(true);
        self.started = true;
        self.spawn(port, shared);
    }

    fn spawn(&mut self, port: u16, shared: Arc<Mutex<LaserObservation>>) {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;

        spawn_runtime_task(move || async move {
            udp_client::run_laser_client(port, shared, shutdown_rx).await;
        });
    }
}

pub struct VideoRuntime {
    shutdown_tx: watch::Sender<bool>,
    started: bool,
}

impl Default for VideoRuntime {
    fn default() -> Self {
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);

        Self {
            shutdown_tx,
            started: false,
        }
    }
}

impl VideoRuntime {
    pub fn ensure_started(&mut self, shared: Arc<Mutex<Option<VideoFrame>>>) {
        if self.started {
            return;
        }

        self.started = true;
        let _ = self.shutdown_tx.send(true);

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;

        spawn_runtime_task(move || async move {
            video_stream::run_video_client(shared, shutdown_rx).await;
        });
    }
}
