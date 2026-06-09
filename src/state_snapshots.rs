use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::laser_protocol::LaserObservation;
use crate::protocol::RoboMasterSignalInfo;

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

pub struct RadarSnapshot {
    pub signal: RoboMasterSignalInfo,
    pub metadata: RadarFeedMetadata,
}

impl RadarSnapshot {
    pub fn capture(
        shared_signal: &Arc<Mutex<RoboMasterSignalInfo>>,
        shared_metadata: &Arc<Mutex<RadarFeedMetadata>>,
    ) -> Option<Self> {
        let signal = shared_signal.lock().ok()?.clone();
        let metadata = shared_metadata.lock().ok()?.clone();

        Some(Self { signal, metadata })
    }
}

pub struct LaserSnapshot {
    pub observation: LaserObservation,
    pub online: bool,
}

impl LaserSnapshot {
    pub fn capture(shared: &Arc<Mutex<LaserObservation>>) -> Option<Self> {
        shared.lock().ok().map(|state| {
            let observation = state.clone();
            let online = observation.is_online();
            Self {
                observation,
                online,
            }
        })
    }
}
