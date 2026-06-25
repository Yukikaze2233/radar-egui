use crate::sdr::protocol::RoboMasterSignalInfo;

#[cfg(feature = "rerun")]
use rerun as rr;

pub struct RerunVisualizer {
    #[cfg(feature = "rerun")]
    rec: Option<rr::RecordingStream>,
}

impl RerunVisualizer {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rerun")]
            rec: None,
        }
    }

    #[cfg(feature = "rerun")]
    fn ensure_connected(&mut self) -> Option<rr::RecordingStream> {
        if let Some(rec) = &self.rec {
            return Some(rec.clone());
        }
        if let Ok(rec) = rr::RecordingStreamBuilder::new("radar-egui").connect_grpc() {
            self.rec = Some(rec.clone());
            return Some(rec);
        }
        None
    }

    pub fn set_frame_sequence(&mut self, frame: i64) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let _ = rec.set_time_sequence("frame", frame);
        }
        #[cfg(not(feature = "rerun"))]
        let _ = frame;
    }

    pub fn log_robot_positions(&mut self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let robots = [
                ("hero", _info.hero_position),
                ("engineer", _info.engineer_position),
                ("infantry1", _info.infantry_position_1),
                ("infantry2", _info.infantry_position_2),
                ("drone", _info.drone_position),
                ("sentinel", _info.sentinel_position),
            ];

            for (name, pos) in robots {
                let entity_path = format!("world/robots/{}", name);
                let _ = rec.log(
                    entity_path.as_str(),
                    &rr::Points3D::new([(pos[0] as f32 / 100.0, 0.0, pos[1] as f32 / 100.0)])
                        .with_radii([0.1])
                        .with_colors([rr::Color::from_rgb(100, 200, 255)]),
                );
            }
        }
    }

    pub fn log_blood(&mut self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let blood_data = [
                ("hero", _info.hero_blood),
                ("engineer", _info.engineer_blood),
                ("infantry1", _info.infantry_blood_1),
                ("infantry2", _info.infantry_blood_2),
                ("saven", _info.saven_blood),
                ("sentinel", _info.sentinel_blood),
            ];

            for (name, blood) in blood_data {
                let entity_path = format!("world/stats/blood/{}", name);
                let _ = rec.log(entity_path.as_str(), &rr::Scalars::new([blood as f64]));
            }
        }
    }

    pub fn log_economy(&mut self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let _ = rec.log(
                "world/stats/economy/remain",
                &rr::Scalars::new([_info.economic_remain as f64]),
            );
            let _ = rec.log(
                "world/stats/economy/total",
                &rr::Scalars::new([_info.economic_total as f64]),
            );
        }
    }

    pub fn log_all(&mut self, info: &RoboMasterSignalInfo) {
        self.log_robot_positions(info);
        self.log_blood(info);
        self.log_economy(info);
    }
}

#[cfg(feature = "rerun")]
impl RerunVisualizer {
    pub fn recording_stream(&mut self) -> Option<rr::RecordingStream> {
        self.ensure_connected()
    }
}

#[cfg(not(feature = "rerun"))]
impl RerunVisualizer {
    pub fn recording_stream(&mut self) -> Option<()> {
        None
    }
}
