use deku::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian", id_type = "u16")]
pub enum DeviceId {
    #[deku(id = "0")]
    Default,
    // Red team
    #[deku(id = "1")]
    RedHero,
    #[deku(id = "2")]
    RedEngineer,
    #[deku(id = "3")]
    RedInfantry3,
    #[deku(id = "4")]
    RedInfantry4,
    #[deku(id = "5")]
    RedInfantry5,
    #[deku(id = "6")]
    RedAerial,
    #[deku(id = "7")]
    RedSentry,
    #[deku(id = "8")]
    RedDart,
    #[deku(id = "9")]
    RedRadar,
    #[deku(id = "10")]
    RedOutpost,
    #[deku(id = "11")]
    RedBase,
    // Blue team
    #[deku(id = "101")]
    BlueHero,
    #[deku(id = "102")]
    BlueEngineer,
    #[deku(id = "103")]
    BlueInfantry3,
    #[deku(id = "104")]
    BlueInfantry4,
    #[deku(id = "105")]
    BlueInfantry5,
    #[deku(id = "106")]
    BlueAerial,
    #[deku(id = "107")]
    BlueSentry,
    #[deku(id = "108")]
    BlueDart,
    #[deku(id = "109")]
    BlueRadar,
    #[deku(id = "110")]
    BlueOutpost,
    #[deku(id = "111")]
    BlueBase,
    // Referee system
    #[deku(id = "0x8080")]
    RefereeServer,
}

impl From<DeviceId> for u16 {
    fn from(v: DeviceId) -> u16 {
        match v {
            DeviceId::Default => 0,
            DeviceId::RedHero => 1,
            DeviceId::RedEngineer => 2,
            DeviceId::RedInfantry3 => 3,
            DeviceId::RedInfantry4 => 4,
            DeviceId::RedInfantry5 => 5,
            DeviceId::RedAerial => 6,
            DeviceId::RedSentry => 7,
            DeviceId::RedDart => 8,
            DeviceId::RedRadar => 9,
            DeviceId::RedOutpost => 10,
            DeviceId::RedBase => 11,
            DeviceId::BlueHero => 101,
            DeviceId::BlueEngineer => 102,
            DeviceId::BlueInfantry3 => 103,
            DeviceId::BlueInfantry4 => 104,
            DeviceId::BlueInfantry5 => 105,
            DeviceId::BlueAerial => 106,
            DeviceId::BlueSentry => 107,
            DeviceId::BlueDart => 108,
            DeviceId::BlueRadar => 109,
            DeviceId::BlueOutpost => 110,
            DeviceId::BlueBase => 111,
            DeviceId::RefereeServer => 0x8080,
        }
    }
}

impl From<u16> for DeviceId {
    fn from(v: u16) -> Self {
        match v {
            0 => DeviceId::Default,
            1 => DeviceId::RedHero,
            2 => DeviceId::RedEngineer,
            3 => DeviceId::RedInfantry3,
            4 => DeviceId::RedInfantry4,
            5 => DeviceId::RedInfantry5,
            6 => DeviceId::RedAerial,
            7 => DeviceId::RedSentry,
            8 => DeviceId::RedDart,
            9 => DeviceId::RedRadar,
            10 => DeviceId::RedOutpost,
            11 => DeviceId::RedBase,
            101 => DeviceId::BlueHero,
            102 => DeviceId::BlueEngineer,
            103 => DeviceId::BlueInfantry3,
            104 => DeviceId::BlueInfantry4,
            105 => DeviceId::BlueInfantry5,
            106 => DeviceId::BlueAerial,
            107 => DeviceId::BlueSentry,
            108 => DeviceId::BlueDart,
            109 => DeviceId::BlueRadar,
            110 => DeviceId::BlueOutpost,
            111 => DeviceId::BlueBase,
            0x8080 => DeviceId::RefereeServer,
            _ => DeviceId::Default,
        }
    }
}
