use std::sync::{Arc, Mutex, Once};
use std::thread;

use tokio::sync::watch;

use crate::protocol::RoboMasterSignalInfo;
use crate::tcp_client;
use crate::theme;
use crate::widgets::{MinimapWidget, StatusPanels};

static FONT_ONCE: Once = Once::new();

const HANDLE_SIZE: f32 = 4.0;
const MIN_SECTION: f32 = 80.0;

pub struct RadarApp {
    shared: Arc<Mutex<RoboMasterSignalInfo>>,
    connection_status: ConnectionStatus,
    last_update: Option<std::time::Instant>,
    _shutdown_tx: watch::Sender<bool>,
    blood_h: f32,
    ammo_h: f32,
    economy_h: f32,
}

#[derive(PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connected,
}

impl Default for RadarApp {
    fn default() -> Self {
        let shared = Arc::new(Mutex::new(RoboMasterSignalInfo::default()));
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let shared_clone = shared.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                tcp_client::run_signal_client(
                    "127.0.0.1:2000",
                    shared_clone,
                    shutdown_rx,
                )
                .await;
            });
        });

        Self {
            shared,
            connection_status: ConnectionStatus::Disconnected,
            last_update: None,
            _shutdown_tx: shutdown_tx,
            blood_h: 240.0,
            ammo_h: 200.0,
            economy_h: 120.0,
        }
    }
}

