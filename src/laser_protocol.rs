//! Laser Guidance UDP 协议解析
//!
//! 协议格式参见 docs/laser-protocol.md

use std::time::{Duration, Instant};

/// 模型检测候选
#[derive(Debug, Clone, Default)]
pub struct ModelCandidate {
    /// 置信度 0.0~1.0
    pub score: f32,
    /// 类别: 0=purple, 1=red, 2=blue
    pub class_id: i32,
    /// 边界框 [x, y, w, h]
    pub bbox: [f32; 4],
    /// 中心坐标 [x, y]
    pub center: [f32; 2],
}

/// 激光引导目标观测数据
#[derive(Debug, Clone, Default)]
pub struct LaserObservation {
    /// 是否检测到目标
    pub detected: bool,
    /// 目标中心坐标 [x, y] (像素)
    pub center: [f32; 2],
    /// 亮度值
    pub brightness: f32,
    /// 轮廓点序列
    pub contour: Vec<[f32; 2]>,
    /// 模型候选列表
    pub candidates: Vec<ModelCandidate>,
    /// 接收时间
    pub received_at: Option<Instant>,
}

impl LaserObservation {
    /// 获取接收后经过的时间
    pub fn elapsed(&self) -> Option<Duration> {
        self.received_at.map(|t| t.elapsed())
    }

    /// 是否在线（2 秒内有数据）
    pub fn is_online(&self) -> bool {
        self.elapsed().is_some_and(|e| e.as_secs() < 2)
    }

    /// 获取最佳候选（score 最高的）
    pub fn best_candidate(&self) -> Option<&ModelCandidate> {
        self.candidates
            .iter()
            .filter(|c| c.score > 0.25)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 获取类别名称
    pub fn class_name(class_id: i32) -> &'static str {
        match class_id {
            0 => "Purple",
            1 => "Red",
            2 => "Blue",
            _ => "?",
        }
    }
}

/// 协议常量
const MAGIC: u16 = 0x4C47; // "LG"
const VERSION: u8 = 1;
const HEADER_SIZE: usize = 16;

/// 解析 Laser UDP 包
pub fn parse_laser_packet(data: &[u8]) -> Option<LaserObservation> {
    if data.len() < HEADER_SIZE + 5 {
        return None;
    }

    // Header
    let magic = u16::from_le_bytes([data[0], data[1]]);
    if magic != MAGIC {
        return None;
    }

    let version = data[2];
    if version != VERSION {
        return None;
    }

    // _seq = data[3];
    // _timestamp = u64::from_le_bytes(data[4..12].try_into().ok()?);
    let payload_len = u32::from_le_bytes(data[12..16].try_into().ok()?) as usize;

    if data.len() < HEADER_SIZE + payload_len {
        return None;
    }

    let mut offset = HEADER_SIZE;

    // detected
    let detected = data[offset] != 0;
    offset += 1;

    // center
    let center_x = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
    offset += 4;
    let center_y = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
    offset += 4;

    // brightness
    let brightness = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
    offset += 4;

    // contour
    let contour_count = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;

    let mut contour = Vec::with_capacity(contour_count);
    for _ in 0..contour_count {
        if offset + 8 > data.len() {
            return None;
        }
        let x = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let y = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        contour.push([x, y]);
    }

    // candidates
    let cand_count = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;

    let mut candidates = Vec::with_capacity(cand_count);
    for _ in 0..cand_count {
        if offset + 28 > data.len() {
            return None;
        }
        let score = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let class_id = i32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let bbox_x = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let bbox_y = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let bbox_w = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let bbox_h = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let cx = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;
        let cy = f32::from_le_bytes(data[offset..offset + 4].try_into().ok()?);
        offset += 4;

        candidates.push(ModelCandidate {
            score,
            class_id,
            bbox: [bbox_x, bbox_y, bbox_w, bbox_h],
            center: [cx, cy],
        });
    }

    Some(LaserObservation {
        detected,
        center: [center_x, center_y],
        brightness,
        contour,
        candidates,
        received_at: Some(Instant::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(parse_laser_packet(&[]).is_none());
    }

    #[test]
    fn test_parse_too_short() {
        assert!(parse_laser_packet(&[0u8; 10]).is_none());
    }

    #[test]
    fn test_parse_invalid_magic() {
        let mut buf = vec![0u8; 32];
        buf[0] = 0x00;
        buf[1] = 0x00;
        assert!(parse_laser_packet(&buf).is_none());
    }

    #[test]
    fn test_parse_valid_no_data() {
        let mut buf = vec![0u8; 32];
        // magic
        buf[0] = 0x47;
        buf[1] = 0x4C;
        // version
        buf[2] = 1;
        // payload_len = 5 (detected + center_x + center_y + brightness + contour_count + candidates_count)
        buf[12] = 21;

        let obs = parse_laser_packet(&buf).unwrap();
        assert!(!obs.detected);
        assert_eq!(obs.center, [0.0, 0.0]);
        assert_eq!(obs.contour.len(), 0);
        assert_eq!(obs.candidates.len(), 0);
    }
}
