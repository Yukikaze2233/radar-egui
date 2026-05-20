//! 比赛组件进程管理模块
//!
//! 从 radar-egui 中 spawn 比赛所需的三个外部进程：
//!   - laser_guidance  脚本 (competition / preview / stream / record)
//!   - SDR 数据桥接    (alliance_radar_sdr/tcp/tcp_launch.py)
//!   - Unity RADAR     (RADAR_APP/RADAR.x86_64)

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const LASER_GUIDANCE_ROOT_ENV: &str = "LASER_GUIDANCE_ROOT";
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

    pub fn start(&mut self, script: LaserScript, device: &str) -> io::Result<()> {
        // 拿走旧状态，后台清理（不阻塞 UI）
        let old_active = self.active.take();
        let old_child = self.child.take();

        if let Some(active) = old_active {
            if active.is_daemon() {
                let _ = std::thread::spawn(move || {
                    send_fifo("quit").ok();
                });
            }
        }
        if let Some(mut child) = old_child {
            let _ = std::thread::spawn(move || {
                let _ = child.kill();
                let _ = child.wait();
                log::info!("Stopped old laser script wrapper");
            });
        }

        let laser_root = laser_guidance_root()?;
        let path = laser_root.join(".script").join(script.script_name());
        let child = Command::new(&path)
            .current_dir(&laser_root)
            .env("LASER_CAMERA_DEVICE", device)
            .env("LASER_HEADLESS", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        log::info!(
            "Started laser script: {:?} from {} (pid={})",
            script,
            laser_root.display(),
            child.id()
        );
        self.child = Some(child);
        self.active = Some(script);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(active) = self.active {
            if active.is_daemon() {
                // 1. 优雅退出：通过 FIFO 通知 daemon
                send_fifo("quit").ok();
                std::thread::sleep(std::time::Duration::from_millis(800));
                // 2. 兜底强杀 (SIGKILL)：daemon 被 disown，wrapper kill 无效
                for name in &["tool_competition", "tool_preview", "ffplay"] {
                    let _ = Command::new("pkill").args(["-9", "-f", name]).output();
                }
                // 3. 清理 FIFO，避免残留阻塞下次启动
                let _ = std::fs::remove_file(LASER_FIFO);
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

    /// 启动 SDR 数据桥接 (thread_init.py)
    ///
    /// 从 SDR 仓库根目录运行，PYTHONPATH=. 解决 parser/tcp 导入。
    pub fn start_sdr(&mut self) -> io::Result<()> {
        self.stop_sdr();

        let sdr_dir = PathBuf::from(SDR_REPO);
        let child = Command::new("python3")
            .arg("thread_init.py")
            .current_dir(&sdr_dir)
            .env("PYTHONPATH", ".")
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

fn laser_guidance_root() -> io::Result<PathBuf> {
    if let Some(root) = std::env::var_os(LASER_GUIDANCE_ROOT_ENV) {
        return valid_laser_root(PathBuf::from(root));
    }

    let mut candidates = Vec::new();
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        let manifest_dir = Path::new(manifest_dir);
        candidates.push(manifest_dir.join("../../laser_guidance"));
        candidates.push(manifest_dir.join("../laser_guidance"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("../../laser_guidance"));
        candidates.push(cwd.join("../laser_guidance"));
    }

    for candidate in candidates {
        if let Ok(root) = valid_laser_root(candidate) {
            return Ok(root);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "laser_guidance root not found; set {LASER_GUIDANCE_ROOT_ENV} to the laser_guidance repo"
        ),
    ))
}

fn valid_laser_root(path: PathBuf) -> io::Result<PathBuf> {
    let script_dir = path.join(".script");
    if script_dir.join("competition").is_file() {
        path.canonicalize()
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not contain .script/competition", path.display()),
        ))
    }
}

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
