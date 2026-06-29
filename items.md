# radar-egui TODO

## 串口通信层

- [x] 串口协议解析器 (serial_parser) — 滑动窗口 cmd_id 扫描
- [x] 设备 ID 枚举 (DeviceId) — 红方/蓝方机器人 + 裁判服务器
- [x] 机器人交互包收发 (0x0301) — RobotInteractionHeader + 变长 user_data
- [x] 雷达自主决策指令发送 (0x0121 → 0x8080)
- [x] 收发线程所有权修复 (try_clone)
- [x] 常规链路协议数据解析 (0x0001–0x020E)
- [x] SerialProtocolData 增加 serial_produced[15] / zmq_produced[15] 双向标志数组
- [x] serial_parser 解析后置位 serial_produced[idx] = 1
- [x] transmitter 重写 — 读取 zmq_produced[idx] → serial_package → 串口发送 → 归 0
- [ ] 串口收发线程正式连线 app/mod.rs + runtime/（当前 demo 占位）

## ZMQ 通信层

- [x] zmq.rs — PUB/SUB 初始化、zmq_send、zmq_recv 封装
- [x] serde + serde_json 依赖引入 (Cargo.toml)
- [ ] zmq/data_format.rs — JSON 消息格式 (cmd + payload)
- [ ] zmq/zmq_package.rs — JSON 组包 (struct → JSON string)
- [ ] zmq/zmq_parser.rs — JSON 解包 (JSON string → struct)
- [ ] ZMQ PUB 线程 — 读取 serial_produced[idx] → JSON → zmq_send
- [ ] ZMQ SUB 线程 — zmq_recv → JSON 解析 → zmq_produced[idx] = 1
- [ ] ZMQ 线程连线 runtime/mod.rs

## SDR 无线链路（TCP 将逐步被 ZMQ 取代）

- [x] SDR TCP 客户端 (127.0.0.1:2000) — parse_signal 滑动窗口解析
- [x] RoboMasterSignalInfo → RadarFeedWriter → egui 状态面板
- [ ] ZMQ 取代 SDR TCP 数据通路

## 可视化 (egui)

<!--
- [x] 机器人位置小地图渲染
- [x] 血量面板 (水平进度条, 颜色编码)
- [x] 弹药面板
- [x] 经济面板 (剩余/总计 + 进度条)
- [x] 增益面板 (每机器人 5 项增益明细)
- [x] SDR 数据面板
-->

## 交互功能

<!--
- [ ] 小地图交互指令
- [ ] 密钥更新/验证发送
-->

## 工程

- [x] ZMQ 依赖集成 (zmq2 crate)
- [ ] ci/cd 构建脚本
- [ ] 测试用例补充
