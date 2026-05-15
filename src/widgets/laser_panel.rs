use egui::{Color32, Pos2, RichText, Vec2};
use std::sync::{Arc, Mutex};

use crate::laser_protocol::LaserObservation;
use crate::theme;
use crate::video_stream::VideoFrame;

pub struct LaserPanel {
    shared: Arc<Mutex<LaserObservation>>,
    video: Arc<Mutex<Option<VideoFrame>>>,
}

impl LaserPanel {
    pub fn new(
        shared: Arc<Mutex<LaserObservation>>,
        video: Arc<Mutex<Option<VideoFrame>>>,
    ) -> Self {
        Self { shared, video }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        self.show_video_stage(ui);
        ui.add_space(16.0);
        self.show_analysis_sidebar(ui);
    }

    pub fn show_video_stage(&self, ui: &mut egui::Ui) {
        let obs = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };

        self.draw_video_with_overlay(ui, &obs);
    }

    pub fn show_analysis_sidebar(&self, ui: &mut egui::Ui) {
        let obs = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };

        self.card(ui, "目标检测", |ui| {
            if obs.detected {
                ui.label(RichText::new("已检测到目标").color(theme::GREEN).size(16.0));
                ui.add_space(8.0);
                egui::Grid::new("target_grid_sidebar")
                    .num_columns(2)
                    .spacing([24.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("中心 X").color(theme::subtext0()).size(14.0));
                        ui.label(
                            RichText::new(format!("{:.1}", obs.center[0]))
                                .color(theme::text())
                                .size(16.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new("中心 Y").color(theme::subtext0()).size(14.0));
                        ui.label(
                            RichText::new(format!("{:.1}", obs.center[1]))
                                .color(theme::text())
                                .size(16.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new("亮度").color(theme::subtext0()).size(14.0));
                        ui.label(
                            RichText::new(format!("{:.2}", obs.brightness))
                                .color(theme::text())
                                .size(16.0),
                        );
                        ui.end_row();
                        ui.label(
                            RichText::new("轮廓点数")
                                .color(theme::subtext0())
                                .size(14.0),
                        );
                        ui.label(
                            RichText::new(obs.contour.len().to_string())
                                .color(theme::text())
                                .size(16.0),
                        );
                        ui.end_row();
                    });
            } else {
                ui.label(
                    RichText::new("未检测到目标")
                        .color(theme::overlay0())
                        .size(16.0),
                );
            }
        });

        ui.add_space(16.0);

        self.card(ui, "模型候选", |ui| {
            if obs.candidates.is_empty() {
                ui.label(RichText::new("无候选").color(theme::overlay0()).size(16.0));
            } else {
                egui::Grid::new("candidates_grid_sidebar")
                    .num_columns(5)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("类别").color(theme::subtext0()).size(14.0));
                        ui.label(RichText::new("置信度").color(theme::subtext0()).size(14.0));
                        ui.label(RichText::new("中心 X").color(theme::subtext0()).size(14.0));
                        ui.label(RichText::new("中心 Y").color(theme::subtext0()).size(14.0));
                        ui.label(RichText::new("边界框").color(theme::subtext0()).size(14.0));
                        ui.end_row();
                        for cand in &obs.candidates {
                            let class_color = match cand.class_id {
                                0 => theme::MAUVE,
                                1 => theme::RED,
                                2 => theme::BLUE,
                                _ => theme::overlay0(),
                            };
                            ui.label(
                                RichText::new(LaserObservation::class_name(cand.class_id))
                                    .color(class_color)
                                    .size(14.0),
                            );
                            ui.label(
                                RichText::new(format!("{:.0}%", cand.score * 100.0))
                                    .color(theme::text())
                                    .size(14.0),
                            );
                            ui.label(
                                RichText::new(format!("{:.1}", cand.center[0]))
                                    .color(theme::text())
                                    .size(14.0),
                            );
                            ui.label(
                                RichText::new(format!("{:.1}", cand.center[1]))
                                    .color(theme::text())
                                    .size(14.0),
                            );
                            ui.label(
                                RichText::new(format!("{:.0}x{:.0}", cand.bbox[2], cand.bbox[3]))
                                    .color(theme::text())
                                    .size(14.0),
                            );
                            ui.end_row();
                        }
                    });
            }
        });
    }

    fn card(&self, ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(theme::card_bg())
            .stroke(egui::Stroke::new(1.0, theme::border()))
            .corner_radius(egui::CornerRadius::ZERO)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new(title).color(theme::text()).size(18.0));
                ui.add_space(10.0);
                add_contents(ui);
            });
    }
}

