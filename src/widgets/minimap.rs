use egui::{Color32, Pos2, Stroke, Vec2};
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

    pub fn show_with_state(
        &self,
        ui: &mut egui::Ui,
        background: Option<&egui::TextureHandle>,
        pan: &mut Vec2,
        zoom: &mut f32,
    ) {
        let available = ui.available_size();
        let size = Vec2::new(available.x, available.y.max(available.x * 0.72));
        let (response, painter) = ui.allocate_painter(size, egui::Sense::drag());
        let rect = response.rect;

        painter.rect_filled(rect, 0.0, theme::app_bg());

        let board_rect = rect;
        if let Some(background) = background {
            if response.hovered() {
                let scroll_delta = ui.ctx().input(|input| input.raw_scroll_delta.y);
                if scroll_delta.abs() > f32::EPSILON {
                    let zoom_factor = (1.0 + scroll_delta * 0.0015).clamp(0.9, 1.1);
                    *zoom = (*zoom * zoom_factor).clamp(0.45, 3.0);
                }
            }

            if response.dragged() {
                *pan += ui.ctx().input(|input| input.pointer.delta());
            }

            let texture_size = background.size_vec2();
            let fit_scale =
                (board_rect.width() / texture_size.x).min(board_rect.height() / texture_size.y);
            let image_size = texture_size * fit_scale * *zoom;
            let max_x = ((image_size.x - board_rect.width()) * 0.5).max(0.0);
            let max_y = ((image_size.y - board_rect.height()) * 0.5).max(0.0);
            pan.x = pan.x.clamp(-max_x, max_x);
            pan.y = pan.y.clamp(-max_y, max_y);

            let image_rect = egui::Rect::from_center_size(board_rect.center() + *pan, image_size);
            painter.image(
                background.id(),
                image_rect,
                egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.rect_filled(image_rect, 0.0, Color32::from_white_alpha(10));
            self.draw_grid(&painter, image_rect.intersect(board_rect));
        } else {
            if response.dragged() {
                *pan = Vec2::ZERO;
            }
            painter.rect_filled(board_rect, 0.0, theme::map_bg());
            self.draw_grid(&painter, board_rect);
        }

        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };

        let world_rect = if let Some(background) = background {
            let texture_size = background.size_vec2();
            let fit_scale =
                (board_rect.width() / texture_size.x).min(board_rect.height() / texture_size.y);
            let image_size = texture_size * fit_scale * *zoom;
            egui::Rect::from_center_size(board_rect.center() + *pan, image_size)
        } else {
            board_rect
        };
        let center = world_rect.center();
        let scale = world_rect.width().min(world_rect.height()) * 0.43 / 3000.0;

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

            painter.circle_filled(screen_pos, 12.0, Color32::from_white_alpha(36));
            painter.circle_filled(screen_pos, 6.5, color);
            painter.circle_stroke(
                screen_pos,
                11.0,
                Stroke::new(1.0, color.gamma_multiply(0.25)),
            );
            painter.text(
                screen_pos + Vec2::new(14.0, -12.0),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(13.0),
                theme::text_muted(),
            );
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let x = rect.left() + t * rect.width();
            let y = rect.top() + t * rect.height();
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(
                    if i == 5 { 1.0 } else { 0.6 },
                    if i == 5 {
                        theme::grid_strong()
                    } else {
                        theme::grid()
                    },
                ),
            );
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(
                    if i == 5 { 1.0 } else { 0.6 },
                    if i == 5 {
                        theme::grid_strong()
                    } else {
                        theme::grid()
                    },
                ),
            );
        }

        let scan_y = rect.top() + rect.height() * 0.18;
        painter.line_segment(
            [
                Pos2::new(rect.left(), scan_y),
                Pos2::new(rect.right(), scan_y),
            ],
            Stroke::new(1.2, theme::RED.gamma_multiply(0.8)),
        );
    }
}
