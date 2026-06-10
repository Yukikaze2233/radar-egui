use std::sync::Once;

use self::video_texture::VideoTextureCache;
use crate::rerun_viz::RerunVisualizer;
use crate::runtime::{LaserRuntime, RadarRuntime, VideoRuntime};
use crate::services::process_control::ProcessControl;
use crate::state::{LaserObservationReader, RadarFeedReader};
use crate::theme;
use crate::laser::video::VideoFrameReader;
use crate::widgets::{LaserPanel, MinimapWidget};

mod assets;
mod connection;
mod theme_apply;
mod video_texture;
mod view;

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

    fn sdr_arg(&self) -> &str {
        match self {
            EnemyColor::Red | EnemyColor::Auto => "red",
            EnemyColor::Blue => "blue",
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

    radar_feed: RadarFeedReader,
    connection_status: ConnectionStatus,
    last_update: Option<std::time::Instant>,
    radar_runtime: RadarRuntime,
    ip: String,
    port: String,
    error_message: Option<String>,
    data_count: u64,
    last_logged_radar_version: u64,
    start_time: std::time::Instant,
    rerun_viz: RerunVisualizer,

    laser_feed: LaserObservationReader,
    laser_runtime: LaserRuntime,
    laser_port: String,
    video_feed: VideoFrameReader,
    video_runtime: VideoRuntime,
    laser_video_texture: VideoTextureCache,

    process_control: ProcessControl,
    camera_device: String,
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
        let (radar_feed, radar_writer) = RadarFeedReader::new_pair();
        let radar_runtime = RadarRuntime::start("127.0.0.1:2000", radar_writer);

        let (laser_feed, laser_writer) = LaserObservationReader::new_pair();
        let laser_runtime = LaserRuntime::new(laser_writer);
        let laser_port = 5001;

        let (video_feed, video_writer) = VideoFrameReader::new_pair();
        let video_runtime = VideoRuntime::new(video_writer);

        Self {
            active_tab: ActiveTab::Radar,
            dark_mode: false,
            minimap_texture: None,
            minimap_texture_failed: false,
            minimap_pan: egui::vec2(0.0, MINIMAP_DEFAULT_PAN_Y),
            minimap_zoom: 1.0,
            logo_texture: None,
            logo_texture_failed: false,
            radar_feed,
            connection_status: ConnectionStatus::Disconnected,
            last_update: None,
            radar_runtime,
            ip: "127.0.0.1".to_string(),
            port: "2000".to_string(),
            error_message: None,
            data_count: 0,
            last_logged_radar_version: 0,
            start_time: std::time::Instant::now(),
            rerun_viz: RerunVisualizer::new(),
            laser_feed,
            laser_runtime,
            laser_port: laser_port.to_string(),
            video_feed,
            video_runtime,
            laser_video_texture: VideoTextureCache::default(),
            process_control: ProcessControl::new(),
            camera_device: "/dev/laser_capture".to_string(),
            enemy_color: EnemyColor::Auto,
            stream_on_start: true,
            record_on_start: false,
        }
    }
}

impl RadarApp {
    fn reconnect(&mut self) {
        self.connection_status = ConnectionStatus::Disconnected;
        self.last_update = None;
        self.error_message = None;
        self.data_count = 0;
        self.last_logged_radar_version = 0;

        self.radar_feed.reset_metadata();

        let addr = format!("{}:{}", self.ip, self.port);
        self.radar_runtime.restart(&addr);
    }

    fn reconnect_laser(&mut self) {
        let port: u16 = self.laser_port.parse().unwrap_or(5001);
        self.laser_runtime.restart(port);
    }

    fn ensure_laser_started(&mut self) {
        let port: u16 = self.laser_port.parse().unwrap_or(5001);
        self.laser_runtime.ensure_started(port);
    }

    fn ensure_video_started(&mut self) {
        self.video_runtime.ensure_started();
    }
}

impl eframe::App for RadarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_fonts(ctx);
        theme::set_dark_mode(self.dark_mode);
        self.ensure_minimap_texture(ctx);
        self.ensure_logo_texture(ctx);
        let radar_snapshot = self.radar_feed.snapshot();
        self.update_connection_status(radar_snapshot.as_ref());
        self.apply_theme(ctx);
        self.process_control.trigger_pending_start_all();

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
                        self.show_radar_sidebar(ui, radar_snapshot.as_ref());
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
                                MinimapWidget::new().show_with_state(
                                    ui,
                                    radar_snapshot.as_ref().map(|snapshot| &snapshot.signal),
                                    self.minimap_texture.as_ref(),
                                    &mut self.minimap_pan,
                                    &mut self.minimap_zoom,
                                );
                            });
                        });
                    });
            }
            ActiveTab::Laser => {
                self.ensure_laser_started();
                self.ensure_video_started();
                let laser_snapshot = self.laser_feed.snapshot();
                self.laser_video_texture.refresh(ctx, &self.video_feed);

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
                        self.show_laser_sidebar(ui, laser_snapshot.as_ref());
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
                                    LaserPanel::new().show_video_stage(
                                        ui,
                                        laser_snapshot
                                            .as_ref()
                                            .map(|snapshot| &snapshot.observation),
                                        self.laser_video_texture.texture(),
                                    );
                                },
                            );
                        });
                    });
            }
        }

        self.show_theme_toggle(ctx);

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}