impl LaserPanel {
    fn draw_video_with_overlay(&self, ui: &mut egui::Ui, obs: &LaserObservation) {
        let available = ui.available_size();
        let width = available.x.max(320.0);
        let height = (width * 9.0 / 16.0).max(220.0);
        let size = Vec2::new(width, height);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, theme::panel_bg());

        let (scale_x, scale_y) = if let Ok(video) = self.video.lock() {
            if let Some(frame) = video.as_ref() {
                let sx = rect.width() / frame.width as f32;
                let sy = rect.height() / frame.height as f32;
                let rgba = bgr_to_rgba(&frame.data, frame.width as usize, frame.height as usize);
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [frame.width as usize, frame.height as usize],
                    &rgba,
                );
                let texture =
                    ui.ctx()
                        .load_texture("video_frame", image, egui::TextureOptions::LINEAR);
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                (sx, sy)
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "等待视频流...",
                    egui::FontId::proportional(24.0),
                    theme::text_on_dark_muted(),
                );
                (rect.width() / 1920.0, rect.height() / 1080.0)
            }
        } else {
            (rect.width() / 1920.0, rect.height() / 1080.0)
        };

        draw_overlay(&painter, rect, obs, scale_x, scale_y);
    }
}

fn bgr_to_rgba(bgr: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; width * height * 4];
    for i in 0..(width * height) {
        let b = bgr[i * 3];
        let g = bgr[i * 3 + 1];
        let r = bgr[i * 3 + 2];
        rgba[i * 4] = r;
        rgba[i * 4 + 1] = g;
        rgba[i * 4 + 2] = b;
        rgba[i * 4 + 3] = 255;
    }
    rgba
}

fn draw_overlay(
    painter: &egui::Painter,
    rect: egui::Rect,
    obs: &LaserObservation,
    scale_x: f32,
    scale_y: f32,
) {
    for cand in &obs.candidates {
        if cand.score < 0.25 {
            continue;
        }
        let color = match cand.class_id {
            0 => Color32::from_rgb(255, 0, 255),
            1 => Color32::from_rgb(255, 50, 50),
            2 => Color32::from_rgb(50, 100, 255),
            _ => Color32::from_rgb(100, 255, 100),
        };
        let x = rect.left() + cand.bbox[0] * scale_x;
        let y = rect.top() + cand.bbox[1] * scale_y;
        painter.rect_stroke(
            egui::Rect::from_min_size(
                Pos2::new(x, y),
                Vec2::new(cand.bbox[2] * scale_x, cand.bbox[3] * scale_y),
            ),
            2.0,
            (2.0, color),
            egui::StrokeKind::Outside,
        );
        painter.text(
            Pos2::new(x, y - 4.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "{} {:.0}%",
                LaserObservation::class_name(cand.class_id),
                cand.score * 100.0
            ),
            egui::FontId::proportional(12.0),
            color,
        );
    }
    if obs.detected {
        let cx = rect.left() + obs.center[0] * scale_x;
        let cy = rect.top() + obs.center[1] * scale_y;
        painter.line_segment(
            [Pos2::new(cx - 8.0, cy), Pos2::new(cx + 8.0, cy)],
            (1.0, Color32::YELLOW),
        );
        painter.line_segment(
            [Pos2::new(cx, cy - 8.0), Pos2::new(cx, cy + 8.0)],
            (1.0, Color32::YELLOW),
        );
    }
    if !obs.contour.is_empty() && obs.contour.len() >= 3 {
        let points: Vec<Pos2> = obs
            .contour
            .iter()
            .map(|p| Pos2::new(rect.left() + p[0] * scale_x, rect.top() + p[1] * scale_y))
            .collect();
        for i in 0..points.len() {
            painter.line_segment(
                [points[i], points[(i + 1) % points.len()]],
                (1.0, theme::GREEN),
            );
        }
    }
}
