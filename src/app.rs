use std::sync::{Arc, Mutex, Once};
use std::thread;

use tokio::sync::watch;

use crate::laser_protocol::LaserObservation;
use crate::protocol::RoboMasterSignalInfo;
use crate::rerun_viz::RerunVisualizer;
use crate::tcp_client;
use crate::theme;
use crate::udp_client;
use crate::video_stream;
use crate::video_stream::VideoFrame;
use crate::widgets::{LaserPanel, MinimapWidget, StatusPanels};

static FONT_ONCE: Once = Once::new();

#[derive(PartialEq, Clone, Copy)]
enum ActiveTab {
    Radar,
    Laser,
}

pub struct RadarApp {
    active_tab: ActiveTab,

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
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true);
            // O_NONBLOCK for FIFO — don't hang if nobody is reading
            opts.custom_flags(libc::O_NONBLOCK);
            match opts.open("/tmp/laser_cmd") {
                Ok(mut fifo) => {
                    let _ = writeln!(fifo, "{cmd}");
                    log::info!("Sent laser command: {}", cmd);
                }
                Err(e) => {
                    log::warn!("Failed to send laser command: {}", e);
                }
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
        self.update_connection_status();
        self.apply_theme(ctx);

        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme::MANTLE)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let radar_selected = self.active_tab == ActiveTab::Radar;
                    let laser_selected = self.active_tab == ActiveTab::Laser;

                    if ui
                        .selectable_label(radar_selected, egui::RichText::new("radar hud").size(16.0))
                        .clicked()
                    {
                        self.active_tab = ActiveTab::Radar;
                    }
                    if ui
                        .selectable_label(laser_selected, egui::RichText::new("laser hud").size(16.0))
                        .clicked()
                    {
                        self.active_tab = ActiveTab::Laser;
                    }

                    ui.separator();

                    match self.active_tab {
                        ActiveTab::Radar => {
                            match self.connection_status {
                                ConnectionStatus::Connected => {
                                    ui.colored_label(theme::CONNECTED, "● Connected")
                                }
                                ConnectionStatus::Disconnected => {
                                    ui.colored_label(theme::DISCONNECTED, "● Disconnected")
                                }
                            };

                            ui.separator();

                            ui.label(
                                egui::RichText::new("IP:")
                                    .color(theme::SUBTEXT0)
                                    .size(14.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.ip).desired_width(120.0),
                            );
                            ui.label(
                                egui::RichText::new("Port:")
                                    .color(theme::SUBTEXT0)
                                    .size(14.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.port).desired_width(60.0),
                            );

                            if ui.button("Connect").clicked() {
                                self.reconnect();
                            }
                        }
                        ActiveTab::Laser => {
                            let laser_online = self
                                .laser_shared
                                .lock()
                                .is_ok_and(|obs| obs.is_online());

                            if laser_online {
                                ui.colored_label(theme::CONNECTED, "● Laser Online");
                            } else {
                                ui.colored_label(theme::DISCONNECTED, "● Laser Offline");
                            }

                            ui.separator();

                            ui.label(
                                egui::RichText::new("Port:")
                                    .color(theme::SUBTEXT0)
                                    .size(14.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.laser_port).desired_width(60.0),
                            );

                            if ui.button("Connect").clicked() {
                                self.reconnect_laser();
                            }

                            ui.separator();

                            if ui.button("Stream On").clicked() {
                                self.send_laser_command("stream on");
                            }
                            if ui.button("Stream Off").clicked() {
                                self.send_laser_command("stream off");
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(last) = self.last_update {
                            let elapsed = last.elapsed().as_secs_f32();
                            ui.label(
                                egui::RichText::new(format!("{:.1}s", elapsed))
                                    .color(theme::OVERLAY0)
                                    .size(14.0),
                            );
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme::MANTLE)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let uptime = self.start_time.elapsed().as_secs();
                    ui.label(
                        egui::RichText::new(format!("Uptime: {}s", uptime))
                            .color(theme::SUBTEXT0)
                            .size(12.0),
                    );

                    match self.active_tab {
                        ActiveTab::Radar => {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Data: {}", self.data_count))
                                    .color(theme::SUBTEXT0)
                                    .size(12.0),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Target: {}:{}", self.ip, self.port))
                                    .color(theme::SUBTEXT0)
                                    .size(12.0),
                            );
                        }
                        ActiveTab::Laser => {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Laser UDP: 0.0.0.0:{}", self.laser_port))
                                    .color(theme::SUBTEXT0)
                                    .size(12.0),
                            );
                        }
                    }

                    if let Some(err) = &self.error_message {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    egui::RichText::new(format!("⚠ {}", err))
                                        .color(theme::RED)
                                        .size(12.0),
                                );
                            },
                        );
                    }
                });
            });

        match self.active_tab {
            ActiveTab::Radar => {
                egui::SidePanel::left("minimap_panel")
                    .default_width(420.0)
                    .frame(egui::Frame::new().fill(theme::BASE).inner_margin(12))
                    .show(ctx, |ui| {
                        MinimapWidget::new(self.shared.clone()).show(ui);
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(theme::BASE).inner_margin(16))
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            StatusPanels::new(self.shared.clone()).show(ui);
                        });
                    });
            }
            ActiveTab::Laser => {
                self.ensure_video_started();

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(theme::BASE).inner_margin(16))
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            LaserPanel::new(self.laser_shared.clone(), self.video_shared.clone()).show(ui);
                        });
                    });
            }
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

impl RadarApp {
    fn setup_fonts(&self, ctx: &egui::Context) {
        FONT_ONCE.call_once(|| {
            let mut fonts = egui::FontDefinitions::default();

            if let Ok(data) =
                std::fs::read("/usr/share/fonts/TTF/JetBrainsMonoNerdFontPropo-Regular.ttf")
            {
                log::info!("Loaded JetBrainsMono NFP (proportional English)");
                fonts.font_data.insert(
                    "jb_propo".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "jb_propo".to_owned());
            }

            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/LXGWWenKaiGBScreen.ttf") {
                log::info!("Loaded LXGW WenKai GB Screen (CJK fallback)");
                fonts.font_data.insert(
                    "lxgw".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("lxgw".to_owned());
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
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("lxgw_mono".to_owned());
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
