use deku::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian", id_type = "u16")]
pub enum DeviceId {
    #[deku(id = "0")]
    Default,
    // 红方
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
    // 蓝方
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
    // 裁判系统
    #[deku(id = "0x8080")]
    RefereeServer,
}
