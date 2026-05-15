use egui::{Color32, RichText, Vec2};
use std::sync::{Arc, Mutex};

use crate::protocol::RoboMasterSignalInfo;
use crate::theme;

pub struct StatusPanels {
    shared: Arc<Mutex<RoboMasterSignalInfo>>,
}

impl StatusPanels {
    pub fn new(shared: Arc<Mutex<RoboMasterSignalInfo>>) -> Self {
        Self { shared }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };

        self.card(
            ui,
            "血量总览",
            "主战单位与关键建筑生命值",
            |ui| {
                self.blood_row(ui, "英雄", info.hero_blood, 200, theme::HERO_COLOR);
                self.blood_row(ui, "工程", info.engineer_blood, 200, theme::ENGINEER_COLOR);
                self.blood_row(
                    ui,
                    "步兵1",
                    info.infantry_blood_1,
                    200,
                    theme::INFANTRY1_COLOR,
                );
                self.blood_row(
                    ui,
                    "步兵2",
                    info.infantry_blood_2,
                    200,
                    theme::INFANTRY2_COLOR,
                );
                self.blood_row(ui, "前哨站", info.saven_blood, 200, theme::TEAL);
                self.blood_row(ui, "哨兵", info.sentinel_blood, 400, theme::SENTINEL_COLOR);
            },
        );

        ui.add_space(14.0);

