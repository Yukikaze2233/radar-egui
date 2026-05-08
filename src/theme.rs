#![allow(dead_code)]

use egui::Color32;

// Catppuccin Mocha palette — base layers
pub const CRUST: Color32 = Color32::from_rgb(0x11, 0x11, 0x1b);
pub const MANTLE: Color32 = Color32::from_rgb(0x18, 0x18, 0x25);
pub const BASE: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2e);
pub const SURFACE0: Color32 = Color32::from_rgb(0x31, 0x32, 0x44);
pub const SURFACE1: Color32 = Color32::from_rgb(0x45, 0x47, 0x5a);
pub const SURFACE2: Color32 = Color32::from_rgb(0x58, 0x5b, 0x70);
pub const OVERLAY0: Color32 = Color32::from_rgb(0x6c, 0x70, 0x86);
pub const SUBTEXT0: Color32 = Color32::from_rgb(0xa6, 0xad, 0xc8);
pub const TEXT: Color32 = Color32::from_rgb(0xcd, 0xd6, 0xf4);

// ── M3-style surface hierarchy (between BASE and SURFACE0) ──
pub const SURFACE_LOW: Color32 = Color32::from_rgb(0x24, 0x24, 0x38);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2c, 0x2c, 0x42);

// ── Card design tokens ──
pub const CARD_BG: Color32 = SURFACE_LOW;
pub const CARD_BORDER: Color32 = Color32::from_rgb(0x3a, 0x3a, 0x50);
pub const CARD_RADIUS: u8 = 12;

// ── Subtle elements ──
pub const GRID_LINE: Color32 = Color32::from_rgb(0x28, 0x28, 0x3c);
pub const GRID_BG: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x2a);

// ── Accent colors ──
pub const BLUE: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa);
pub const GREEN: Color32 = Color32::from_rgb(0xa6, 0xe3, 0xa1);
pub const RED: Color32 = Color32::from_rgb(0xf3, 0x8b, 0xa8);
pub const YELLOW: Color32 = Color32::from_rgb(0xf9, 0xe2, 0xaf);
pub const PEACH: Color32 = Color32::from_rgb(0xfa, 0xb3, 0x87);
pub const TEAL: Color32 = Color32::from_rgb(0x94, 0xe2, 0xd5);
pub const MAUVE: Color32 = Color32::from_rgb(0xcb, 0xa6, 0xf7);
pub const SAPPHIRE: Color32 = Color32::from_rgb(0x74, 0xc7, 0xec);
pub const SKY: Color32 = Color32::from_rgb(0x89, 0xdc, 0xeb);
pub const LAVENDER: Color32 = Color32::from_rgb(0xb4, 0xbe, 0xfe);

pub const HERO_COLOR: Color32 = BLUE;
pub const ENGINEER_COLOR: Color32 = TEAL;
pub const INFANTRY1_COLOR: Color32 = GREEN;
pub const INFANTRY2_COLOR: Color32 = YELLOW;
pub const DRONE_COLOR: Color32 = PEACH;
pub const SENTINEL_COLOR: Color32 = MAUVE;

pub const BLOOD_HIGH: Color32 = GREEN;
pub const BLOOD_MID: Color32 = YELLOW;
pub const BLOOD_LOW: Color32 = RED;

pub const CONNECTED: Color32 = GREEN;
pub const DISCONNECTED: Color32 = RED;

/// Lighten a color for a progress-bar highlight stripe.
pub fn lighten(c: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_add(amount),
        c.g().saturating_add(amount),
        c.b().saturating_add(amount),
    )
}
