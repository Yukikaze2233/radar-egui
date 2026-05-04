/// RoboMaster signal data parsed from GNU Radio stream.
///
/// Packet layout (~102 bytes total):
///   cmd_id_1 (0x0A01): 26 bytes — 6 robot positions × [i16, i16]
///   cmd_id_2 (0x0A02): 14 bytes — 6 robot blood × u16
///   cmd_id_3 (0x0A03): 12 bytes — 5 robot ammunition × u16
///   cmd_id_4 (0x0A04): 12 bytes — economy remain(u16) + total(u16) + occupation_status(6 bytes)
///   cmd_id_5 (0x0A05): 38 bytes — 5 robot gains × [1+2+1+1+2 bytes] + sentinel_posture(1 byte)
///
/// Byte order: big-endian for most fields, little-endian for 2-byte gain sub-fields.
#[derive(Debug, Clone, Default)]
pub struct RoboMasterSignalInfo {
    // Positions (0x0A01) — each [x, y] as i16
    pub hero_position: [i16; 2],
    pub engineer_position: [i16; 2],
    pub infantry_position_1: [i16; 2],
    pub infantry_position_2: [i16; 2],
    pub drone_position: [i16; 2],
    pub sentinel_position: [i16; 2],

    // Blood (0x0A02) — each u16
    pub hero_blood: u16,
    pub engineer_blood: u16,
    pub infantry_blood_1: u16,
    pub infantry_blood_2: u16,
    pub saven_blood: u16,
    pub sentinel_blood: u16,

    // Ammunition (0x0A03) — each u16
    pub hero_ammunition: u16,
    pub infantry_ammunition_1: u16,
    pub infantry_ammunition_2: u16,
    pub drone_ammunition: u16,
    pub sentinel_ammunition: u16,

    // Economy (0x0A04)
    pub economic_remain: u16,
    pub economic_total: u16,
    pub occupation_status: [u8; 6],

    // Gains (0x0A05) — each robot: [health_regen(1), cooling(2 LE), defense(1), neg_defense(1), attack(2 LE)]
    pub hero_gain: [u8; 7],
    pub engineer_gain: [u8; 7],
    pub infantry_gain_1: [u8; 7],
    pub infantry_gain_2: [u8; 7],
    pub sentinel_gain: [u8; 7],
    pub sentinel_posture: u8,
}

