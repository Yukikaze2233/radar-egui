use super::{ConnectionStatus, RadarApp};
use crate::state::RadarSnapshot;

impl RadarApp {
    pub(super) fn update_connection_status(&mut self, snapshot: Option<&RadarSnapshot>) {
        if let Some(snapshot) = snapshot {
            self.data_count = snapshot.metadata.packet_count;
            self.last_update = snapshot.metadata.last_packet_at;

            if snapshot.metadata.version > self.last_logged_radar_version {
                self.rerun_viz
                    .set_frame_sequence(snapshot.metadata.version as i64);
                self.rerun_viz.log_all(&snapshot.signal);
                self.last_logged_radar_version = snapshot.metadata.version;
            }

            if let Some(last) = snapshot.metadata.last_packet_at {
                if last.elapsed().as_secs() <= 5 {
                    self.connection_status = ConnectionStatus::Connected;
                    self.error_message = None;
                } else {
                    self.connection_status = ConnectionStatus::Disconnected;
                    self.error_message = Some("Connection lost".to_string());
                }
            } else {
                self.connection_status = ConnectionStatus::Disconnected;
                self.error_message = None;
            }
        } else {
            self.connection_status = ConnectionStatus::Disconnected;
            self.error_message = None;
        }
    }
}
