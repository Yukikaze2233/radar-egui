use crate::protocol::RoboMasterSignalInfo;

#[cfg(feature = "rerun")]
use rerun as rr;

pub struct RerunVisualizer {
    #[cfg(feature = "rerun")]
    rec: Option<rr::RecordingStream>,
}

impl RerunVisualizer {
    pub fn new() -> Self {
        #[cfg(feature = "rerun")]
        {
            let rec = rr::RecordingStreamBuilder::new("radar-egui")
                .connect_tcp()
                .ok();
            Self { rec }
        }
        #[cfg(not(feature = "rerun"))]
        Self {}
    }

    pub fn log_robot_positions(&self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = &self.rec {
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

    pub fn log_blood(&self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = &self.rec {
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
                let _ = rec.log(
                    entity_path.as_str(),
                    &rr::Scalar::new(blood as f64),
                );
            }
        }
    }

    pub fn log_economy(&self, _info: &RoboMasterSignalInfo) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = &self.rec {
            let _ = rec.log(
                "world/stats/economy/remain",
                &rr::Scalar::new(_info.economic_remain as f64),
            );
            let _ = rec.log(
                "world/stats/economy/total",
                &rr::Scalar::new(_info.economic_total as f64),
            );
        }
    }

    pub fn log_all(&self, info: &RoboMasterSignalInfo) {
        self.log_robot_positions(info);
        self.log_blood(info);
        self.log_economy(info);
    }
}
