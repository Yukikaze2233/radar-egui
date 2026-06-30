# radar-egui TODO

## 串口通信层

- [x] 串口协议解析器 (serial_parser) — 滑动窗口 cmd_id 扫描
- [x] 设备 ID 枚举 (DeviceId) — From<u16> / From<DeviceId> 双向转换
- [x] 机器人交互包收发 (0x0301) — RobotInteractionData + subcontext 解析
- [x] 常规链路协议数据解析 (0x0001–0x020E) + SDR 解析占位
- [x] SerialProtocolData + serial_produced[15] / zmq_produced[15] 双向标志数组
- [x] transmitter 重写 — 读取 zmq_produced[idx] → serial_package → 串口发送 → 归 0
- [x] serial_parser 解析后置位 serial_produced[idx] = 1
- [ ] 串口收发线程正式连线 app/mod.rs + runtime/（当前 demo 占位）
- [ ] 串口发送分批次 — 不同 cmd_id 按各自频率独立发送

## ZMQ 通信层

- [x] zmq.rs — PUB/SUB 初始化、zmq_send、zmq_recv 封装
- [x] serde + serde_json 依赖引入 (Cargo.toml)
- [x] zmq/data_format.rs — ZmqMessageId + Transmit*/Receive* 结构体 + ZmqData 聚合
- [x] zmq_package.rs — JSON 组包 (SerialProtocolData → String)
- [x] zmq_parser.rs — JSON 解包 (JSON bytes → 类型分发)
- [x] ZMQ_PUB_* / ZMQ_SUB_* 消息 ID 空间独立定义
- [x] ZmqSdrRuntime — ZMQ SUB SDR 线程 (std::thread, 无 tokio)
- [x] ZmqLaserRuntime — ZMQ SUB Laser 线程 (std::thread, 无 tokio)
- [ ] ZMQ PUB 线程 — 读取 serial_produced[idx] → zmq_package → zmq_send

## SDR 无线链路（TCP → ZMQ 已删除）

- [x] `src/sdr/` 目录已删除
- [x] 所有引用迁移至 `zmq/data_format::ReceiveSdr`（字段拆为子结构体 position/blood/ammo/state/gain/key）
- [x] rerun_visualizer / minimap / panels 字段路径已更新
- [x] `RadarFeed*` → `ZmqReader`/`ZmqWriter` 统一

## 工程

- [x] ZMQ 依赖集成 (zmq2 crate)
- [x] 串口模块中文注释英文化
- [x] SerialMetadata / ZmqMetadata 监控层已删除（ZMQ 自动重连）
- [ ] CI/CD 构建脚本
- [ ] 测试用例补充
