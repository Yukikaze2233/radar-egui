//! Point Cloud SHM 协议定义
//!
//! 共享内存布局（与 laser/video.rs 双缓冲模式一致）：
//!
//! ┌─ ShmHeader (64 B) ────────────────────────────────────┐
//! │ +0   u32 magic       = 0x50434446 ("PCDF")             │
//! │ +4   u32 version     = 1                               │
//! │ +8   u32 point_count (本帧实际点数)                     │
//! │ +12  u32 max_points  (缓冲区容量)                       │
//! │ +16  atomic<u32> frame_seq (release 写入, acquire 读取) │
//! │ +20  atomic<u32> write_idx  (0 或 1)                   │
//! │ +24  u32 stride      (每点字节数，典型 16)              │
//! │ +28  u8[36] _pad                                      │
//! ├────────────────────────────────────────────────────────┤
//! │ buf_[0]  (max_points × stride bytes)                  │
//! ├────────────────────────────────────────────────────────┤
//! │ buf_[1]  (max_points × stride bytes)                  │
//! └────────────────────────────────────────────────────────┘
//!
//! 每点格式 (stride = 16): [f32 x, f32 y, f32 z, u32 rgba] (LE)

/// SHM magic: "PCDF"
pub const SHM_MAGIC: u32 = 0x50434446;
/// SHM 头部大小
pub const HEADER_SIZE: usize = 64;
/// 默认每点字节数
pub const DEFAULT_STRIDE: u32 = 16;

/// 一帧点云在 Rust 侧的表示
#[derive(Debug, Clone, Default)]
pub struct PointCloudFrame {
    pub points: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 4]>,
    pub normals: Vec<[f32; 3]>,
    pub seq: u32,
}

impl PointCloudFrame {
    /// 从 SHM 原始字节解析一帧点云
    ///
    /// `data` 是一整块 buffer（max_points × stride 字节），
    /// 只有前 `point_count` 个点有效。
    pub fn from_raw(data: &[u8], point_count: u32, stride: u32) -> Self {
        let count = point_count as usize;
        let stride = stride as usize;
        let mut points = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);
        let mut normals = Vec::with_capacity(count);

        for i in 0..count {
            let off = i * stride;
            if off + 12 > data.len() {
                break;
            }
            let x = f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
            let y = f32::from_le_bytes([
                data[off + 4],
                data[off + 5],
                data[off + 6],
                data[off + 7],
            ]);
            let z = f32::from_le_bytes([
                data[off + 8],
                data[off + 9],
                data[off + 10],
                data[off + 11],
            ]);
            points.push([x, y, z]);

            if stride >= 16 && off + 16 <= data.len() {
                let r = data[off + 12];
                let g = data[off + 13];
                let b = data[off + 14];
                let a = data[off + 15];
                colors.push([r, g, b, a]);
            } else {
                colors.push([255, 255, 255, 255]);
            }

            if stride >= 28 && off + 28 <= data.len() {
                let nx = f32::from_le_bytes([
                    data[off + 16], data[off + 17], data[off + 18], data[off + 19],
                ]);
                let ny = f32::from_le_bytes([
                    data[off + 20], data[off + 21], data[off + 22], data[off + 23],
                ]);
                let nz = f32::from_le_bytes([
                    data[off + 24], data[off + 25], data[off + 26], data[off + 27],
                ]);
                normals.push([nx, ny, nz]);
            } else {
                normals.push([0.0, 0.0, 1.0]);
            }
        }

        Self {
            points,
            colors,
            normals,
            seq: 0,
        }
    }
}
