# Laser Guidance UDP 协议

本文档定义 `laser_guidance` 与 `radar-egui` 之间的 UDP 数据传输协议。

## 概述

- **传输方式**: UDP
- **默认地址**: `127.0.0.1:5001`
- **字节序**: 小端序 (Little-Endian)
- **发送频率**: 跟随检测帧率（默认 60 FPS）

## 包结构

```
Offset  Size   Field           Description
──────────────────────────────────────────────────────────────
HEADER (16 bytes)
──────────────────────────────────────────────────────────────
0       2      magic           固定 0x4C47 ("LG")
2       1      version         协议版本，当前 1
3       1      seq             帧序号 0~255 循环
4       8      timestamp       微秒时间戳 (steady_clock)
12      4      payload_len     Payload 字节数

──────────────────────────────────────────────────────────────
PAYLOAD (变长)
──────────────────────────────────────────────────────────────
16      1      detected        是否检测到目标 (0=false, 1=true)
17      4      center_x        目标中心 X (float, 像素)
21      4      center_y        目标中心 Y (float, 像素)
25      4      brightness      亮度值 (float)
29      4      contour_count   轮廓点数量 (u32)
33      8×N    contour[]       轮廓点 {f32 x, f32 y} × N

        4      candidates_count 候选数量 (u32)
        28×M   candidates[]    候选列表 × M
          ├─ 4   score           置信度 (float, 0.0~1.0)
          ├─ 4   class_id        类别 (i32: 0=purple, 1=red, 2=blue)
          ├─ 4   bbox_x          边界框 X (float)
          ├─ 4   bbox_y          边界框 Y (float)
          ├─ 4   bbox_w          边界框宽度 (float)
          ├─ 4   bbox_h          边界框高度 (float)
          ├─ 4   center_x        候选中心 X (float)
          └─ 4   center_y        候选中心 Y (float)
```

## Payload 尺寸计算

```
payload_size = 17                          # detected + center(8) + brightness(4) + contour_count(4)
             + contour_count × 8           # 每个轮廓点 8 字节
             + 4                           # candidates_count
             + candidates_count × 28       # 每个候选 28 字节
```

## 场景示例

| 场景 | contour | candidates | payload | 总包大小 |
|------|---------|------------|---------|----------|
| 无检测 | 0 | 0 | 21 B | 37 B |
| 简单检测 | 10 | 1 | 129 B | 145 B |
| 复杂检测 | 50 | 10 | 701 B | 717 B |

## 类别定义

| class_id | 名称 | 颜色 |
|----------|------|------|
| 0 | Purple | 紫色 |
| 1 | Red | 红色 |
| 2 | Blue | 蓝色 |

## C++ 发送示例 (laser_guidance)

```cpp
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <cstring>
#include <chrono>

struct LaserUdpSender {
    int sock;
    sockaddr_in addr;
    uint8_t seq;

    LaserUdpSender(const char* host = "127.0.0.1", uint16_t port = 5001) : seq(0) {
        sock = socket(AF_INET, SOCK_DGRAM, 0);
        addr.sin_family = AF_INET;
        addr.sin_port = htons(port);
        inet_pton(AF_INET, host, &addr.sin_addr);
    }

    void send(const rmcs_laser_guidance::TargetObservation& obs) {
        uint8_t buf[4096];
        size_t offset = 0;

        // Header
        buf[offset++] = 0x47;  // magic low
        buf[offset++] = 0x4C;  // magic high
        buf[offset++] = 1;     // version
        buf[offset++] = seq++;

        auto now = std::chrono::steady_clock::now();
        auto us = std::chrono::duration_cast<std::chrono::microseconds>(
            now.time_since_epoch()).count();
        std::memcpy(buf + offset, &us, 8); offset += 8;

        // Reserve payload_len (fill later)
        size_t len_offset = offset;
        offset += 4;

        // Payload
        size_t payload_start = offset;
        buf[offset++] = obs.detected ? 1 : 0;

        float center_x = obs.center.x;
        float center_y = obs.center.y;
        std::memcpy(buf + offset, &center_x, 4); offset += 4;
        std::memcpy(buf + offset, &center_y, 4); offset += 4;
        std::memcpy(buf + offset, &obs.brightness, 4); offset += 4;

        uint32_t contour_count = obs.contour.size();
        std::memcpy(buf + offset, &contour_count, 4); offset += 4;
        for (const auto& p : obs.contour) {
            float px = p.x, py = p.y;
            std::memcpy(buf + offset, &px, 4); offset += 4;
            std::memcpy(buf + offset, &py, 4); offset += 4;
        }

        uint32_t cand_count = obs.candidates.size();
        std::memcpy(buf + offset, &cand_count, 4); offset += 4;
        for (const auto& c : obs.candidates) {
            std::memcpy(buf + offset, &c.score, 4); offset += 4;
            std::memcpy(buf + offset, &c.class_id, 4); offset += 4;
            std::memcpy(buf + offset, &c.bbox.x, 4); offset += 4;
            std::memcpy(buf + offset, &c.bbox.y, 4); offset += 4;
            std::memcpy(buf + offset, &c.bbox.width, 4); offset += 4;
            std::memcpy(buf + offset, &c.bbox.height, 4); offset += 4;
            std::memcpy(buf + offset, &c.center.x, 4); offset += 4;
            std::memcpy(buf + offset, &c.center.y, 4); offset += 4;
        }

        // Fill payload_len
        uint32_t payload_len = offset - payload_start;
        std::memcpy(buf + len_offset, &payload_len, 4);

        sendto(sock, buf, offset, 0, (sockaddr*)&addr, sizeof(addr));
    }
};
```

