use crate::zmq::data_format::ReceiveSdr;

#[cfg(feature = "rerun")]
use rerun as rr;

pub struct RerunVisualizer {
    #[cfg(feature = "rerun")]
    rec: Option<rr::RecordingStream>,
    #[cfg(feature = "rerun")]
    last_connect_attempt: Option<std::time::Instant>,
}

impl RerunVisualizer {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rerun")]
            rec: None,
            #[cfg(feature = "rerun")]
            last_connect_attempt: None,
        }
    }

    #[cfg(feature = "rerun")]
    fn ensure_connected(&mut self) -> Option<rr::RecordingStream> {
        if let Some(rec) = &self.rec {
            return Some(rec.clone());
        }
        let now = std::time::Instant::now();
        if let Some(last) = self.last_connect_attempt {
            if now.duration_since(last).as_secs() < 5 {
                return None;
            }
        }
        self.last_connect_attempt = Some(now);
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

    pub fn log_robot_positions(&mut self, _info: &ReceiveSdr) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let robots: &[(&str, [i16; 2])] = &[
                ("hero", [_info.hero_x, _info.hero_y]),
                ("engineer", [_info.engineer_x, _info.engineer_y]),
                ("infantry1", [_info.infantry_3_x, _info.infantry_3_y]),
                ("infantry2", [_info.infantry_4_x, _info.infantry_4_y]),
                ("drone", [_info.aerial_x, _info.aerial_y]),
                ("sentinel", [_info.sentry_x, _info.sentry_y]),
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

    pub fn log_blood(&mut self, _info: &ReceiveSdr) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let blood_data = [
                ("hero", _info.hero_blood),
                ("engineer", _info.engineer_blood),
                ("infantry1", _info.infantry_3_blood),
                ("infantry2", _info.infantry_4_blood),
                ("saven", _info.reserved),
                ("sentinel", _info.sentry_blood),
            ];

            for (name, blood) in blood_data {
                let entity_path = format!("world/stats/blood/{}", name);
                let _ = rec.log(entity_path.as_str(), &rr::Scalars::new([blood as f64]));
            }
        }
    }

    pub fn log_economy(&mut self, _info: &ReceiveSdr) {
        #[cfg(feature = "rerun")]
        if let Some(rec) = self.ensure_connected() {
            let _ = rec.log(
                "world/stats/economy/remain",
                &rr::Scalars::new([_info.remaining_gold as f64]),
            );
            let _ = rec.log(
                "world/stats/economy/total",
                &rr::Scalars::new([_info.total_gold as f64]),
            );
        }
    }

    pub fn log_all(&mut self, info: &ReceiveSdr) {
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