impl eframe::App for RadarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_fonts(ctx);
        self.update_connection_status();
        self.apply_theme(ctx);

        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::new().fill(theme::MANTLE).inner_margin(egui::Margin::symmetric(16, 10)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("radar hud").color(theme::SUBTEXT0).size(16.0));
                    ui.separator();
                    match self.connection_status {
                        ConnectionStatus::Connected => ui.colored_label(theme::CONNECTED, "● Connected"),
                        ConnectionStatus::Disconnected => ui.colored_label(theme::DISCONNECTED, "● Disconnected"),
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(last) = self.last_update {
                            let elapsed = last.elapsed().as_secs_f32();
                            ui.label(egui::RichText::new(format!("{:.1}s", elapsed)).color(theme::OVERLAY0).size(14.0));
                        }
                    });
                });
            });

        egui::SidePanel::left("minimap_panel")
            .default_width(420.0)
            .frame(egui::Frame::new().fill(theme::BASE).inner_margin(12))
            .show(ctx, |ui| {
                MinimapWidget::new(self.shared.clone()).show(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BASE))
            .show(ctx, |ui| {
                let total = ui.available_height();
                let gains_h = (total - self.blood_h - self.ammo_h - self.economy_h - HANDLE_SIZE * 3.0).max(MIN_SECTION);

                self.draw_section(ui, self.blood_h, |ui, shared| {
                    StatusPanels::new(shared).show_blood(ui);
                });
                self.draw_handle(ui, 0);

                self.draw_section(ui, self.ammo_h, |ui, shared| {
                    StatusPanels::new(shared).show_ammo(ui);
                });
                self.draw_handle(ui, 1);

                self.draw_section(ui, self.economy_h, |ui, shared| {
                    StatusPanels::new(shared).show_economy(ui);
                });
                self.draw_handle(ui, 2);

                self.draw_section(ui, gains_h, |ui, shared| {
                    StatusPanels::new(shared).show_gains(ui);
                });
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

impl RadarApp {
    fn draw_section(&self, ui: &mut egui::Ui, height: f32, add_contents: impl FnOnce(&mut egui::Ui, Arc<Mutex<RoboMasterSignalInfo>>)) {
        let rect = egui::Rect::from_min_size(
            ui.cursor().left_top(),
            egui::Vec2::new(ui.available_width(), height),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::TOP)), |ui| {
            egui::Frame::new().fill(theme::BASE).inner_margin(egui::Margin::symmetric(16, 8)).show(ui, |ui| {
                add_contents(ui, self.shared.clone());
            });
        });
    }

    fn draw_handle(&mut self, ui: &mut egui::Ui, index: usize) {
        let available = ui.cursor();
        let rect = egui::Rect::from_min_size(
            available.left_top(),
            egui::Vec2::new(available.width(), HANDLE_SIZE),
        );

        let id = egui::Id::new("handle").with(index);
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        if response.hovered() || response.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeRow);
        }

        let color = if response.hovered() || response.dragged() {
            theme::BLUE
        } else {
            theme::SURFACE1
        };

        ui.painter().line_segment(
            [
                egui::Pos2::new(rect.left() + 16.0, rect.center().y),
                egui::Pos2::new(rect.right() - 16.0, rect.center().y),
            ],
            (2.0, color),
        );

        if response.dragged() {
            let delta = response.drag_delta().y;
            match index {
                0 => self.blood_h = (self.blood_h + delta).max(MIN_SECTION),
                1 => self.ammo_h = (self.ammo_h + delta).max(MIN_SECTION),
                2 => self.economy_h = (self.economy_h + delta).max(MIN_SECTION),
                _ => {}
            }
        }
    }

    fn setup_fonts(&self, ctx: &egui::Context) {
        FONT_ONCE.call_once(|| {
            let mut fonts = egui::FontDefinitions::default();

            // English primary: JetBrainsMono Nerd Font Propo (proportional, with icons)
            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/JetBrainsMonoNerdFontPropo-Regular.ttf") {
                log::info!("Loaded JetBrainsMono NFP (proportional English)");
                fonts.font_data.insert("jb_propo".to_owned(), egui::FontData::from_owned(data).into());
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "jb_propo".to_owned());
            }

            // CJK fallback: LXGW WenKai GB Screen
            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/LXGWWenKaiGBScreen.ttf") {
                log::info!("Loaded LXGW WenKai GB Screen (CJK fallback)");
                fonts.font_data.insert("lxgw".to_owned(), egui::FontData::from_owned(data).into());
                fonts.families.entry(egui::FontFamily::Proportional).or_default().push("lxgw".to_owned());
            }

            // Monospace: JetBrains Mono (numbers, code)
            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf") {
                log::info!("Loaded JetBrains Mono (monospace)");
                fonts.font_data.insert("jb_mono".to_owned(), egui::FontData::from_owned(data).into());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "jb_mono".to_owned());
            }

            // Monospace CJK fallback: LXGW WenKai Mono
            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/LXGWWenKaiMonoGBScreen.ttf") {
                log::info!("Loaded LXGW WenKai Mono GB Screen (mono CJK fallback)");
                fonts.font_data.insert("lxgw_mono".to_owned(), egui::FontData::from_owned(data).into());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().push("lxgw_mono".to_owned());
            }

            ctx.set_fonts(fonts);
        });
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut v = egui::Visuals::dark();
        v.override_text_color = Some(theme::TEXT);
        v.widgets.inactive.bg_fill = theme::SURFACE0;
        v.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        v.widgets.inactive.weak_bg_fill = theme::SURFACE0;
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme::SUBTEXT0);
        v.widgets.hovered.bg_fill = theme::SURFACE1;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme::SURFACE2);
        v.widgets.active.bg_fill = theme::SURFACE2;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme::OVERLAY0);
        v.selection.bg_fill = theme::SURFACE2;
        v.selection.stroke = egui::Stroke::new(1.0, theme::BLUE);
        v.extreme_bg_color = theme::CRUST;
        v.faint_bg_color = theme::MANTLE;
        v.window_fill = theme::BASE;
        v.window_stroke = egui::Stroke::new(0.5, theme::SURFACE1);
        ctx.set_visuals(v);
    }

    fn update_connection_status(&mut self) {
        if let Ok(info) = self.shared.lock() {
            let is_default = info.hero_position == [0, 0]
                && info.hero_blood == 0
                && info.hero_ammunition == 0;

            if !is_default {
                self.connection_status = ConnectionStatus::Connected;
                self.last_update = Some(std::time::Instant::now());
            } else if let Some(last) = self.last_update {
                if last.elapsed().as_secs() > 5 {
                    self.connection_status = ConnectionStatus::Disconnected;
                }
            }
        }
    }
}
