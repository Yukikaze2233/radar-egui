#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

use egui::Color32;

static DARK_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_dark_mode(enabled: bool) {
    DARK_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_dark_mode() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

fn pick(light: Color32, dark: Color32) -> Color32 {
    if is_dark_mode() {
        dark
    } else {
        light
    }
}

pub fn app_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xf5, 0xf7, 0xfb),
        Color32::from_rgb(0x11, 0x11, 0x1b),
    )
}

pub fn shell_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xee, 0xf2, 0xf7),
        Color32::from_rgb(0x18, 0x18, 0x25),
    )
}

pub fn rail_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xf5, 0xf7, 0xfb),
        Color32::from_rgb(0x11, 0x11, 0x1b),
    )
}

pub fn panel_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xee, 0xf2, 0xf7),
        Color32::from_rgb(0x1e, 0x1e, 0x2e),
    )
}

pub fn card_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xff, 0xff, 0xff),
        Color32::from_rgb(0x31, 0x32, 0x44),
    )
}

pub fn card_bg_muted() -> Color32 {
    pick(
        Color32::from_rgb(0xf8, 0xfa, 0xfd),
        Color32::from_rgb(0x45, 0x47, 0x5a),
    )
}

pub fn map_frame() -> Color32 {
    pick(
        Color32::from_rgb(0xee, 0xf2, 0xf7),
        Color32::from_rgb(0x18, 0x18, 0x25),
    )
}

pub fn map_bg() -> Color32 {
    pick(
        Color32::from_rgb(0xfc, 0xfd, 0xff),
        Color32::from_rgb(0xf5, 0xf7, 0xfb),
    )
}

pub fn border() -> Color32 {
    pick(
        Color32::from_rgb(0xd7, 0xdf, 0xeb),
        Color32::from_rgb(0x45, 0x47, 0x5a),
    )
}

pub fn border_strong() -> Color32 {
    pick(
        Color32::from_rgb(0xb8, 0xc5, 0xd9),
        Color32::from_rgb(0x58, 0x5b, 0x70),
    )
}

pub fn grid() -> Color32 {
    pick(
        Color32::from_rgb(0xe4, 0xea, 0xf3),
        Color32::from_rgb(0xd7, 0xde, 0xe9),
    )
}

pub fn grid_strong() -> Color32 {
    pick(
        Color32::from_rgb(0xc3, 0xcf, 0xdf),
        Color32::from_rgb(0xa6, 0xb5, 0xc9),
    )
}

pub fn shadow() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(0x0f, 0x17, 0x2a, 0x16),
        Color32::from_rgba_premultiplied(0x00, 0x00, 0x00, 0x36),
    )
}

pub fn text() -> Color32 {
    pick(
        Color32::from_rgb(0x1f, 0x29, 0x37),
        Color32::from_rgb(0xcd, 0xd6, 0xf4),
    )
}

pub fn text_muted() -> Color32 {
    pick(
        Color32::from_rgb(0x66, 0x70, 0x83),
        Color32::from_rgb(0xa6, 0xad, 0xc8),
    )
}

pub fn text_faint() -> Color32 {
    pick(
        Color32::from_rgb(0x94, 0xa3, 0xb8),
        Color32::from_rgb(0x6c, 0x70, 0x86),
    )
}

pub fn text_on_dark() -> Color32 {
    Color32::from_rgb(0xcd, 0xd6, 0xf4)
}

pub fn text_on_dark_muted() -> Color32 {
    Color32::from_rgb(0xa6, 0xad, 0xc8)
}

pub const BLUE: Color32 = Color32::from_rgb(0x2f, 0x6b, 0xff);
pub const BLUE_SOFT: Color32 = Color32::from_rgb(0xdc, 0xe7, 0xff);
pub const GREEN: Color32 = Color32::from_rgb(0x22, 0xc5, 0x5e);
pub const RED: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);
pub const YELLOW: Color32 = Color32::from_rgb(0xf5, 0x9e, 0x0b);
pub const PEACH: Color32 = Color32::from_rgb(0xf9, 0x73, 0x16);
pub const TEAL: Color32 = Color32::from_rgb(0x14, 0xb8, 0xa6);
pub const MAUVE: Color32 = Color32::from_rgb(0xa8, 0x55, 0xf7);
pub const SAPPHIRE: Color32 = Color32::from_rgb(0x38, 0xb6, 0xff);
pub const SKY: Color32 = Color32::from_rgb(0x0e, 0xae, 0xe9);
pub const LAVENDER: Color32 = Color32::from_rgb(0x8b, 0x9d, 0xff);

pub const HERO_COLOR: Color32 = Color32::from_rgb(0x3b, 0x82, 0xf6);
pub const ENGINEER_COLOR: Color32 = Color32::from_rgb(0x14, 0xb8, 0xa6);
pub const INFANTRY1_COLOR: Color32 = Color32::from_rgb(0x22, 0xc5, 0x5e);
pub const INFANTRY2_COLOR: Color32 = Color32::from_rgb(0xea, 0xb3, 0x08);
pub const DRONE_COLOR: Color32 = Color32::from_rgb(0xf9, 0x73, 0x16);
pub const SENTINEL_COLOR: Color32 = Color32::from_rgb(0xa8, 0x55, 0xf7);

pub const CONNECTED: Color32 = GREEN;
pub const DISCONNECTED: Color32 = RED;

pub fn crust() -> Color32 {
    app_bg()
}

pub fn mantle() -> Color32 {
    shell_bg()
}

pub fn base() -> Color32 {
    panel_bg()
}

pub fn surface0() -> Color32 {
    card_bg_muted()
}

pub fn surface1() -> Color32 {
    border()
}

pub fn surface2() -> Color32 {
    border_strong()
}

pub fn overlay0() -> Color32 {
    text_faint()
}

pub fn subtext0() -> Color32 {
    text_muted()
}
