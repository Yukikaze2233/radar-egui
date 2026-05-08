use egui::{Pos2, Vec2};
use std::sync::{Arc, Mutex};

use crate::protocol::RoboMasterSignalInfo;
use crate::theme;

pub struct MinimapWidget {
    shared: Arc<Mutex<RoboMasterSignalInfo>>,
}

impl MinimapWidget {
    pub fn new(shared: Arc<Mutex<RoboMasterSignalInfo>>) -> Self {
        Self { shared }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = Vec2::new(available.x, available.x);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let scale = rect.width() * 0.45 / 3000.0;
        let r = 8;

        painter.rect_filled(rect, r, theme::GRID_BG);

        // Soft grid
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let x = rect.left() + t * rect.width();
            let y = rect.top() + t * rect.height();
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                (0.5, theme::GRID_LINE),
            );
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                (0.5, theme::GRID_LINE),
            );
        }

        // Crosshair
        let ch = theme::SURFACE1;
        painter.line_segment(
            [Pos2::new(center.x, rect.top()), Pos2::new(center.x, rect.bottom())],
            (0.5, ch),
        );
        painter.line_segment(
            [Pos2::new(rect.left(), center.y), Pos2::new(rect.right(), center.y)],
            (0.5, ch),
        );

        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };

        let robots: &[(&str, [i16; 2], egui::Color32)] = &[
            ("英雄", info.hero_position, theme::HERO_COLOR),
            ("工程", info.engineer_position, theme::ENGINEER_COLOR),
            ("步兵1", info.infantry_position_1, theme::INFANTRY1_COLOR),
            ("步兵2", info.infantry_position_2, theme::INFANTRY2_COLOR),
            ("无人机", info.drone_position, theme::DRONE_COLOR),
            ("哨兵", info.sentinel_position, theme::SENTINEL_COLOR),
        ];

        for &(name, pos, color) in robots {
            let screen_pos = Pos2::new(
                center.x + pos[0] as f32 * scale,
                center.y - pos[1] as f32 * scale,
            );

            // Glow ring
            let glow = egui::Color32::from_rgba_premultiplied(
                color.r(), color.g(), color.b(), 60,
            );
            painter.circle_filled(screen_pos, 13.0, glow);

            // Solid dot
            painter.circle_filled(screen_pos, 6.0, color);

            // Pill label
            let label = name;
            let font_id = egui::FontId::proportional(12.0);
            let text_pos = screen_pos + Vec2::new(16.0, -7.0);

            // Measure text size for pill background
            let galley = ui.painter().layout_no_wrap(
                label.to_owned(),
                font_id.clone(),
                theme::TEXT,
            );
            let pad = Vec2::new(8.0, 3.0);
            let pill_rect = egui::Rect::from_center_size(
                text_pos + Vec2::new(galley.size().x / 2.0, 0.0),
                galley.size() + pad * 2.0,
            );
            painter.rect_filled(pill_rect, 6, theme::SURFACE_LOW);
            painter.rect_stroke(
                pill_rect,
                6,
                (0.5, theme::CARD_BORDER),
                egui::StrokeKind::Inside,
            );

            painter.text(text_pos, egui::Align2::LEFT_CENTER, label, font_id, color);
        }
    }
}