/// Parse a raw byte buffer into RoboMasterSignalInfo.
///
/// Uses sliding-window scan for cmd_id at each byte position,
/// matching the Python gnuradio_frame_parser behavior.
/// Returns None if buffer is too short (< 90 bytes) or no valid cmd_id found.
pub fn parse_signal(data: &[u8]) -> Option<RoboMasterSignalInfo> {
    if data.len() < 90 {
        return None;
    }

    let mut info = RoboMasterSignalInfo::default();
    let mut found_any = false;

    for i in 0..data.len() {
        if i + 2 > data.len() {
            break;
        }
        let cmd_id = u16::from_be_bytes([data[i], data[i + 1]]);

        match cmd_id {
            // 0x0A01: Positions (26 bytes: 2 cmd + 24 data)
            0x0A01 if i + 26 <= data.len() => {
                info.hero_position[0] = i16::from_be_bytes([data[i + 2], data[i + 3]]);
                info.hero_position[1] = i16::from_be_bytes([data[i + 4], data[i + 5]]);
                info.engineer_position[0] = i16::from_be_bytes([data[i + 6], data[i + 7]]);
                info.engineer_position[1] = i16::from_be_bytes([data[i + 8], data[i + 9]]);
                info.infantry_position_1[0] = i16::from_be_bytes([data[i + 10], data[i + 11]]);
                info.infantry_position_1[1] = i16::from_be_bytes([data[i + 12], data[i + 13]]);
                info.infantry_position_2[0] = i16::from_be_bytes([data[i + 14], data[i + 15]]);
                info.infantry_position_2[1] = i16::from_be_bytes([data[i + 16], data[i + 17]]);
                info.drone_position[0] = i16::from_be_bytes([data[i + 18], data[i + 19]]);
                info.drone_position[1] = i16::from_be_bytes([data[i + 20], data[i + 21]]);
                info.sentinel_position[0] = i16::from_be_bytes([data[i + 22], data[i + 23]]);
                info.sentinel_position[1] = i16::from_be_bytes([data[i + 24], data[i + 25]]);
                found_any = true;
            }
            // 0x0A02: Blood (14 bytes: 2 cmd + 12 data)
            0x0A02 if i + 14 <= data.len() => {
                info.hero_blood = u16::from_be_bytes([data[i + 2], data[i + 3]]);
                info.engineer_blood = u16::from_be_bytes([data[i + 4], data[i + 5]]);
                info.infantry_blood_1 = u16::from_be_bytes([data[i + 6], data[i + 7]]);
                info.infantry_blood_2 = u16::from_be_bytes([data[i + 8], data[i + 9]]);
                info.saven_blood = u16::from_be_bytes([data[i + 10], data[i + 11]]);
                info.sentinel_blood = u16::from_be_bytes([data[i + 12], data[i + 13]]);
                found_any = true;
            }
            // 0x0A03: Ammunition (12 bytes: 2 cmd + 10 data)
            0x0A03 if i + 12 <= data.len() => {
                info.hero_ammunition = u16::from_be_bytes([data[i + 2], data[i + 3]]);
                info.infantry_ammunition_1 = u16::from_be_bytes([data[i + 4], data[i + 5]]);
                info.infantry_ammunition_2 = u16::from_be_bytes([data[i + 6], data[i + 7]]);
                info.drone_ammunition = u16::from_be_bytes([data[i + 8], data[i + 9]]);
                info.sentinel_ammunition = u16::from_be_bytes([data[i + 10], data[i + 11]]);
                found_any = true;
            }
            // 0x0A04: Economy (12 bytes: 2 cmd + 10 data)
            0x0A04 if i + 12 <= data.len() => {
                info.economic_remain = u16::from_be_bytes([data[i + 2], data[i + 3]]);
                info.economic_total = u16::from_be_bytes([data[i + 4], data[i + 5]]);
                info.occupation_status.copy_from_slice(&data[i + 6..i + 12]);
                found_any = true;
            }
            // 0x0A05: Gains (38 bytes: 2 cmd + 36 data)
            // Each robot gain: [1 BE, 2 LE, 1 BE, 1 BE, 2 LE] = 7 bytes
            0x0A05 if i + 38 <= data.len() => {
                info.hero_gain.copy_from_slice(&data[i + 2..i + 9]);
                info.engineer_gain.copy_from_slice(&data[i + 9..i + 16]);
                info.infantry_gain_1.copy_from_slice(&data[i + 16..i + 23]);
                info.infantry_gain_2.copy_from_slice(&data[i + 23..i + 30]);
                info.sentinel_gain.copy_from_slice(&data[i + 30..i + 37]);
                info.sentinel_posture = data[i + 37];
                found_any = true;
            }
            _ => {}
        }
    }

    if found_any {
        Some(info)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_too_short() {
        assert!(parse_signal(&[0u8; 50]).is_none());
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_signal(&[]).is_none());
    }

    #[test]
    fn test_parse_valid_cmd_id_1() {
        let mut buf = vec![0u8; 102];
        // cmd_id_1 = 0x0A01 at offset 0
        buf[0] = 0x0A;
        buf[1] = 0x01;
        // hero_position = [100, 200]
        buf[2] = 0x00;
        buf[3] = 0x64; // 100
        buf[4] = 0x00;
        buf[5] = 0xC8; // 200

        let info = parse_signal(&buf).unwrap();
        assert_eq!(info.hero_position, [100, 200]);
    }
}