        self.card(ui, "弹药", "即时载弹量", |ui| {
            egui::Grid::new("ammo_grid")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    self.ammo_row(ui, "英雄", info.hero_ammunition, theme::HERO_COLOR);
                    self.ammo_row(
                        ui,
                        "步兵1",
                        info.infantry_ammunition_1,
                        theme::INFANTRY1_COLOR,
                    );
                    self.ammo_row(
                        ui,
                        "步兵2",
                        info.infantry_ammunition_2,
                        theme::INFANTRY2_COLOR,
                    );
                    self.ammo_row(ui, "无人机", info.drone_ammunition, theme::DRONE_COLOR);
                    self.ammo_row(ui, "哨兵", info.sentinel_ammunition, theme::SENTINEL_COLOR);
                });
        });

        ui.add_space(14.0);

        self.card(ui, "经济", "当前资源 / 已获得资源", |ui| {
            let econ_ratio = if info.economic_total > 0 {
                info.economic_remain as f32 / info.economic_total as f32
            } else {
                0.0
            };

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{}", info.economic_remain))
                        .color(theme::text())
                        .size(30.0),
                );
                ui.label(
                    RichText::new(format!("/ {}", info.economic_total))
                        .color(theme::text_muted())
                        .size(18.0),
                );
            });
            ui.add_space(8.0);
            self.progress_bar(ui, econ_ratio, theme::BLUE, None);
        });

        ui.add_space(14.0);

        self.card(ui, "占领状态", "点位控制概览", |ui| {
            ui.horizontal_wrapped(|ui| {
                let labels = ["A", "B", "C", "D", "E", "F"];
                for (i, label) in labels.iter().enumerate() {
                    let active = info.occupation_status[i] != 0;
                    let fill = if active {
                        egui::Color32::from_rgb(0xe7, 0xf8, 0xee)
                    } else {
                        theme::card_bg_muted()
                    };
                    let stroke = if active {
                        theme::GREEN
                    } else {
                        theme::border()
                    };
                    let text = if active {
                        theme::GREEN
                    } else {
                        theme::text_faint()
                    };
                    egui::Frame::new()
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0, stroke))
                        .corner_radius(egui::CornerRadius::same(255))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(*label).color(text).size(15.0));
                        });
                    ui.add_space(4.0);
                }
            });
        });

        ui.add_space(14.0);

        self.card(ui, "增益矩阵", "关键 buff 参数", |ui| {
            egui::Grid::new("gains_grid")
                .num_columns(6)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    for label in ["机器人", "回血", "冷却", "防御", "降防", "攻击"] {
                        ui.label(RichText::new(label).color(theme::text_faint()).size(13.0));
                    }
                    ui.end_row();

                    self.gain_row(ui, "英雄", &info.hero_gain, theme::HERO_COLOR);
                    self.gain_row(ui, "工程", &info.engineer_gain, theme::ENGINEER_COLOR);
                    self.gain_row(ui, "步兵1", &info.infantry_gain_1, theme::INFANTRY1_COLOR);
                    self.gain_row(ui, "步兵2", &info.infantry_gain_2, theme::INFANTRY2_COLOR);
                    self.gain_row(ui, "哨兵", &info.sentinel_gain, theme::SENTINEL_COLOR);
                });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("哨兵姿态")
                        .color(theme::text_muted())
                        .size(14.0),
                );
                ui.label(
                    RichText::new(info.sentinel_posture.to_string())
                        .color(theme::text())
                        .size(18.0),
                );
            });
        });
    }

    fn card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        subtitle: &str,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        egui::Frame::new()
            .fill(theme::card_bg())
            .stroke(egui::Stroke::new(1.0, theme::border()))
            .corner_radius(egui::CornerRadius::same(18))
            .shadow(egui::epaint::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: theme::shadow(),
            })
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(title).color(theme::text()).size(17.0));
                    ui.label(
                        RichText::new(subtitle)
                            .color(theme::text_muted())
                            .size(12.0),
                    );
                });
                ui.add_space(12.0);
                add_contents(ui);
            });
    }

    fn blood_row(
        &self,
        ui: &mut egui::Ui,
        name: &str,
        current: u16,
        max: u16,
        robot_color: Color32,
    ) {
        let ratio = current as f32 / max as f32;
        let fill_color = if ratio > 0.5 {
            robot_color
        } else if ratio > 0.25 {
            theme::YELLOW
        } else {
            theme::RED
        };

        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(name).color(theme::text_muted()).size(14.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{}", current))
                        .color(theme::text())
                        .size(14.0),
                );
            });
        });
        ui.add_space(6.0);
        self.progress_bar(ui, ratio, fill_color, None);
        ui.add_space(10.0);
    }

    fn progress_bar(&self, ui: &mut egui::Ui, ratio: f32, fill: Color32, value: Option<u16>) {
        let height = 16.0;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
        let rounding = egui::CornerRadius::same(255);

        ui.painter()
            .rect_filled(rect, rounding, theme::card_bg_muted());

        let fill_width = rect.width() * ratio.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            let fill_rect =
                egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
            ui.painter().rect_filled(fill_rect, rounding, fill);
        }

        if let Some(val) = value {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                val.to_string(),
                egui::FontId::proportional(12.0),
                theme::text(),
            );
        }
    }

    fn ammo_row(&self, ui: &mut egui::Ui, name: &str, ammo: u16, color: Color32) {
        ui.label(RichText::new(name).color(theme::text_muted()).size(14.0));
        ui.label(RichText::new(ammo.to_string()).color(color).size(24.0));
        ui.end_row();
    }

    fn gain_row(&self, ui: &mut egui::Ui, name: &str, gain: &[u8; 7], color: Color32) {
        ui.label(RichText::new(name).color(color).size(14.0));
        ui.label(
            RichText::new(gain[0].to_string())
                .color(theme::text())
                .size(14.0),
        );
        let cooling = u16::from_le_bytes([gain[1], gain[2]]);
        ui.label(
            RichText::new(cooling.to_string())
                .color(theme::text())
                .size(14.0),
        );
        ui.label(
            RichText::new(gain[3].to_string())
                .color(theme::text())
                .size(14.0),
        );
        ui.label(
            RichText::new(gain[4].to_string())
                .color(theme::text())
                .size(14.0),
        );
        let attack = u16::from_le_bytes([gain[5], gain[6]]);
        ui.label(
            RichText::new(attack.to_string())
                .color(theme::text())
                .size(14.0),
        );
        ui.end_row();
    }
}
