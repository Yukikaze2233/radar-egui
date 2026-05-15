//! 比赛组件进程管理模块
//!
//! 从 radar-egui 中 spawn 比赛所需的三个外部进程：
//!   - laser_guidance  脚本 (competition / preview / stream / record)
//!   - SDR 数据桥接    (alliance_radar_sdr/tcp/tcp_launch.py)
//!   - Unity RADAR     (RADAR_APP/RADAR.x86_64)

use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

const LASER_SCRIPTS_DIR: &str = "../laser_guidance/.script";
const LASER_FIFO: &str = "/tmp/laser_cmd";
const SDR_REPO: &str = "../alliance_radar_sdr";
const UNITY_BIN: &str = "../RADAR_APP/RADAR.x86_64";

// ── LaserScript ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LaserScript {
    Competition,
    Preview,
    Stream,
    Record,
}

impl LaserScript {
    pub fn label(&self) -> &'static str {
        match self {
            LaserScript::Competition => "Competition",
            LaserScript::Preview => "Preview",
            LaserScript::Stream => "Stream",
            LaserScript::Record => "Record",
        }
    }

    fn script_name(&self) -> &'static str {
        match self {
            LaserScript::Competition => "competition",
            LaserScript::Preview => "preview",
            LaserScript::Stream => "stream",
            LaserScript::Record => "record",
        }
    }

    pub fn is_daemon(&self) -> bool {
        matches!(self, LaserScript::Competition | LaserScript::Stream)
    }
}

// ── ScriptRunner ─────────────────────────────────────────────────────────────

pub struct ScriptRunner {
    // Laser
    child: Option<Child>,
    active: Option<LaserScript>,

    // SDR bridge
    sdr_child: Option<Child>,

    // Unity RADAR
    unity_child: Option<Child>,
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {
            child: None,
            active: None,
            sdr_child: None,
            unity_child: None,
        }
    }

    // ── Laser ────────────────────────────────────────────────────────────────

    pub fn start(&mut self, script: LaserScript) -> io::Result<()> {
        self.stop();

        let path = PathBuf::from(LASER_SCRIPTS_DIR).join(script.script_name());
        let child = Command::new(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        log::info!("Started laser script: {:?} (pid={})", script, child.id());
        self.child = Some(child);
        self.active = Some(script);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(active) = self.active {
            if active.is_daemon() {
                send_fifo("quit").ok();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            log::info!("Stopped laser script wrapper");
        }
        self.active = None;
    }

    pub fn is_running(&self) -> bool {
        self.active.is_some()
    }

    pub fn active(&self) -> Option<LaserScript> {
        self.active
    }

    // ── SDR ──────────────────────────────────────────────────────────────────

    /// 启动 SDR 数据桥接 (tcp_launch.py)
    ///
    /// cd 到 tcp/ 子目录解决 `from tcp_comm` 同级导入，
    /// PYTHONPATH=.. 解决 `from parser.xxx` 跨目录导入。
    pub fn start_sdr(&mut self) -> io::Result<()> {
        self.stop_sdr();

        let script_dir = PathBuf::from(SDR_REPO).join("tcp");
        let child = Command::new("python3")
            .arg("tcp_launch.py")
            .current_dir(&script_dir)
            .env("PYTHONPATH", "..")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        log::info!("Started SDR bridge (pid={})", child.id());
        self.sdr_child = Some(child);
        Ok(())
    }

    pub fn stop_sdr(&mut self) {
        if let Some(mut child) = self.sdr_child.take() {
            let _ = child.kill();
            let _ = child.wait();
            log::info!("Stopped SDR bridge");
        }
    }

    pub fn is_sdr_running(&self) -> bool {
        self.sdr_child.is_some()
    }

    // ── Unity RADAR ──────────────────────────────────────────────────────────

    pub fn start_unity(&mut self) -> io::Result<()> {
        self.stop_unity();

        let child = Command::new(UNITY_BIN)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        log::info!("Started Unity RADAR (pid={})", child.id());
        self.unity_child = Some(child);
        Ok(())
    }

    pub fn stop_unity(&mut self) {
        if let Some(mut child) = self.unity_child.take() {
            let _ = child.kill();
            let _ = child.wait();
            log::info!("Stopped Unity RADAR");
        }
    }

    pub fn is_unity_running(&self) -> bool {
        self.unity_child.is_some()
    }

    // ── 一键比赛 ────────────────────────────────────────────────────────────

    /// 顺序启动 SDR → Laser (需外部传入 script 和 enemy_cmd)
    /// 调用方应自行处理 Laser 启动和 enemy_color FIFO 发送
    pub fn start_all(&mut self, laser_script: LaserScript) -> io::Result<()> {
        self.start_sdr()?;

        // 等 SDR 桥接连上 GNU Radio
        std::thread::sleep(std::time::Duration::from_secs(1));

        self.start(laser_script)
    }

    /// 停止全部进程
    pub fn stop_all(&mut self) {
        self.stop();
        self.stop_sdr();
        self.stop_unity();
    }
}

impl Drop for ScriptRunner {
    fn drop(&mut self) {
        self.stop_all();
    }
}

// ── 静态辅助函数 ────────────────────────────────────────────────────────────

pub fn daemon_alive() -> bool {
    use std::os::unix::fs::FileTypeExt;
    match std::fs::metadata(LASER_FIFO) {
        Ok(meta) => meta.file_type().is_fifo(),
        Err(_) => false,
    }
}

pub fn send_fifo(cmd: &str) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true);
    opts.custom_flags(libc::O_NONBLOCK);

    let mut fifo = opts.open(LASER_FIFO)?;
    writeln!(fifo, "{cmd}")?;
    log::info!("FIFO sent: {}", cmd);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels() {
        assert_eq!(LaserScript::Competition.label(), "Competition");
        assert_eq!(LaserScript::Preview.label(), "Preview");
        assert_eq!(LaserScript::Stream.label(), "Stream");
        assert_eq!(LaserScript::Record.label(), "Record");
    }

    #[test]
    fn test_is_daemon() {
        assert!(LaserScript::Competition.is_daemon());
        assert!(!LaserScript::Preview.is_daemon());
        assert!(LaserScript::Stream.is_daemon());
        assert!(!LaserScript::Record.is_daemon());
    }

    #[test]
    fn test_new_runner_is_idle() {
        let runner = ScriptRunner::new();
        assert!(!runner.is_running());
        assert!(!runner.is_sdr_running());
        assert!(!runner.is_unity_running());
        assert!(runner.active().is_none());
    }
}
