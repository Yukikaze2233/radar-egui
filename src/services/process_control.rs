use std::io;

use crate::script_runner::{self, LaserScript, ScriptRunner};

struct PendingStartAll {
    launch_at: std::time::Instant,
    laser_script: LaserScript,
    camera_device: String,
    enemy_cmd: String,
    stream_cmd: String,
    record_cmd: String,
}

pub struct ProcessControl {
    script_runner: ScriptRunner,
    pending_start_all: Option<PendingStartAll>,
}

impl ProcessControl {
    pub fn new() -> Self {
        Self {
            script_runner: ScriptRunner::new(),
            pending_start_all: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.script_runner.is_running()
    }

    pub fn active(&self) -> Option<LaserScript> {
        self.script_runner.active()
    }

    pub fn daemon_alive(&self) -> bool {
        script_runner::daemon_alive()
    }

    pub fn is_sdr_running(&self) -> bool {
        self.script_runner.is_sdr_running()
    }

    pub fn is_unity_running(&self) -> bool {
        self.script_runner.is_unity_running()
    }

    pub fn has_pending_start_all(&self) -> bool {
        self.pending_start_all.is_some()
    }

    pub fn send_laser_command(&self, cmd: &str) {
        let cmd = cmd.to_owned();
        std::thread::spawn(move || {
            if let Err(e) = script_runner::send_fifo(&cmd) {
                log::warn!("Failed to send laser command '{}': {}", cmd, e);
            }
        });
    }

    pub fn start_script(&mut self, script: LaserScript, camera_device: &str) -> io::Result<()> {
        self.cancel_pending_start_all();
        self.script_runner.start(script, camera_device)
    }

    pub fn start_script_with_daemon_config(
        &mut self,
        script: LaserScript,
        camera_device: &str,
        enemy_cmd: String,
        stream_cmd: String,
        record_cmd: String,
    ) -> io::Result<()> {
        self.cancel_pending_start_all();
        self.script_runner.start(script, camera_device)?;

        if script.is_daemon() {
            Self::spawn_start_all_commands(enemy_cmd, stream_cmd, record_cmd);
        }

        Ok(())
    }

    pub fn stop_script(&mut self) {
        self.cancel_pending_start_all();
        self.script_runner.stop();
    }

    pub fn start_sdr(&mut self, enemy_color: &str) -> io::Result<()> {
        self.cancel_pending_start_all();
        self.script_runner.start_sdr(enemy_color)
    }

    pub fn stop_sdr(&mut self) {
        self.cancel_pending_start_all();
        self.script_runner.stop_sdr();
    }

    pub fn start_unity(&mut self) -> io::Result<()> {
        self.script_runner.start_unity()
    }

    pub fn stop_unity(&mut self) {
        self.script_runner.stop_unity();
    }

    pub fn schedule_start_all(
        &mut self,
        sdr_enemy_color: &str,
        camera_device: &str,
        enemy_cmd: String,
        stream_cmd: String,
        record_cmd: String,
    ) -> io::Result<()> {
        self.cancel_pending_start_all();
        self.script_runner.start_sdr(sdr_enemy_color)?;
        self.pending_start_all = Some(PendingStartAll {
            launch_at: std::time::Instant::now() + std::time::Duration::from_secs(1),
            laser_script: LaserScript::Competition,
            camera_device: camera_device.to_owned(),
            enemy_cmd,
            stream_cmd,
            record_cmd,
        });
        Ok(())
    }

    pub fn stop_all(&mut self) {
        self.cancel_pending_start_all();
        self.script_runner.stop_all();
    }

    pub fn cancel_pending_start_all(&mut self) {
        self.pending_start_all = None;
    }

    pub fn trigger_pending_start_all(&mut self) {
        let Some(pending) = self.pending_start_all.take() else {
            return;
        };

        if std::time::Instant::now() < pending.launch_at {
            self.pending_start_all = Some(pending);
            return;
        }

        if let Err(e) = self
            .script_runner
            .start(pending.laser_script, &pending.camera_device)
        {
            log::error!("Start All failed: {}", e);
            return;
        }

        Self::spawn_start_all_commands(pending.enemy_cmd, pending.stream_cmd, pending.record_cmd);
    }

    fn spawn_start_all_commands(enemy_cmd: String, stream_cmd: String, record_cmd: String) {
        std::thread::spawn(move || {
            for _ in 0..100 {
                let ok = script_runner::send_fifo(&enemy_cmd).is_ok()
                    && script_runner::send_fifo(&stream_cmd).is_ok()
                    && script_runner::send_fifo(&record_cmd).is_ok();
                if ok {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            log::warn!("Timed out sending config after Start All");
        });
    }
}
