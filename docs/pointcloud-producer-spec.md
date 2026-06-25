# Point Cloud Producer Spec

## 目标

修改 `model_to_map.cpp`，使输出的 PCD 带法向量字段，并写入共享内存 `/pointcloud_frame` 供 radar-egui 消费。

## 改动 1：PCD 输出加法向量

文件：`model_to_maptools/model_to_map/src/model_to_map.cpp`

当前函数 `append_subdivided_triangle_samples`（约 132-175 行）对 OBJ 三角形做细分采样，
输出 `pcl::PointXYZ`。改为输出 `pcl::PointXYZRGBNormal`，每个采样点带三角形面法向量。

具体改动：
- 输出点云类型：`pcl::PointXYZ` → `pcl::PointXYZRGBNormal`
- 三角形面法向量：`normal = (v1-v0) × (v2-v0)`，归一化后所有采样点共用同一个 normal
- PCD 字段顺序：`x y z rgb normal_x normal_y normal_z`

## 改动 2：写入共享内存

新增函数或独立脚本，加载 `map.pcd` 并写入 `/pointcloud_frame`。

### SHM 协议

```text
Header (64 字节):
  offset  size  field       说明
  0       4     magic        u32 = 0x50434446 ("PCDF")
  4       4     version      u32 = 1
  8       4     point_count  u32 = 实际点数
  12      4     max_points   u32 = 缓冲区容量（≥ point_count）
  16      4     frame_seq    atomic u32，写入时 +1 (release)
  20      4     write_idx    atomic u32，写入 1 - 上次值 (release)
  24      4     stride       u32 = 28
  28      36    _pad         0

Buffer (双缓冲，各 max_points × 28 字节):
  每点 28 字节: [x:f32, y:f32, z:f32, r:u8, g:u8, b:u8, a:u8, nx:f32, ny:f32, nz:f32]
```

### 写入流程（一次性写入，frame_seq=1）

```text
1. shm_open("/pointcloud_frame", O_CREAT | O_RDWR, 0666)
2. ftruncate(fd, 64 + 2 × max_points × 28)
3. mmap(... MAP_SHARED)
4. 写 Header: magic, version, point_count, max_points, stride=28
5. write_idx = 0 → 写 buf[0]
6. 内存屏障 + frame_seq = 1 (release)
7. munmap, close
```

### 坐标映射

PCD 点 `(x, y, z)` 先做坐标映射再写入 SHM：

```text
SHM.x = -PCD.x    // 长度（翻 X 轴，匹配 rerun 坐标系）
SHM.y =  PCD.z    // 宽度
SHM.z =  PCD.y    // 高度
```

法向量同理：`(-nx, nz, ny)`。

## 验收标准

- `pcl_viewer map.pcd` 能看到带颜色的点云
- radar-egui 切到 ◉ Radar 标签后，rerun viewer 中显示场地点云
- 不同朝向的面呈现不同颜色（地面暖白、墙面灰）
