# radar-egui

Real-time RoboMaster competition HUD built in Rust with egui.

## Overview

radar-egui connects to the SDR data stream via TCP, parses RoboMaster game state packets, and displays real-time battlefield information including robot positions, blood levels, ammunition, economy, and gain status.

## Prerequisites

- Rust toolchain (1.75+)
- Linux (X11 or Wayland)
- SDR data source running on `127.0.0.1:2000`

## Build and Run

```bash
# Build
cargo build --release

# Run
cargo run --release

# Run with logging
RUST_LOG=info cargo run --release
```

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Radar HUD                              ● Connected         │
├───────────────────────┬─────────────────────────────────────┤
│                       │  Blood                              │
│                       │  ┌──────────────────────────────┐   │
│    Battlefield Map    │  │ Hero      ████████░░  160/200│   │
│                       │  │ Engineer  ██████████  200/200│   │
│    ·Hero              │  │ Infantry1 ████░░░░░░   80/200│   │
│    ·Engineer          │  │ Infantry2 ███████░░░  140/200│   │
│    ·Infantry1         │  │ Saven     ██████████  200/200│   │
│    ·Infantry2         │  │ Sentinel  ██████████  400/400│   │
│    ·Drone             │  └──────────────────────────────┘   │
│    ·Sentinel          │                                     │
│                       │  Ammunition                         │
│                       │  ┌──────────────────────────────┐   │
│                       │  │ Hero       85                 │   │
│                       │  │ Infantry1  100                │   │
│                       │  │ Infantry2  92                 │   │
│                       │  │ Drone      78                 │   │
│                       │  │ Sentinel   65                 │   │
│                       │  └──────────────────────────────┘   │
│                       │                                     │
│                       │  Economy                            │
│                       │  ┌──────────────────────────────┐   │
│                       │  │ Remain: 1200 / Total: 1500   │   │
│                       │  │ ████████░░ 80%               │   │
│                       │  └──────────────────────────────┘   │
│                       │                                     │
│                       │  Gains                              │
│                       │  ┌──────────────────────────────┐   │
│                       │  │ Robot  Regen Cool Def NegD Atk│   │
│                       │  │ Hero    2    10   5   3   8  │   │
│                       │  │ ...                          │   │
│                       │  └──────────────────────────────┘   │
└───────────────────────┴─────────────────────────────────────┘
```

## Data Source

radar-egui consumes data from `alliance_radar_sdr` via TCP:

| Port | Direction | Data |
|------|-----------|------|
| `127.0.0.1:2000` | Receive | RoboMaster_Signal_Info (102 bytes) |

### Packet Structure

| cmd_id | Name | Fields | Bytes |
|--------|------|--------|-------|
| 0x0A01 | Positions | 6 robots × [i16, i16] | 26 |
| 0x0A02 | Blood | 6 robots × u16 | 14 |
| 0x0A03 | Ammunition | 5 robots × u16 | 12 |
| 0x0A04 | Economy | remain(u16) + total(u16) + status(6B) | 12 |
| 0x0A05 | Gains | 5 robots × [1+2+1+1+2] + posture(1) | 38 |

Byte order: big-endian for most fields, little-endian for 2-byte gain sub-fields.

## Module Structure

```
src/
├── main.rs           # Entry point, egui window setup
├── protocol.rs       # RoboMasterSignalInfo struct + binary parser
├── tcp_client.rs     # Async TCP client with auto-reconnect
├── app.rs            # egui application with 4-panel layout
└── widgets/
    ├── mod.rs        # Re-exports
    ├── minimap.rs    # 2D battlefield minimap (Painter)
    └── panels.rs     # Blood/ammo/economy/gain panels
```

## Dependencies

- `eframe` / `egui` — immediate mode GUI
- `tokio` — async TCP client
- `log` / `env_logger` — logging

## License

MIT
