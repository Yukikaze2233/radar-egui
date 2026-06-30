use std::future::Future;
use std::thread;

use tokio::sync::watch;

use crate::laser::observer;
use crate::laser::video::{self, VideoFrameWriter};
use crate::pointcloud::reader;
// use crate::sdr::client; // TODO: replace with ZMQ SUB runtime
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

// TODO: replace with ZMQ SUB runtime
// pub struct RadarRuntime { ... }

pub struct LaserRuntime {
    shutdown_tx: watch::Sender<bool>,
    started: bool,
    writer: LaserObservationWriter,
}

impl LaserRuntime {
    pub fn new(writer: LaserObservationWriter) -> Self {
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

    pub fn ensure_started(&mut self, port: u16) {
        if self.started {
            return;
        }

        self.started = true;
        self.spawn(port);
    }

    pub fn restart(&mut self, port: u16) {
        let _ = self.shutdown_tx.send(true);
        self.started = true;
        self.spawn(port);
    }

    fn spawn(&mut self, port: u16) {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;
        let writer = self.writer.clone();

        spawn_runtime_task(move || async move {
            observer::run_laser_client(port, writer, shutdown_rx).await;
        });
    }
}

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
