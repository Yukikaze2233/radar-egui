use std::sync::{Arc, Mutex, Once};
use std::thread;

use tokio::sync::watch;

use crate::laser_protocol::LaserObservation;
use crate::protocol::RoboMasterSignalInfo;
use crate::rerun_viz::RerunVisualizer;
use crate::script_runner::{self, LaserScript, ScriptRunner};
use crate::tcp_client;
use crate::theme;
use crate::udp_client;
use crate::video_stream;
use crate::video_stream::VideoFrame;
use crate::widgets::{LaserPanel, MinimapWidget, StatusPanels};

static FONT_ONCE: Once = Once::new();
const MINIMAP_BG_PATH: &str = "assets/minimap_bg.png";
const LOGO_PATH: &str = "assets/logo.png";
const MINIMAP_DEFAULT_PAN_Y: f32 = 18.0;

#[derive(PartialEq, Clone, Copy)]
enum ActiveTab {
    Radar,
    Laser,
}

#[derive(Clone, Copy, PartialEq)]
enum EnemyColor {
    Red,
    Blue,
    Auto,
}

impl EnemyColor {
    fn label(&self) -> &str {
        match self {
            EnemyColor::Red => "Red",
            EnemyColor::Blue => "Blue",
            EnemyColor::Auto => "Auto",
        }
    }

    fn fifo_cmd(&self) -> &str {
        match self {
            EnemyColor::Red => "enemy red",
            EnemyColor::Blue => "enemy blue",
            EnemyColor::Auto => "enemy auto",
        }
    }
}

pub struct RadarApp {
    active_tab: ActiveTab,
    dark_mode: bool,
    minimap_texture: Option<egui::TextureHandle>,
    minimap_texture_failed: bool,
    minimap_pan: egui::Vec2,
    minimap_zoom: f32,
    logo_texture: Option<egui::TextureHandle>,
    logo_texture_failed: bool,

    shared: Arc<Mutex<RoboMasterSignalInfo>>,
    connection_status: ConnectionStatus,
    last_update: Option<std::time::Instant>,
    shutdown_tx: watch::Sender<bool>,
    ip: String,
    port: String,
    error_message: Option<String>,
    data_count: u64,
    start_time: std::time::Instant,
    rerun_viz: RerunVisualizer,

    laser_shared: Arc<Mutex<LaserObservation>>,
    laser_shutdown_tx: watch::Sender<bool>,
    laser_port: String,
    video_shared: Arc<Mutex<Option<VideoFrame>>>,
    video_shutdown_tx: watch::Sender<bool>,

    script_runner: ScriptRunner,
    enemy_color: EnemyColor,
    stream_on_start: bool,
    record_on_start: bool,
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
        let addr = "127.0.0.1:2000".to_string();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                tcp_client::run_signal_client(&addr, shared_clone, shutdown_rx).await;
            });
        });

        let laser_shared = Arc::new(Mutex::new(LaserObservation::default()));
        let (laser_shutdown_tx, laser_shutdown_rx) = watch::channel(false);
        let laser_shared_clone = laser_shared.clone();
        let laser_port = 5001;
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                udp_client::run_laser_client(laser_port, laser_shared_clone, laser_shutdown_rx)
                    .await;
            });
        });

        let video_shared: Arc<Mutex<Option<VideoFrame>>> = Arc::new(Mutex::new(None));
        let (video_shutdown_tx, _video_shutdown_rx) = watch::channel(false);

        Self {
            active_tab: ActiveTab::Radar,
            dark_mode: false,
            minimap_texture: None,
            minimap_texture_failed: false,
            minimap_pan: egui::vec2(0.0, MINIMAP_DEFAULT_PAN_Y),
            minimap_zoom: 1.0,
            logo_texture: None,
            logo_texture_failed: false,
            shared,
            connection_status: ConnectionStatus::Disconnected,
            last_update: None,
            shutdown_tx,
            ip: "127.0.0.1".to_string(),
            port: "2000".to_string(),
            error_message: None,
            data_count: 0,
            start_time: std::time::Instant::now(),
            rerun_viz: RerunVisualizer::new(),
            laser_shared,
            laser_shutdown_tx,
            laser_port: laser_port.to_string(),
            video_shared,
            video_shutdown_tx,
            script_runner: ScriptRunner::new(),
            enemy_color: EnemyColor::Auto,
            stream_on_start: true,
            record_on_start: false,
        }
    }
}

