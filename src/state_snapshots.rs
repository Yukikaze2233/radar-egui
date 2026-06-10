use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::laser_protocol::LaserObservation;
use crate::protocol::RoboMasterSignalInfo;

#[derive(Default)]
struct RadarFeedState {
    signal: RoboMasterSignalInfo,
    metadata: RadarFeedMetadata,
}

#[derive(Clone)]
pub struct RadarFeedReader {
    inner: Arc<Mutex<RadarFeedState>>,
}

#[derive(Clone)]
pub struct RadarFeedWriter {
    inner: Arc<Mutex<RadarFeedState>>,
}

#[derive(Clone, Default)]
pub struct RadarFeedMetadata {
    pub packet_count: u64,
    pub version: u64,
    pub last_packet_at: Option<Instant>,
}

impl RadarFeedMetadata {
    pub fn mark_packet(&mut self) {
        self.packet_count += 1;
        self.version += 1;
        self.last_packet_at = Some(Instant::now());
    }
}

impl Default for RadarFeedReader {
    fn default() -> Self {
        Self::new_pair().0
    }
}

impl RadarFeedReader {
    pub fn new_pair() -> (Self, RadarFeedWriter) {
        let inner = Arc::new(Mutex::new(RadarFeedState::default()));

        (
            Self {
                inner: inner.clone(),
            },
            RadarFeedWriter { inner },
        )
    }

    pub fn snapshot(&self) -> Option<RadarSnapshot> {
        let state = self.inner.lock().ok()?;

        Some(RadarSnapshot {
            signal: state.signal.clone(),
            metadata: state.metadata.clone(),
        })
    }

    pub fn reset_metadata(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.metadata = RadarFeedMetadata::default();
        }
    }
}

impl RadarFeedWriter {
    pub fn publish(&self, signal: RoboMasterSignalInfo) {
        if let Ok(mut state) = self.inner.lock() {
            state.signal = signal;
            state.metadata.mark_packet();
        }
    }
}

pub struct RadarSnapshot {
    pub signal: RoboMasterSignalInfo,
    pub metadata: RadarFeedMetadata,
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
