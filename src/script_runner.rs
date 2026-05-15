//! 激光引导脚本调用模块
//!
//! 从 radar-egui 中 spawn laser_guidance 的 .script/ 脚本，
//! 支持 competition / preview / stream / record 四种进程。
//!
//! # 集成方式（手动）
//!
//! 1. `main.rs` 添加 `mod script_runner;`
//! 2. `app.rs` 顶部 `use crate::script_runner::{LaserScript, ScriptRunner, daemon_alive, send_fifo};`
//! 3. `RadarApp` 结构体加字段 `script_runner: ScriptRunner`
//! 4. `Default` 初始化 `script_runner: ScriptRunner::new()`
//! 5. 在 Laser 侧边栏加按钮调用 `start()` / `stop()`

use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// 激光脚本目录，相对于 radar-egui 项目根目录。
/// 默认假设 laser_guidance 仓库在同级目录下。
const LASER_SCRIPTS_DIR: &str = "../laser_guidance/.script";
const LASER_FIFO: &str = "/tmp/laser_cmd";

// ── LaserScript ──────────────────────────────────────────────────────────────

/// 可启动的激光引导脚本类型
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LaserScript {
    /// 比赛模式守护进程（tool_competition + ffplay）
    Competition,
    /// 本地预览（cv::imshow 窗口）
    Preview,
    /// RTP 推流守护进程（tool_preview + ffplay）
    Stream,
    /// 视频录制
    Record,
}

impl LaserScript {
    /// UI 展示用标签
    pub fn label(&self) -> &'static str {
        match self {
            LaserScript::Competition => "Competition",
            LaserScript::Preview => "Preview",
            LaserScript::Stream => "Stream",
            LaserScript::Record => "Record",
        }
    }

    /// 对应的 .script/ 下脚本文件名
    fn script_name(&self) -> &'static str {
        match self {
            LaserScript::Competition => "competition",
            LaserScript::Preview => "preview",
            LaserScript::Stream => "stream",
            LaserScript::Record => "record",
        }
    }

    /// 该脚本是否以 daemon 方式运行（后台 tool_* 进程不随 bash wrapper 退出）
    fn is_daemon(&self) -> bool {
        matches!(self, LaserScript::Competition | LaserScript::Stream)
    }
}

// ── ScriptRunner ─────────────────────────────────────────────────────────────

/// 管理单个激光脚本子进程的生命周期
pub struct ScriptRunner {
    /// bash 封装脚本的子进程句柄。daemon 类脚本的 wrapper 在 ffplay
    /// 关闭后自行退出，但后台 tool_* 进程仍存活。
    child: Option<Child>,
    /// 当前活动的脚本类型
    active: Option<LaserScript>,
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {
            child: None,
            active: None,
        }
    }

    /// 启动脚本（非阻塞），重复调用前会先 `stop()` 旧的。
    ///
    /// - Competition / Stream：spawn bash → 后台启动 tool_* daemon → 自动拉起 ffplay
    /// - Preview：spawn bash → 阻塞在 cv::imshow 直到窗口关闭
    /// - Record：spawn bash → 阻塞在 tool_record 直到录制结束
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

    /// 停止当前脚本：
    /// - daemon 类：先通过 FIFO 发送 "quit" 优雅退出守护进程，再 kill bash wrapper
    /// - 非 daemon 类：直接 kill 子进程
    pub fn stop(&mut self) {
        if let Some(active) = self.active {
            if active.is_daemon() {
                send_fifo("quit").ok();
                // 给 daemon 一点时间清理
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

    /// 是否有脚本正在运行（bash wrapper 存活）
    pub fn is_running(&self) -> bool {
        self.active.is_some()
    }

    /// 当前活动的脚本类型
    pub fn active(&self) -> Option<LaserScript> {
        self.active
    }
}

impl Drop for ScriptRunner {
    fn drop(&mut self) {
        self.stop();
    }
}

// ── 静态辅助函数 ────────────────────────────────────────────────────────────

/// 检查是否有激光守护进程在运行（competition / stream 的 tool_* 进程）
///
/// 通过 FIFO 管道存在性判断，比 pgrep 更轻量。
/// 注意：daemon_alive 为 true 意味着可以通过 `send_fifo` 发送 runtime 命令。
pub fn daemon_alive() -> bool {
    use std::os::unix::fs::FileTypeExt;
    match std::fs::metadata(LASER_FIFO) {
        Ok(meta) => meta.file_type().is_fifo(),
        Err(_) => false,
    }
}

/// 向激光守护进程的 FIFO 发送控制命令
///
/// # 可用命令
///
/// | 命令 | 作用 |
/// |------|------|
/// | `stream on` / `stream off` | 推流开关 |
/// | `record on` / `record off` | 录制开关 |
/// | `enemy red` / `enemy blue` / `enemy auto` | 敌方颜色过滤 |
/// | `backend tensorrt` / `backend onnx` | 推理后端切换 |
/// | `ekf on` / `ekf off` | EKF 跟踪开关 |
/// | `quit` | 优雅退出守护进程 |
///
/// # 示例
///
/// ```ignore
/// send_fifo("stream on")?;
/// send_fifo("enemy red")?;
/// ```
pub fn send_fifo(cmd: &str) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true);
    opts.custom_flags(libc::O_NONBLOCK); // 防止无人读取时阻塞

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
        assert!(runner.active().is_none());
    }
}