impl RadarApp {
    fn reconnect(&mut self) {
        let _ = self.shutdown_tx.send(true);

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        self.shutdown_tx = shutdown_tx;

        self.connection_status = ConnectionStatus::Disconnected;
        self.last_update = None;
        self.error_message = None;

        let shared = self.shared.clone();
        let addr = format!("{}:{}", self.ip, self.port);
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                tcp_client::run_signal_client(&addr, shared, shutdown_rx).await;
            });
        });
    }

    fn reconnect_laser(&mut self) {
        let _ = self.laser_shutdown_tx.send(true);

        let (laser_shutdown_tx, laser_shutdown_rx) = watch::channel(false);
        self.laser_shutdown_tx = laser_shutdown_tx;

        let laser_shared = self.laser_shared.clone();
        let port: u16 = self.laser_port.parse().unwrap_or(5001);
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                udp_client::run_laser_client(port, laser_shared, laser_shutdown_rx).await;
            });
        });
    }

    fn send_laser_command(&self, cmd: &str) {
        let cmd = cmd.to_owned();
        std::thread::spawn(move || {
            if let Err(e) = script_runner::send_fifo(&cmd) {
                log::warn!("Failed to send laser command '{}': {}", cmd, e);
            }
        });
    }

    fn ensure_video_started(&mut self) {
        use std::sync::atomic::{AtomicBool, Ordering};
        static STARTED: AtomicBool = AtomicBool::new(false);
        if STARTED.swap(true, Ordering::Relaxed) {
            return;
        }

        let _ = self.video_shutdown_tx.send(true);
        let (tx, rx) = watch::channel(false);
        self.video_shutdown_tx = tx;

        let shared = self.video_shared.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                video_stream::run_video_client(shared, rx).await;
            });
        });
    }
}

impl eframe::App for RadarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_fonts(ctx);
        theme::set_dark_mode(self.dark_mode);
        self.ensure_minimap_texture(ctx);
        self.ensure_logo_texture(ctx);
        self.update_connection_status();
        self.apply_theme(ctx);

        match self.active_tab {
            ActiveTab::Radar => {
                egui::SidePanel::right("radar_inspector")
                    .exact_width(356.0)
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(
                        egui::Frame::new()
                            .fill(theme::panel_bg())
                            .inner_margin(egui::Margin::same(18)),
                    )
                    .show(ctx, |ui| {
                        self.show_radar_sidebar(ui);
                    });

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(theme::app_bg())
                            .inner_margin(egui::Margin::same(18)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(58.0, ui.available_height()),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    self.show_mode_rail(ui);
                                },
                            );
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Radar Workspace")
                                            .color(theme::text())
                                            .size(21.0),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(
                                            "white battle board / live robot overlay",
                                        )
                                        .color(theme::text_muted())
                                        .size(13.0),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Reset View").clicked() {
                                                self.minimap_pan =
                                                    egui::vec2(0.0, MINIMAP_DEFAULT_PAN_Y);
                                                self.minimap_zoom = 1.0;
                                            }
                                        },
                                    );
                                });
                                ui.add_space(14.0);
                                MinimapWidget::new(self.shared.clone()).show_with_state(
                                    ui,
                                    self.minimap_texture.as_ref(),
                                    &mut self.minimap_pan,
                                    &mut self.minimap_zoom,
                                );
                            });
                        });
                    });
            }
            ActiveTab::Laser => {
                self.ensure_video_started();

                egui::SidePanel::right("laser_inspector")
                    .exact_width(356.0)
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(
                        egui::Frame::new()
                            .fill(theme::panel_bg())
                            .inner_margin(egui::Margin::same(18)),
                    )
                    .show(ctx, |ui| {
                        self.show_laser_sidebar(ui);
                    });

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(theme::app_bg())
                            .inner_margin(egui::Margin::same(18)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(58.0, ui.available_height()),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    self.show_mode_rail(ui);
                                },
                            );
                            ui.add_space(12.0);
                            let content_width = ui.available_width();
                            ui.allocate_ui_with_layout(
                                egui::vec2(content_width, ui.available_height()),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("Laser Workspace")
                                                .color(theme::text())
                                                .size(21.0),
                                        );
                                        ui.add_space(12.0);
                                        ui.label(
                                            egui::RichText::new(
                                                "video feed / target overlay / live detections",
                                            )
                                            .color(theme::text_muted())
                                            .size(13.0),
                                        );
                                    });
                                    ui.add_space(14.0);
                                    LaserPanel::new(
                                        self.laser_shared.clone(),
                                        self.video_shared.clone(),
                                    )
                                    .show_video_stage(ui);
                                },
                            );
                        });
                    });
            }
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

