use egui::{Color32, Pos2, RichText, Vec2};
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

    pub fn show_blood(&self, ui: &mut egui::Ui) {
        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };
        self.section_header(ui, "血量");
        self.blood_bar(ui, "英雄", info.hero_blood, 200, theme::HERO_COLOR);
        self.blood_bar(ui, "工程", info.engineer_blood, 200, theme::ENGINEER_COLOR);
        self.blood_bar(ui, "步兵1", info.infantry_blood_1, 200, theme::INFANTRY1_COLOR);
        self.blood_bar(ui, "步兵2", info.infantry_blood_2, 200, theme::INFANTRY2_COLOR);
        self.blood_bar(ui, "前哨站", info.saven_blood, 200, theme::TEAL);
        self.blood_bar(ui, "哨兵", info.sentinel_blood, 400, theme::SENTINEL_COLOR);
    }

    pub fn show_ammo(&self, ui: &mut egui::Ui) {
        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };
        self.section_header(ui, "弹药");
        egui::Grid::new("ammo_grid")
            .num_columns(2)
            .spacing([32.0, 8.0])
            .show(ui, |ui| {
                self.ammo_row(ui, "英雄", info.hero_ammunition, theme::HERO_COLOR);
                self.ammo_row(ui, "步兵1", info.infantry_ammunition_1, theme::INFANTRY1_COLOR);
                self.ammo_row(ui, "步兵2", info.infantry_ammunition_2, theme::INFANTRY2_COLOR);
                self.ammo_row(ui, "无人机", info.drone_ammunition, theme::DRONE_COLOR);
                self.ammo_row(ui, "哨兵", info.sentinel_ammunition, theme::SENTINEL_COLOR);
            });
    }

    pub fn show_economy(&self, ui: &mut egui::Ui) {
        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };
        self.section_header(ui, "经济");
        let econ_ratio = if info.economic_total > 0 {
            info.economic_remain as f32 / info.economic_total as f32
        } else {
            0.0
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}", info.economic_remain))
                    .color(theme::TEXT)
                    .size(24.0),
            );
            ui.label(
                RichText::new(format!(" / {}", info.economic_total))
                    .color(theme::OVERLAY0)
                    .size(16.0),
            );
        });
        ui.add_space(6.0);
        self.progress_bar(ui, econ_ratio, theme::SAPPHIRE, None);
    }

    pub fn show_gains(&self, ui: &mut egui::Ui) {
        let info = match self.shared.lock() {
            Ok(state) => state.clone(),
            Err(_) => return,
        };
        self.section_header(ui, "增益");
        egui::Grid::new("gains_grid")
            .num_columns(6)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("机器人").color(theme::SUBTEXT0).size(14.0));
                ui.label(RichText::new("回血").color(theme::SUBTEXT0).size(14.0));
                ui.label(RichText::new("冷却").color(theme::SUBTEXT0).size(14.0));
                ui.label(RichText::new("防御").color(theme::SUBTEXT0).size(14.0));
                ui.label(RichText::new("降防").color(theme::SUBTEXT0).size(14.0));
                ui.label(RichText::new("攻击").color(theme::SUBTEXT0).size(14.0));
                ui.end_row();

                self.gain_row(ui, "英雄", &info.hero_gain, theme::HERO_COLOR);
                self.gain_row(ui, "工程", &info.engineer_gain, theme::ENGINEER_COLOR);
                self.gain_row(ui, "步兵1", &info.infantry_gain_1, theme::INFANTRY1_COLOR);
                self.gain_row(ui, "步兵2", &info.infantry_gain_2, theme::INFANTRY2_COLOR);
                self.gain_row(ui, "哨兵", &info.sentinel_gain, theme::SENTINEL_COLOR);
            });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("哨兵姿态").color(theme::SUBTEXT0).size(14.0));
            ui.label(RichText::new(format!("{}", info.sentinel_posture)).color(theme::TEXT).size(16.0));
        });
    }

    fn section_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.label(RichText::new(title).color(theme::TEXT).size(16.0));
        ui.add_space(2.0);
        let rect = ui.available_rect_before_wrap();
        let line_y = ui.cursor().top() + 2.0;
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), line_y),
                Pos2::new(rect.right(), line_y),
            ],
            (0.5, theme::SURFACE1),
        );
        ui.add_space(8.0);
    }

    fn blood_bar(&self, ui: &mut egui::Ui, name: &str, current: u16, max: u16, robot_color: Color32) {
        let ratio = current as f32 / max as f32;
        let fill_color = if ratio > 0.5 {
            robot_color
        } else if ratio > 0.25 {
            theme::YELLOW
        } else {
            theme::RED
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{:>6}", name))
                    .color(theme::SUBTEXT0)
                    .size(15.0),
            );
            ui.add_space(12.0);
            self.progress_bar(ui, ratio, fill_color, Some(current));
        });
    }

    fn progress_bar(&self, ui: &mut egui::Ui, ratio: f32, fill: Color32, value: Option<u16>) {
        let height = 20.0;
        let width = 260.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
        let rounding = egui::CornerRadius::same(10);

        ui.painter().rect_filled(rect, rounding, theme::SURFACE0);

        let fill_width = rect.width() * ratio.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
            ui.painter().rect_filled(fill_rect, rounding, fill);
        }

        if let Some(val) = value {
            let text = format!("{}", val);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(13.0),
                theme::TEXT,
            );
        }
    }

    fn ammo_row(&self, ui: &mut egui::Ui, name: &str, ammo: u16, color: Color32) {
        ui.label(RichText::new(name).color(theme::SUBTEXT0).size(15.0));
        ui.label(RichText::new(format!("{}", ammo)).color(color).size(20.0));
        ui.end_row();
    }

    fn gain_row(&self, ui: &mut egui::Ui, name: &str, gain: &[u8; 7], color: Color32) {
        ui.label(RichText::new(name).color(color).size(15.0));
        ui.label(RichText::new(format!("{}", gain[0])).color(theme::TEXT).size(15.0));
        let cooling = u16::from_le_bytes([gain[1], gain[2]]);
        ui.label(RichText::new(format!("{}", cooling)).color(theme::TEXT).size(15.0));
        ui.label(RichText::new(format!("{}", gain[3])).color(theme::TEXT).size(15.0));
        ui.label(RichText::new(format!("{}", gain[4])).color(theme::TEXT).size(15.0));
        let attack = u16::from_le_bytes([gain[5], gain[6]]);
        ui.label(RichText::new(format!("{}", attack)).color(theme::TEXT).size(15.0));
        ui.end_row();
    }
}
