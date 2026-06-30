use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::laser::protocol::LaserObservation;
use crate::zmq::data_format::ReceiveSdr;

#[derive(Default)]
struct SdrFeedState {
    signal: ReceiveSdr,
    metadata: SdrFeedMetadata,
}

#[derive(Clone)]
pub struct SdrFeedReader {
    inner: Arc<Mutex<SdrFeedState>>,
}

#[derive(Clone)]
pub struct SdrFeedWriter {
    inner: Arc<Mutex<SdrFeedState>>,
}

#[derive(Clone, Default)]
pub struct SdrFeedMetadata {
    pub packet_count: u64,
    pub version: u64,
    pub last_packet_at: Option<Instant>,
}

impl SdrFeedMetadata {
    pub fn mark_packet(&mut self) {
        self.packet_count += 1;
        self.version += 1;
        self.last_packet_at = Some(Instant::now());
    }
}

impl Default for SdrFeedReader {
    fn default() -> Self {
        Self::new_pair().0
    }
}

impl SdrFeedReader {
    pub fn new_pair() -> (Self, SdrFeedWriter) {
        let inner = Arc::new(Mutex::new(SdrFeedState::default()));

        (
            Self {
                inner: inner.clone(),
            },
            SdrFeedWriter { inner },
        )
    }

    pub fn snapshot(&self) -> Option<SdrSnapshot> {
        let state = self.inner.lock().ok()?;

        Some(SdrSnapshot {
            signal: state.signal.clone(),
            metadata: state.metadata.clone(),
        })
    }

    pub fn reset_metadata(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.metadata.packet_count = 0;
            state.metadata.last_packet_at = None;
        }
    }
}

impl SdrFeedWriter {
    pub fn publish(&self, signal: ReceiveSdr) {
        if let Ok(mut state) = self.inner.lock() {
            state.signal = signal;
            state.metadata.mark_packet();
        }
    }
}

pub struct SdrSnapshot {
    pub signal: ReceiveSdr,
    pub metadata: SdrFeedMetadata,
}

#[derive(Clone)]
pub struct LaserObservationReader {
    inner: Arc<Mutex<LaserObservation>>,
}

#[derive(Clone)]
pub struct LaserObservationWriter {
    inner: Arc<Mutex<LaserObservation>>,
}

impl Default for LaserObservationReader {
    fn default() -> Self {
        Self::new_pair().0
    }
}

impl LaserObservationReader {
    pub fn new_pair() -> (Self, LaserObservationWriter) {
        let inner = Arc::new(Mutex::new(LaserObservation::default()));

        (
            Self {
                inner: inner.clone(),
            },
            LaserObservationWriter { inner },
        )
    }

    pub fn snapshot(&self) -> Option<LaserSnapshot> {
        self.inner.lock().ok().map(|state| {
            let observation = state.clone();
            let online = observation.is_online();

            LaserSnapshot {
                observation,
                online,
            }
        })
    }
}

impl LaserObservationWriter {
    pub fn publish(&self, observation: LaserObservation) {
        if let Ok(mut state) = self.inner.lock() {
            *state = observation;
        }
    }
}

pub struct LaserSnapshot {
    pub observation: LaserObservation,
    pub online: bool,
}

use crate::pointcloud::protocol::PointCloudFrame;

#[derive(Clone)]
pub struct PointCloudFrameReader {
    inner: Arc<Mutex<Option<PointCloudFrame>>>,
}

#[derive(Clone)]
pub struct PointCloudFrameWriter {
    inner: Arc<Mutex<Option<PointCloudFrame>>>,
}

impl Default for PointCloudFrameReader {
    fn default() -> Self {
        Self::new_pair().0
    }
}

impl PointCloudFrameReader {
    pub fn new_pair() -> (Self, PointCloudFrameWriter) {
        let inner = Arc::new(Mutex::new(None));
        (
            Self {
                inner: inner.clone(),
            },
            PointCloudFrameWriter { inner },
        )
    }

    pub fn with_frame<R>(&self, read: impl FnOnce(Option<&PointCloudFrame>) -> R) -> Option<R> {
        let frame = self.inner.lock().ok()?;
        let snapshot = frame.clone();
        drop(frame);
        Some(read(snapshot.as_ref()))
    }
}

impl PointCloudFrameWriter {
    pub fn update(&self, frame: PointCloudFrame) {
        if let Ok(mut state) = self.inner.lock() {
            *state = Some(frame);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut state) = self.inner.lock() {
            *state = None;
        }
    }
}