impl RadarApp {
    fn ensure_minimap_texture(&mut self, ctx: &egui::Context) {
        if self.minimap_texture.is_some() || self.minimap_texture_failed {
            return;
        }

        match image::open(MINIMAP_BG_PATH) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.minimap_texture = Some(ctx.load_texture(
                    "unity_minimap_bg",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                log::info!("Loaded minimap background from {}", MINIMAP_BG_PATH);
            }
            Err(err) => {
                self.minimap_texture_failed = true;
                log::warn!(
                    "Failed to load minimap background from {}: {}",
                    MINIMAP_BG_PATH,
                    err
                );
            }
        }
    }

    fn ensure_logo_texture(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() || self.logo_texture_failed {
            return;
        }

        match image::open(LOGO_PATH) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.logo_texture =
                    Some(ctx.load_texture("rail_logo", color_image, egui::TextureOptions::LINEAR));
                log::info!("Loaded rail logo from {}", LOGO_PATH);
            }
            Err(err) => {
                self.logo_texture_failed = true;
                log::warn!("Failed to load rail logo from {}: {}", LOGO_PATH, err);
            }
        }
    }

    fn show_mode_rail(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if let Some(texture) = self.logo_texture.as_ref() {
                    ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_exact_size(egui::vec2(34.0, 34.0))
                            .corner_radius(egui::CornerRadius::same(255)),
                    );
                } else {
                    let (logo_rect, _) =
                        ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(logo_rect.center(), 17.0, theme::BLUE_SOFT);
                    ui.painter().text(
                        logo_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "R",
                        egui::FontId::proportional(16.0),
                        theme::BLUE,
                    );
                }

                ui.add_space(8.0);
                self.show_mode_button(ui, "◎", ActiveTab::Radar, "Radar");
                ui.add_space(8.0);
                self.show_mode_button(ui, "◈", ActiveTab::Laser, "Laser");
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("{} pkt", self.data_count))
                    .color(theme::text_muted())
                    .size(12.0),
            );
            ui.label(
                egui::RichText::new(format!("{}s", self.start_time.elapsed().as_secs()))
                    .color(theme::text_faint())
                    .size(12.0),
            );
            ui.add_space(ui.available_height().max(0.0));
            if ui.button(if self.dark_mode { "☼" } else { "☾" }).clicked() {
                self.dark_mode = !self.dark_mode;
            }
            ui.add_space(8.0);
        });
    }

    fn show_mode_button(&mut self, ui: &mut egui::Ui, title: &str, tab: ActiveTab, subtitle: &str) {
        let selected = self.active_tab == tab;
        let fill = if selected {
            theme::BLUE
        } else {
            theme::card_bg()
        };
        let stroke = if selected {
            egui::Stroke::NONE
        } else {
            egui::Stroke::new(1.0, theme::border())
        };
        let text_color = if selected {
            theme::text_on_dark()
        } else {
            theme::text()
        };
        let sub_color = if selected {
            theme::BLUE_SOFT
        } else {
            theme::text_faint()
        };

        let response = egui::Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::Margin::symmetric(8, 10))
            .show(ui, |ui| {
                ui.set_min_width(42.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(title).color(text_color).size(18.0));
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(subtitle).color(sub_color).size(9.0));
                });
            })
            .response
            .interact(egui::Sense::click());

        if response.clicked() {
            self.active_tab = tab;
        }
    }

    fn show_radar_sidebar(&mut self, ui: &mut egui::Ui) {
        Self::white_card(ui, "连接", |ui| {
            Self::status_chip(
                ui,
                self.connection_status == ConnectionStatus::Connected,
                "Signal feed",
            );
            ui.add_space(12.0);
            egui::Grid::new("radar_conn_grid")
                .num_columns(2)
                .min_col_width(78.0)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("IP")
                            .color(theme::text_muted())
                            .size(13.0),
                    );
                    ui.add(egui::TextEdit::singleline(&mut self.ip).desired_width(f32::INFINITY));
                    ui.end_row();
                    ui.label(
                        egui::RichText::new("Port")
                            .color(theme::text_muted())
                            .size(13.0),
                    );
                    ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(f32::INFINITY));
                    ui.end_row();
                });
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Reconnect radar stream"),
                )
                .clicked()
            {
                self.reconnect();
            }
            ui.add_space(8.0);
            egui::Grid::new("radar_meta_grid")
                .num_columns(2)
                .min_col_width(78.0)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Packets")
                            .color(theme::text_faint())
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(self.data_count.to_string())
                            .color(theme::text())
                            .size(12.0),
                    );
                    ui.end_row();
                    ui.label(
                        egui::RichText::new("Last live")
                            .color(theme::text_faint())
                            .size(12.0),
                    );
                    let age = self
                        .last_update
                        .map(|last| format!("{:.1}s", last.elapsed().as_secs_f32()))
                        .unwrap_or_else(|| "--".to_string());
                    ui.label(egui::RichText::new(age).color(theme::text()).size(12.0));
                    ui.end_row();
                });

            if let Some(err) = &self.error_message {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(err).color(theme::RED).size(12.0));
            }
        });

        ui.add_space(14.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                StatusPanels::new(self.shared.clone()).show(ui);
            });
    }

    fn show_laser_sidebar(&mut self, ui: &mut egui::Ui) {
        let laser_online = self.laser_shared.lock().is_ok_and(|obs| obs.is_online());

        Self::white_card(ui, "数据源", |ui| {
            Self::status_chip(ui, laser_online, "Laser UDP");
            ui.add_space(12.0);
            egui::Grid::new("laser_conn_grid")
                .num_columns(2)
                .min_col_width(78.0)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Port")
                            .color(theme::text_muted())
                            .size(13.0),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.laser_port)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();
                });
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Reconnect laser listener"),
                )
                .clicked()
            {
                self.reconnect_laser();
            }
        });

        ui.add_space(14.0);
        Self::white_card(ui, "脚本控制", |ui| {
            let running = self.script_runner.is_running();
            let daemon_ok = script_runner::daemon_alive();
            let active_label = self
                .script_runner
                .active()
                .map(|s| s.label())
                .unwrap_or("Idle");

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("状态:")
                        .color(theme::text_muted())
                        .size(13.0),
                );
                Self::status_chip(ui, running, active_label);
            });
            if daemon_ok && !running {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("daemon 存活 (可通过流控制发送命令)")
                        .color(theme::text_faint())
                        .size(11.0),
                );
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("敌方颜色:")
                        .color(theme::text_muted())
                        .size(13.0),
                );
                egui::ComboBox::from_id_salt("enemy_color")
                    .selected_text(self.enemy_color.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.enemy_color,
                            EnemyColor::Red,
                            "Red",
                        );
                        ui.selectable_value(
                            &mut self.enemy_color,
                            EnemyColor::Blue,
                            "Blue",
                        );
                        ui.selectable_value(
                            &mut self.enemy_color,
                            EnemyColor::Auto,
                            "Auto",
                        );
                    });
            });
            ui.add_space(4.0);
            ui.checkbox(&mut self.stream_on_start, "启动时推流");
            ui.checkbox(&mut self.record_on_start, "启动时内录");
            ui.add_space(6.0);
            let scripts = [
                [LaserScript::Competition, LaserScript::Preview],
                [LaserScript::Stream, LaserScript::Record],
            ];
            ui.columns(2, |columns| {
                for (row_index, row) in scripts.iter().enumerate() {
                    for (column, script) in columns.iter_mut().zip(row.iter()) {
                        let label = script.label();
                        if column
                            .add_sized(
                                [column.available_width(), 30.0],
                                egui::Button::new(label),
                            )
                            .clicked()
                        {
                            if let Err(e) = self.script_runner.start(*script) {
                                log::error!("Failed to start {}: {}", label, e);
                            } else if script.is_daemon() {
                                let enemy_cmd = self.enemy_color.fifo_cmd().to_owned();
                                let stream_cmd = if self.stream_on_start {
                                    "stream on"
                                } else {
                                    "stream off"
                                }
                                .to_owned();
                                let record_cmd = if self.record_on_start {
                                    "record on"
                                } else {
                                    "record off"
                                }
                                .to_owned();
                                std::thread::spawn(move || {
                                    for _ in 0..100 {
                                        let ok = script_runner::send_fifo(&enemy_cmd).is_ok()
                                            && script_runner::send_fifo(&stream_cmd).is_ok()
                                            && script_runner::send_fifo(&record_cmd).is_ok();
                                        if ok {
                                            return;
                                        }
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            50,
                                        ));
                                    }
                                    log::warn!(
                                        "Timed out sending launch config to daemon"
                                    );
                                });
                            }
                        }
                    }
                    if row_index + 1 < scripts.len() {
                        for column in &mut columns[..] {
                            column.add_space(6.0);
                        }
                    }
                }
            });
            if running {
                ui.add_space(10.0);
                if ui
                    .add_sized(
                        [ui.available_width(), 30.0],
                        egui::Button::new("Stop"),
                    )
                    .clicked()
                {
                    self.script_runner.stop();
                }
            }
        });

        ui.add_space(14.0);
        Self::white_card(ui, "比赛进程", |ui| {
            let sdr_ok = self.script_runner.is_sdr_running();
            let unity_ok = self.script_runner.is_unity_running();

            // SDR 桥接
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SDR:")
                        .color(theme::text_muted())
                        .size(13.0),
                );
                Self::status_chip(ui, sdr_ok, if sdr_ok { "Running" } else { "Idle" });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if sdr_ok {
                        if ui
                            .add_sized([72.0, 24.0], egui::Button::new("Stop"))
                            .clicked()
                        {
                            self.script_runner.stop_sdr();
                        }
                    } else {
                        if ui
                            .add_sized([72.0, 24.0], egui::Button::new("Start"))
                            .clicked()
                        {
                            if let Err(e) = self.script_runner.start_sdr() {
                                log::error!("Failed to start SDR: {}", e);
                            }
                        }
                    }
                });
            });
            ui.add_space(2.0);

            // Unity RADAR
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Radar:")
                        .color(theme::text_muted())
                        .size(13.0),
                );
                Self::status_chip(ui, unity_ok, if unity_ok { "Running" } else { "Idle" });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if unity_ok {
                        if ui
                            .add_sized([72.0, 24.0], egui::Button::new("Stop"))
                            .clicked()
                        {
                            self.script_runner.stop_unity();
                        }
                    } else {
                        if ui
                            .add_sized([72.0, 24.0], egui::Button::new("Start"))
                            .clicked()
                        {
                            if let Err(e) = self.script_runner.start_unity() {
                                log::error!("Failed to start Unity: {}", e);
                            }
                        }
                    }
                });
            });

            // Start All — 顺序启动 SDR → Laser
            ui.add_space(10.0);
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Start All (SDR → Laser Competition)"),
                )
                .clicked()
            {
                let enemy_cmd = self.enemy_color.fifo_cmd().to_owned();
                let stream_cmd = if self.stream_on_start {
                    "stream on"
                } else {
                    "stream off"
                }
                .to_owned();
                let record_cmd = if self.record_on_start {
                    "record on"
                } else {
                    "record off"
                }
                .to_owned();
                if let Err(e) =
                    self.script_runner
                        .start_all(LaserScript::Competition)
                {
                    log::error!("Start All failed: {}", e);
                } else {
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            let ok = script_runner::send_fifo(&enemy_cmd).is_ok()
                                && script_runner::send_fifo(&stream_cmd).is_ok()
                                && script_runner::send_fifo(&record_cmd).is_ok();
                            if ok {
                                return;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        log::warn!("Timed out sending config after Start All");
                    });
                }
            }

            // Stop All
            if sdr_ok || unity_ok || self.script_runner.is_running() {
                ui.add_space(6.0);
                if ui
                    .add_sized(
                        [ui.available_width(), 30.0],
                        egui::Button::new("Stop All"),
                    )
                    .clicked()
                {
                    self.script_runner.stop_all();
                }
            }
        });

        ui.add_space(14.0);
        Self::white_card(ui, "流控制", |ui| {
            ui.columns(2, |columns| {
                if columns[0]
                    .add_sized(
                        [columns[0].available_width(), 32.0],
                        egui::Button::new("Stream on"),
                    )
                    .clicked()
                {
                    self.send_laser_command("stream on");
                }
                if columns[1]
                    .add_sized(
                        [columns[1].available_width(), 32.0],
                        egui::Button::new("Stream off"),
                    )
                    .clicked()
                {
                    self.send_laser_command("stream off");
                }
            });
        });

        ui.add_space(14.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                LaserPanel::new(self.laser_shared.clone(), self.video_shared.clone())
                    .show_analysis_sidebar(ui);
            });
    }

    fn white_card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
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
                    ui.label(egui::RichText::new(title).color(theme::text()).size(16.0));
                });
                ui.add_space(10.0);
                add_contents(ui);
            });
    }

    fn status_chip(ui: &mut egui::Ui, ok: bool, label: &str) {
        let fill = if ok {
            theme::success_bg()
        } else {
            theme::error_bg()
        };
        let text = if ok { theme::GREEN } else { theme::RED };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(egui::CornerRadius::same(255))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("● {}", label))
                        .color(text)
                        .size(12.0),
                );
            });
    }

    fn setup_fonts(&self, ctx: &egui::Context) {
        FONT_ONCE.call_once(|| {
            let mut fonts = egui::FontDefinitions::default();

            // JetBrains Maple Mono: preferred proportional font for Latin/UI
            if let Ok(data) =
                std::fs::read("/usr/share/fonts/TTF/JetBrains-Maple-Mono-NF-XX-XX/JetBrainsMapleMono-Regular.ttf")
            {
                log::info!("Loaded JetBrains Maple Mono (Latin + CJK)");
                fonts.font_data.insert(
                    "maple".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "maple".to_owned());
            }

            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf") {
                log::info!("Loaded JetBrains Mono (monospace)");
                fonts.font_data.insert(
                    "jb_mono".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "jb_mono".to_owned());
            }

            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/LXGWWenKaiMonoGBScreen.ttf") {
                log::info!("Loaded LXGW WenKai Mono GB Screen (mono CJK fallback)");
                fonts.font_data.insert(
                    "lxgw_mono".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("lxgw_mono".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("lxgw_mono".to_owned());
            }

            ctx.set_fonts(fonts);
        });
    }

    fn apply_theme(&self, ctx: &egui::Context) {
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

    fn update_connection_status(&mut self) {
        if let Ok(info) = self.shared.lock() {
            let is_default =
                info.hero_position == [0, 0] && info.hero_blood == 0 && info.hero_ammunition == 0;

            if !is_default {
                self.connection_status = ConnectionStatus::Connected;
                self.last_update = Some(std::time::Instant::now());
                self.data_count += 1;
                self.error_message = None;
                self.rerun_viz.log_all(&info);
            } else if let Some(last) = self.last_update {
                if last.elapsed().as_secs() > 5 {
                    self.connection_status = ConnectionStatus::Disconnected;
                    self.error_message = Some("Connection lost".to_string());
                }
            }
        }
    }
}
