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

        painter.rect_filled(rect, 8.0, theme::CRUST);

        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let x = rect.left() + t * rect.width();
            let y = rect.top() + t * rect.height();
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                (0.5, theme::SURFACE0),
            );
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                (0.5, theme::SURFACE0),
            );
        }

        painter.line_segment(
            [Pos2::new(center.x, rect.top()), Pos2::new(center.x, rect.bottom())],
            (0.5, theme::SURFACE1),
        );
        painter.line_segment(
            [Pos2::new(rect.left(), center.y), Pos2::new(rect.right(), center.y)],
            (0.5, theme::SURFACE1),
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

            painter.circle_filled(screen_pos, 7.0, color);
            painter.text(
                screen_pos + Vec2::new(14.0, -10.0),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(14.0),
                theme::SUBTEXT0,
            );
        }
    }
}