## Rust 接收示例 (radar-egui)

```rust
use tokio::net::UdpSocket;

#[derive(Debug, Clone)]
pub struct LaserObservation {
    pub detected: bool,
    pub center: [f32; 2],
    pub brightness: f32,
    pub contour: Vec<[f32; 2]>,
    pub candidates: Vec<ModelCandidate>,
}

#[derive(Debug, Clone)]
pub struct ModelCandidate {
    pub score: f32,
    pub class_id: i32,
    pub bbox: [f32; 4],  // x, y, w, h
    pub center: [f32; 2],
}

pub fn parse_laser_packet(data: &[u8]) -> Option<LaserObservation> {
    if data.len() < 20 {
        return None;
    }

    // Header
    let magic = u16::from_le_bytes([data[0], data[1]]);
    if magic != 0x4C47 {
        return None;
    }
    let version = data[2];
    if version != 1 {
        return None;
    }

    // Payload
    let mut offset = 16;
    let detected = data[offset] != 0; offset += 1;

    let center_x = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
    let center_y = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
    let brightness = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;

    let contour_count = u32::from_le_bytes(data[offset..offset+4].try_into().ok()?) as usize; offset += 4;
    let mut contour = Vec::with_capacity(contour_count);
    for _ in 0..contour_count {
        let x = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let y = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        contour.push([x, y]);
    }

    let cand_count = u32::from_le_bytes(data[offset..offset+4].try_into().ok()?) as usize; offset += 4;
    let mut candidates = Vec::with_capacity(cand_count);
    for _ in 0..cand_count {
        let score = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let class_id = i32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let bbox_x = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let bbox_y = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let bbox_w = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let bbox_h = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let cx = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
        let cy = f32::from_le_bytes(data[offset..offset+4].try_into().ok()?); offset += 4;
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
    })
}
```

## 测试验证

发送测试包：

```bash
# Python 快速测试
python3 -c "
import socket, struct

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
buf = b'LG'           # magic
buf += b'\x01'         # version
buf += b'\x00'         # seq
buf += struct.pack('<Q', 1234567890)  # timestamp
buf += struct.pack('<I', 21)          # payload_len
buf += b'\x01'         # detected=true
buf += struct.pack('<ff', 960.0, 540.0)  # center
buf += struct.pack('<f', 0.85)           # brightness
buf += struct.pack('<I', 0)              # contour_count
buf += struct.pack('<I', 0)              # candidates_count
sock.sendto(buf, ('127.0.0.1', 5001))
print('Sent test packet')
"
```

## 配置

在 `radar-egui` 的配置中（未来支持）：

```yaml
laser:
  enabled: true
  udp_port: 5001
```

或通过命令行参数：

```bash
radar-egui --laser-port 5001
```
