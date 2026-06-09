use super::RadarApp;
use crate::theme;

impl RadarApp {
    pub(super) fn show_theme_toggle(&mut self, ctx: &egui::Context) {
        let accent = if self.dark_mode {
            egui::Color32::from_rgb(0xc8, 0xd7, 0xff)
        } else {
            egui::Color32::from_rgb(0xf4, 0xb9, 0x42)
        };
        let hover_text = if self.dark_mode {
            "Switch to light mode"
        } else {
            "Switch to dark mode"
        };
        let track_fill = theme::card_bg();
        let track_stroke = egui::Stroke::new(1.0, Self::alpha(accent, 52));
        let knob_fill = if self.dark_mode {
            egui::Color32::from_rgb(0x3a, 0x44, 0x62)
        } else {
            egui::Color32::from_rgb(0xff, 0xef, 0xc2)
        };
        let knob_stroke = egui::Stroke::new(1.0, Self::alpha(accent, 92));
        let animation = ctx.animate_bool(egui::Id::new("theme_toggle_knob"), self.dark_mode);
        let sun_color = if self.dark_mode {
            Self::alpha(theme::text_faint(), 170)
        } else {
            egui::Color32::from_rgb(0xf2, 0xb7, 0x21)
        };
        let moon_color = if self.dark_mode {
            egui::Color32::from_rgb(0xd9, 0xe7, 0xff)
        } else {
            egui::Color32::from_rgb(0x6f, 0x7f, 0x9e)
        };

        egui::Area::new("global_theme_toggle".into())
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(18.0, -18.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(54.0, 28.0), egui::Sense::click());
                let response = response.on_hover_text(hover_text);

                let painter = ui.painter();
                painter.rect_filled(rect, egui::CornerRadius::same(14), track_fill);
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::same(14),
                    track_stroke,
                    egui::StrokeKind::Middle,
                );

                let knob_center_x =
                    egui::lerp((rect.left() + 14.0)..=(rect.right() - 14.0), animation);
                let knob_rect = egui::Rect::from_center_size(
                    egui::pos2(knob_center_x, rect.center().y),
                    egui::vec2(24.0, 24.0),
                );
                painter.rect_filled(knob_rect, egui::CornerRadius::same(12), knob_fill);
                painter.rect_stroke(
                    knob_rect,
                    egui::CornerRadius::same(12),
                    knob_stroke,
                    egui::StrokeKind::Middle,
                );

                let sun_center = egui::pos2(rect.left() + 14.0, rect.center().y);
                let moon_center = egui::pos2(rect.right() - 14.0, rect.center().y);

                Self::draw_sun_icon(painter, sun_center, 4.0, sun_color);
                Self::draw_moon_icon(
                    painter,
                    moon_center,
                    5.0,
                    moon_color,
                    if self.dark_mode {
                        knob_fill
                    } else {
                        track_fill
                    },
                );

                if animation > 0.0 && animation < 1.0 {
                    ctx.request_repaint();
                }

                if response.clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
    }

    pub(super) fn apply_theme(&self, ctx: &egui::Context) {
        let mut v = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.override_text_color = Some(theme::text());
        v.widgets.inactive.bg_fill = theme::card_bg();
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme::border());
        v.widgets.inactive.weak_bg_fill = theme::card_bg_muted();
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme::text_muted());
        v.widgets.hovered.bg_fill = theme::card_bg_muted();
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme::border_strong());
        v.widgets.active.bg_fill = theme::BLUE_SOFT;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme::BLUE);
        v.widgets.open.bg_fill = theme::card_bg();
        v.widgets.open.bg_stroke = egui::Stroke::new(1.0, theme::border_strong());
        v.selection.bg_fill = theme::BLUE_SOFT;
        v.selection.stroke = egui::Stroke::new(1.0, theme::BLUE);
        v.extreme_bg_color = theme::card_bg();
        v.faint_bg_color = theme::card_bg_muted();
        v.window_fill = theme::panel_bg();
        v.window_stroke = egui::Stroke::new(1.0, theme::border());
        ctx.set_visuals(v);
    }

    fn draw_sun_icon(
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        color: egui::Color32,
    ) {
        painter.circle_filled(center, radius, color);
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::TAU / 8.0;
            let direction = egui::vec2(angle.cos(), angle.sin());
            painter.line_segment(
                [
                    center + direction * (radius + 2.0),
                    center + direction * (radius + 4.5),
                ],
                egui::Stroke::new(1.4, color),
            );
        }
    }

    fn draw_moon_icon(
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        color: egui::Color32,
        cutout_fill: egui::Color32,
    ) {
        painter.circle_filled(center, radius, color);
        painter.circle_filled(center + egui::vec2(2.6, -0.4), radius - 0.8, cutout_fill);
        painter.circle_stroke(
            center + egui::vec2(-0.4, 0.0),
            radius - 0.5,
            egui::Stroke::new(1.0, Self::alpha(color, 110)),
        );
        painter.circle_filled(
            center + egui::vec2(-3.0, -3.0),
            0.8,
            Self::alpha(color, 220),
        );
        painter.circle_filled(
            center + egui::vec2(-1.4, -4.6),
            0.45,
            Self::alpha(color, 170),
        );
    }

    fn alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
    }
}
